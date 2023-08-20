use eframe::egui;
use eframe::egui::{Context, TextEdit};
use crate::requests::Message;

pub struct ChatScreen {
    pub text: String,
    pub contact: String,
    pub messages: Vec<Message>,
    pub error: Option<String>
}

impl ChatScreen {
    pub fn update(&mut self, ctx: &Context) -> (bool, bool) {
        let mut update = false;
        let mut back = false;
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Back").clicked(){
                back = true;
            }
            ui.heading(self.contact.as_str());
            egui::ScrollArea::vertical().show(ui, |ui| {
                for message in &self.messages {
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
        (update, back)
    }
}
