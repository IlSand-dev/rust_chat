use std::fs::{File, OpenOptions};
use std::io::ErrorKind::NotFound;
use std::io::Write;
use std::path::Path;
use rusqlite::{Connection, Error, Result};
use rusqlite::Error::QueryReturnedNoRows;
use models::User;
use crate::models;
use crate::requests::Message;

pub struct DbManager {
    conn: Connection,
}

impl DbManager {
    pub fn build(path: &str) -> Result<DbManager> {
        if Path::new(&path).exists() {
            let conn = Connection::open(path)?;
            Ok(DbManager { conn })
        } else {
            let conn = Connection::open(path)?;
            let manager = DbManager { conn };
            manager.create_tables()?;
            Ok(manager)
        }
    }

    fn create_tables(&self) -> Result<()> {
        self.conn.execute("PRAGMA foreign_keys = ON", ())?;
        self.conn.execute("CREATE TABLE users(username VARCHAR PRIMARY KEY, password VARCHAR)", ())?;
        self.conn.execute("CREATE TABLE connections(\
        id INTEGER PRIMARY KEY AUTOINCREMENT, \
        first VARCHAR, \
        second VARCHAR, \
        FOREIGN KEY (first) REFERENCES users (username) ON DELETE CASCADE, \
        FOREIGN KEY (second) REFERENCES users (username) ON DELETE CASCADE  \
        )", ())?;
        Ok(())
    }

    pub fn add_user(&self, username: String, password: String) -> Result<()> {
        self.conn.execute("INSERT INTO users(username, password) VALUES (?1, ?2)", (&username, &password))?;
        Ok(())
    }

    pub fn login_user(&self, username: String, password: String) -> Result<User> {
        Ok(self.conn.query_row("SELECT * FROM users WHERE username = ?1 AND password = ?2", (&username, &password), |row| {
            Ok(User::new(
                row.get(0)?,
                row.get(1)?,
            ))
        })?)
    }

    pub fn user_exists(&self, username: String) -> Result<bool> {
        match self.conn.query_row("SELECT * FROM users WHERE username = ?1", [&username], |row| Ok(())) {
            Ok(_) => Ok(true),
            Err(e) => {
                if e == QueryReturnedNoRows {
                    Ok(false)
                } else {
                    Err(e)
                }
            }
        }
    }

    pub fn add_connection(&self, first: &str, second: &str) -> Result<bool, String> {
        match self.conn.query_row("SELECT * FROM connections WHERE first=?1 AND second=?2 OR first=?2 AND second=?1", [&first, &second], |row| Ok(())) {
            Ok(_) => {
                Err("Connection exists".to_string())
            }
            Err(e) => {
                if e == QueryReturnedNoRows {
                    if let Err(e) = self.conn.execute("INSERT INTO connections(first, second) VALUES (?1, ?2)", [&first, &second]) {
                        Err(e.to_string())
                    } else {
                        Ok(true)
                    }
                } else {
                    Err(e.to_string())
                }
            }
        }
    }

    pub fn get_connections(&self, username: String) -> Result<Vec<String>> {
        let mut result:Vec<String> = vec![];
        if let Err(e) = self.conn.query_row("SELECT second FROM connections WHERE first=?1", [&username], |row| {
            result.push(row.get(0).unwrap());
            Ok(())
        }){
            if e != QueryReturnedNoRows{return Err(e)}
        };
        if let Err(e) = self.conn.query_row("SELECT first FROM connections WHERE second=?1", [&username], |row| {
            result.push(row.get(0).unwrap());
            Ok(())
        }){
            if e != QueryReturnedNoRows{return Err(e)}
        };;
        Ok(result)
    }

    pub fn get_history(&self, first: &str, second: &str) -> Result<File> {
        Ok(self.conn.query_row("SELECT id FROM connections WHERE first=?1 AND second=?2 OR first=?2 AND second=?1 ", [&first, &second], |row|{
            println!("{:?}", row);
            let file_name:i32 = row.get(0).unwrap();
            let file_name = file_name.to_string();
            match OpenOptions::new().read(true).write(true).open(&file_name) {
                Ok(file) => {
                    Ok(file)
                }
                Err(e) => {
                    let mut file = OpenOptions::new()
                        .read(true)
                        .write(true)
                        .create(true)
                        .open(&file_name)
                        .unwrap();
                    let vec:Vec<Message> = Vec::new();
                    file.write(serde_json::to_string(&vec).unwrap().as_bytes()).unwrap();
                    Ok(file)
                }
            }
        })?)
    }
}