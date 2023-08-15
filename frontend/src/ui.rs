use std::sync::Arc;
use eframe::{egui, Error, Frame};
use eframe::egui::Context;
use crate::requests::{Register, Request, Login, Message};
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use eframe::egui::Key::V;
use eframe::egui::TextEdit;
use crate::models::User;
use crate::responses::Response;
use crate::ui::chat::ChatScreen;
use crate::ui::contacts::ContactsScreen;

use crate::ui::login::LoginScreen;
use crate::ui::register::RegisterScreen;

mod login;
mod register;
mod contacts;
mod chat;


pub struct App {
    view: View,
    tx: Sender<Request>,
    rx: Receiver<Response>,
    user: String,
    contacts: Vec<String>,
    messages: Vec<Message>,
    error: Option<String>,
    add_window: bool,
    add_username: String,
}

impl App {
    pub fn new(tx: Sender<Request>, rx: Receiver<Response>, contacts: Vec<String>) -> Self {
        App { view: View::default(), tx, rx, user: "".to_string(), contacts,messages: Vec::new(), error: None, add_window: false, add_username: "".to_string() }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        match self.view {
            View::Login(ref mut login) => {
                let (update, register) = login.update(ctx, &self.error);
                if update {
                    let request = Request::Login(Login::new(login.username.clone(), login.password.clone()));
                    self.tx.send(request).  unwrap();
                }
                if register {
                    self.view = View::Register(RegisterScreen::default());
                    ctx.request_repaint();
                    return;
                }
                if let Ok(response) = self.rx.try_recv() {
                    match response {
                        Response::Ok => {
                            self.user = login.username.clone();
                            self.tx.send(Request::GetContacts).unwrap();
                            self.view = View::Contacts(ContactsScreen::default());
                            ctx.request_repaint();
                        }
                        Response::Error(e) => {
                            self.error = Some(e.data);
                            ctx.request_repaint();
                        }
                        _ => {}
                    }
                }
            }
            View::Register(ref mut register) => {
                let (update, login) = register.update(ctx, &self.error);
                if update {
                    let request = Request::Register(Register::new(register.username.clone(), register.password.clone()));
                    if let Err(e) = self.tx.send(request) {
                        println!("{:?}", e);
                        self.error = Some(e.to_string());
                    };
                }
                if login {
                    self.view = View::Login(LoginScreen::default());
                    ctx.request_repaint();
                    return;
                }
                if let Ok(response) = self.rx.try_recv() {
                    match response {
                        Response::Ok => {
                            self.view = View::Login(LoginScreen::default());
                            ctx.request_repaint();
                            return;
                        }
                        Response::Error(e) => {
                            println!("{:?}", e);
                        }
                        _ => {}
                    }
                }
            }
            View::Contacts(ref mut contacts_screen) => {
                let (user, add) = contacts_screen.update(ctx, &self.contacts);
                if let Some(user) = user {
                    self.tx.send(Request::GetHistory {username: user.clone()}).unwrap();
                    self.view = View::Chat(ChatScreen{contact: user, text:"".to_string()});
                    ctx.request_repaint();
                }
                if add {
                    self.add_username = "".to_string();
                    self.add_window = true
                }
                if self.add_window {
                    let mut changed = false;
                    egui::Window::new("Enter username").show(ctx, |ui| {
                        ui.add(TextEdit::singleline(&mut self.add_username));
                        if ui.button("Find").clicked() {
                            self.add_window = false;
                            changed = true;
                        }
                    });
                    if changed{
                        self.tx.send(Request::AddContact{username: self.add_username.clone()}).unwrap();
                    }
                }
                if let Ok(response) = self.rx.try_recv() {
                    match response {
                        Response::Ok => {
                            self.tx.send(Request::GetContacts).unwrap();
                        }
                        Response::Error(e) => {
                            self.error = Some(e.data);
                        }
                        Response::Contacts{contacts} => {
                            self.contacts = contacts;
                        }
                        _ => {}
                    };
                };
            },
            View::Chat(ref mut chat) => {
                if chat.update(ctx, &self.messages){
                    self.messages.push(Message::new(self.user.clone(), chat.text.clone()));
                    self.tx.send(Request::Message(Message::new(chat.contact.clone(), chat.text.clone()))).unwrap();
                    chat.text = "".to_string();
                }
                if let Ok(response) = self.rx.try_recv(){
                    match response {
                        Response::Error(e) => {
                            self.error = Some(e.data)
                        }
                        Response::Message(message) => {
                            self.messages.push(message);
                        }
                        Response::History(history) => {
                            self.messages = history;
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}


pub enum View {
    Login(LoginScreen),
    Contacts(ContactsScreen),
    Register(RegisterScreen),
    Chat(ChatScreen)
}

impl Default for View {
    fn default() -> Self {
        View::Login(LoginScreen::default())
    }
}