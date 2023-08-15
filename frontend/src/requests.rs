use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Request {
    Message(Message),
    Login(Login),
    Register(Register),
    GetContacts,
    GetHistory{username: String},
    AddContact{username: String}
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message{
    pub username: String,
    pub text: String,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Login{
    pub username: String,
    pub password: String,
    #[serde(skip)]
    pub error:Option<String>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Register {
    pub username: String,
    pub password: String,
}




impl Message{
    pub fn new(username: String, text: String) -> Self{
        Message{username, text}
    }
}


impl Login{
    pub fn new(username: String, password: String) -> Self{
        let mut hasher = DefaultHasher::new();
        password.hash(&mut hasher);
        Login{username, password:hasher.finish().to_string(), error: None }
    }

    pub fn hash_password(&mut self){
        let mut hasher = DefaultHasher::new();
        self.password.hash(&mut hasher);
        self.password = hasher.finish().to_string();
    }
}

impl Register {
    pub fn new(username: String, password: String) -> Self{
        let mut hasher = DefaultHasher::new();
        password.hash(&mut hasher);
        Register{username, password:hasher.finish().to_string()}
    }
}
