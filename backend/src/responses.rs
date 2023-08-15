use serde::{Serialize, Deserialize};
use crate::requests;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Response {
    Ok,
    Error(Error),
    Contacts{contacts: Vec<String>},
    Message(requests::Message),
    History(Vec<requests::Message>)
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Error {
    data: String,
}


impl Response {
    pub fn to_json(&self) -> String{
        serde_json::to_string(&self).unwrap()
    }
}

impl Error {
    pub fn new(data: String) -> Self {
        Error { data }
    }
}