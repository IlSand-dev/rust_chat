use eframe::egui;
use eframe::egui::{Context, TextEdit};
use crate::requests::Message;

pub struct ChatScreen {
    pub text: String,
    pub contact: String,
}

impl ChatScreen {
    pub fn update(&mut self, ctx: &Context, messages: &Vec<Message>) -> bool {
        let mut update = false;
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(self.contact.as_str());
            egui::ScrollArea::vertical().show(ui, |ui| {
                for message in messages {
                    ui.label(format!("{}: {}", message.username, message.text));
                }
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::BOTTOM), |ui| {
                if ui.add_sized([100., 100.], egui::Button::new("Send")).clicked() {
                    update = true;
                };
                ui.add_sized([ui.available_width(), 100.],TextEdit::singleline(&mut self.text).hint_text("Message"))
            })
        });
        update
    }
}
