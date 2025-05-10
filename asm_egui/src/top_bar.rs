use crate::app::EguiApp;
use egui::containers::PopupCloseBehavior;
use egui::{popup_below_widget, Context, Ui};
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
        let mut locked_top = self.server_app.top().lock();
        let Some(file_path) = &mut locked_top.file_path else { return; };
        let edit_path_ui = ui.text_edit_singleline(file_path);

        let popup_id = ui.make_persistent_id("file_path_popup");
        if edit_path_ui.gained_focus() {
            let server_locked = self.server.lock();
            let Some(server) = server_locked.deref() else { return; };
            server.search(&mut locked_top);
            ui.memory_mut(|m| m.open_popup(popup_id));
        }

        let search_results = locked_top.search_result.clone();
        drop(locked_top);

        if search_results.is_empty() { return; }
        popup_below_widget(
            ui, popup_id, &edit_path_ui,
            PopupCloseBehavior::CloseOnClickOutside, |ui| {
                ui.vertical(|ui| {
                    for result in search_results {
                        if ui.label(result.to_string()).clicked() {
                            let server_locked = self.server.lock();
                            let Some(server) = server_locked.deref() else { return; };
                            let mut content_locked = self.server_app.content().lock();
                            let mut locked_top = self.server_app.top().lock();
                            let accessor = server.accessor.lock();
                            let Some(accessor) = accessor.deref() else { return; };
                            server.switch_or_open_lock_free(
                                &result, accessor, &mut content_locked, &mut locked_top,
                            );
                        }
                    }
                })
            });
    }
}