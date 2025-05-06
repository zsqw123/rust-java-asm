use crate::app::EguiApp;
use egui::{Context, Ui};
use java_asm_server::AsmServer;
use std::ops::Deref;

impl EguiApp {
    pub(crate) fn top_bar(&mut self, ctx: &Context) {
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.vertical(|ui| {
                let server_app = &mut self.server_app;
                // loading state
                let locked_top = server_app.top().lock();
                let loading_state = &locked_top.loading_state;
                if loading_state.in_loading {
                    ui.horizontal(|ui| {
                        ui.label(format!("Loading... {:.2}%", loading_state.loading_progress * 100.0));
                    });
                }
                drop(locked_top);

                self.interaction_area(ui);
            });
        });
    }

    fn interaction_area(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.button("📂 Open...").clicked() {
                AsmServer::dialog_to_open_file(
                    self.server.clone(), self.server_app.clone(),
                );
            }
            self.file_path_input(ui);
        });
    }

    fn file_path_input(&mut self, ui: &mut Ui) {
        let server_locked = self.server.lock();
        let Some(server) = server_locked.deref() else { return; };
        let accessor = server.accessor.lock();
        let Some(accessor) = accessor.deref() else { return; };
        // let all_classes: Vec<String> = accessor.read_classes().iter().map(|s| s.to_string()).collect();
        let mut locked_top = self.server_app.top().lock();
        let Some(file_path) = &mut locked_top.file_path else { return; };
        ui.label(file_path.to_string());
        // ui.add(AutoCompleteTextEdit::new(
        //     file_path, all_classes,
        // ));
    }
}