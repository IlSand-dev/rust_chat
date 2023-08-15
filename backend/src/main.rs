mod models;
mod requests;
mod db_manager;
mod responses;

use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};
use bytes::Bytes;
use rusqlite::{Connection, Result};
use serde::Serialize;
use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf};
use tokio::io::WriteHalf;
use tokio::sync::{broadcast, oneshot};
use tokio::sync::broadcast::{Sender, Receiver};
use tokio::task::JoinHandle;
use models::User;
use crate::db_manager::DbManager;
use crate::requests::{Login, Message, Requests};
use crate::responses::{Error, Response};

type Mng = Arc<Mutex<DbManager>>;
type Resp = (String, Response);

const ADDR: &str = "localhost:8000";
const DB_PATH: &str = "chat_db.sqlite3";

#[tokio::main]
async fn main() {
    let (tx, _) = broadcast::channel(16);
    let listener = TcpListener::bind(ADDR).await.unwrap();
    let mng: Mng = Arc::new(Mutex::new(DbManager::build(DB_PATH).unwrap()));
    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();
        println!("{:?}", addr);
        let tx = tx.clone();
        let rx = tx.subscribe();
        let mng = mng.clone();
        tokio::spawn(async move {
            process(socket, tx, rx, mng).await;
        });
    }
}

async fn process(mut socket: TcpStream, tx: Sender<Resp>, mut rx: Receiver<Resp>, mng: Mng) {
    let (mut rd, mut wr) = io::split(socket);
    let mut read_task: Option<JoinHandle<()>> = None;
    let mut write_task: Option<JoinHandle<()>> = None;
    loop {
        let mut buf = [0; 4096];
        let n = rd.read(&mut buf).await.unwrap();
        if n == 0 {
            if let Some(task) = read_task {
                task.abort();
            }
            if let Some(task) = write_task {
                task.abort();
            }
            return;
        }
        let request = serde_json::from_slice::<Requests>(&buf[0..n]);
        match request {
            Ok(rq) => match rq {
                Requests::Register(data) => {
                    let result = {
                        let mng = mng.lock().unwrap();
                        mng.add_user(data.username, data.password)
                    };
                    match result {
                        Ok(_) => {
                            wr.write_all(
                                Response::Ok.to_json().as_bytes()).await.unwrap()
                        }
                        Err(e) => {
                            eprintln!("Error during creating User: {e}");
                            wr.write_all(
                                Response::Error(Error::new(
                                    "User hasn't created".to_string())).to_json().as_bytes()
                            ).await.unwrap();
                        }
                    };
                }
                Requests::Login(data) => {
                    let user;
                    {
                        let mng = mng.lock().unwrap();

                        user = mng.login_user(data.username, data.password);
                    }
                    let user = match user {
                        Ok(user) => { user }
                        Err(_) => {
                            wr.write(
                                serde_json::to_string(&Response::Error(
                                    Error::new("Username or password is incorrect".to_string())
                                )).unwrap().as_bytes()
                            ).await.unwrap();
                            return;
                        }
                    };
                    wr.write_all(Response::Ok.to_json().as_bytes()).await.unwrap();
                    let username = user.name.clone();
                    read_task = Some(tokio::spawn(async move {
                        read_messages(rd, tx, username, mng).await;
                    }));
                    let username = user.name.clone();
                    write_task = Some(tokio::spawn(async move {
                        write_messages(wr, rx, username).await;
                    }));
                    break;
                }
                _ => {
                    wr.write_all(Response::Error(Error::new(
                        "Login required".to_string())).to_json().as_bytes()
                    ).await.unwrap();
                }
            }
            Err(e) => {
                println!("{}", e);
                break;
            }
        }
    }
}

async fn write_messages(mut wr: WriteHalf<TcpStream>, mut rx: Receiver<Resp>, username: String) {
    loop {
        let (resp_username, response) = rx.recv().await.unwrap();
        if resp_username == username {
            let msg = response.to_json();
            println!("{}", msg);
            wr.write(msg.as_bytes()).await.unwrap();
        }
    }
}

async fn read_messages(mut rd: ReadHalf<TcpStream>, mut tx: Sender<Resp>, username: String, db_manager: Mng) {
    loop {
        let mut buf = [0; 4096];
        let n = rd.read(&mut buf).await.unwrap();
        if n == 0 {
            return;
        }
        let request = serde_json::from_slice::<Requests>(&buf[0..n]);
        match request {
            Ok(rq) => match rq {
                Requests::Message(data) => {
                    let message = Message::new(username.clone(), data.text);
                    let response = {
                        let db_manager = db_manager.lock().unwrap();
                        match db_manager.get_history(username.as_str(), data.username.as_str()) {
                            Ok(mut file) => {
                                let mut buf = [0; 16384];
                                let n = file.read(&mut buf).unwrap();
                                let mut history: Vec<Message> = serde_json::from_slice(&buf[0..n]).unwrap();
                                history.push(message.clone());
                                file.set_len(0).unwrap();
                                file.seek(SeekFrom::Start(0)).unwrap();
                                file.write(serde_json::to_string(&history).unwrap().as_bytes()).unwrap();
                                Response::Ok
                            }
                            Err(e) => { Response::Error(Error::new(e.to_string())) }
                        }
                    };
                    match response {
                        Response::Ok => { tx.send((data.username.clone(), Response::Message(message))).unwrap(); }
                        Response::Error(error) => { tx.send((username.clone(), Response::Error(error))).unwrap(); }
                        _ => {}
                    };
                }
                Requests::GetContacts => {
                    let response = {
                        let db_manager = db_manager.lock().unwrap();
                        match db_manager.get_connections(username.clone()) {
                            Ok(contacts) => { Response::Contacts { contacts } }
                            Err(e) => { Response::Error(Error::new(e.to_string())) }
                        }
                    };
                    println!("{}", tx.receiver_count());
                    tx.send((username.clone(), response)).unwrap();
                }
                Requests::AddContact { username: second_user } => {
                    let response = {
                        let db_manager = db_manager.lock().unwrap();
                        match db_manager.add_connection(username.as_str(), second_user.as_str()) {
                            Ok(_) => { Response::Ok }
                            Err(e) => { Response::Error(Error::new(e.to_string())) }
                        }
                    };
                    tx.send((username.clone(), response)).unwrap();
                }
                Requests::GetHistory { username: second_user } => {
                    let response = {
                        let db_manager = db_manager.lock().unwrap();
                        match db_manager.get_history(username.as_str(), second_user.as_str()) {
                            Ok(mut file) => {
                                let mut buf = [0; 16384];
                                let n = file.read(&mut buf).unwrap();
                                let history: Vec<Message> = serde_json::from_slice(&buf[0..n]).unwrap();
                                Response::History(history)
                            }
                            Err(e) => { Response::Error(Error::new(e.to_string())) }
                        }
                    };
                    tx.send((username.clone(), response)).unwrap();
                }
                _ => {}
            }
            Err(e) => {
                println!("{}", e);
                break;
            }
        }
    }
}