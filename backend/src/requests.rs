use std::hash::{Hash, Hasher};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum Requests {
    Message(Message),
    Login(Login),
    Register(Register),
    GetContacts,
    GetHistory{username: String},
    AddContact{username: String}
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub username: String,
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Login {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Register {
    pub username: String,
    pub password: String,
}



impl Message {
    pub fn new(username: String, text: String) -> Self {
        Message { username, text }
    }
}


impl Login {
    pub fn new(username: String, password: String) -> Self {
        Login {
            username,
            password,
        }
    }
}

impl Register {
    pub fn new(username: String, password: String) -> Self {
        Register { username, password }
    }
}

