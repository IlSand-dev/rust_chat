use eframe::egui;

#[derive(Default)]
pub struct ContactsScreen{
    pub error: Option<String>
}

impl ContactsScreen{
    pub fn update(&mut self, ctx: &egui::Context, contacts: &Vec<String>) -> (Option<String>, bool){
        let mut user = None;
        let mut add = false;
        egui::CentralPanel::default().show(ctx, |ui|{
            ui.heading("Contacts");
            egui::ScrollArea::vertical().show(ui, |ui|{
                for contact in contacts{
                    if ui.button(contact).clicked(){
                        user = Some(contact.clone());
                    }
                }
                if ui.button("Add contact").clicked(){
                    add = true
                }
            })
        });
        (user, add)
    }
}