use std::sync::Arc;
use eframe::egui::{self, Color32, TextEdit};
use crate::requests::Login;

#[derive(Default)]
pub struct LoginScreen{
    pub username: String,
    pub password: String,
}

impl LoginScreen{
    pub fn update(&mut self, ctx: &egui::Context, error: &Option<String>) -> (bool, bool){
        let mut update = false;
        let mut register = false;
        egui::CentralPanel::default().show(ctx, |ui|{
            ui.heading("Argix");
            ui.add_space(10.);
            ui.group(|ui|{
                ui.set_max_width(300.);
                if let Some(ref error) = error{
                    ui.colored_label(Color32::from_rgb(255, 0, 0), error);
                }
                ui.vertical(|ui|{
                    ui.heading("Log in");
                    ui.add(TextEdit::singleline(&mut self.username).hint_text("Username"));
                    ui.add(TextEdit::singleline(&mut self.password).hint_text("Password").password(true));
                    if ui.button("Log in").clicked(){
                        update = true;
                    }
                    if ui.button("Register").clicked(){
                        register = true;
                    }
                });
            })
        });
        (update, register)
    }
}