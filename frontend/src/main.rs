#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ui;
mod requests;
mod models;
mod settings;
mod responses;

use std::io::Read;
use std::sync::mpsc;
use serde::{Serialize, Deserialize};
use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::io::{ReadHalf};
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use requests::Request;
use crate::responses::Response;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    let rt = Runtime::new().expect("Unable to create runtime");

    let (tx_req, rx_req) = mpsc::channel::<Request>();
    let (tx_resp, rx_resp) = mpsc::channel::<Response>();

    let _enter = rt.enter();

    std::thread::spawn(move || {
        rt.block_on(async {
            loop {
                let mut server = TcpStream::connect(settings::SERVER).await;
                let tx_resp = tx_resp.clone();
                match server {
                    Ok(server) =>{
                        let (mut rd, mut wr) = io::split(server);
                        let (kill_tx, mut kill_rx) = tokio::sync::oneshot::channel();
                        tokio::spawn(async move {
                            read_messages(rd, tx_resp).await;
                            kill_tx.send("kill").unwrap();
                        });
                        loop {
                            match rx_req.try_recv() {
                                Ok(request) => {
                                    wr.write(serde_json::to_string(&request).unwrap().as_bytes()).await.unwrap();
                                }
                                Err(_) => {}
                            }
                            if let Ok(_) = kill_rx.try_recv() {
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("{}", e)
                    }
                }
            }
        })
    });

    eframe::run_native(
        "RustChat",
        eframe::NativeOptions::default(),
        Box::new(|cc| Box::new(ui::App::new(tx_req, rx_resp, Vec::new()))),
    )
}


async fn read_messages(mut rd: ReadHalf<TcpStream>, mut tx: mpsc::Sender<Response>) {
    loop {
        let mut buf = [0; 4096];
        let n = rd.read(&mut buf).await.unwrap();
        if n == 0{
            break
        }
        println!("{}", String::from_utf8_lossy(&buf[0..n]));
        match serde_json::from_slice::<Response>(&buf[0..n]) {
            Ok(response) => {
                println!("{:?}", response);
                tx.send(response).unwrap();
            }
            Err(e) => {  }
        }
    }
}