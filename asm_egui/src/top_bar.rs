use crate::app::EguiApp;
use egui::containers::PopupCloseBehavior;
use egui::{popup_below_widget, Context, Id, Response, TextEdit, Ui};
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

        let edit_path_ui = Self::file_path_input_area(ui, file_path);

        let popup_id = Id::new("file_path_popup");
        if edit_path_ui.gained_focus() {
            let server_locked = self.server.lock();
            let Some(server) = server_locked.deref() else { return; };
            server.search(&mut locked_top);
            ui.memory_mut(|m| m.open_popup(popup_id));
        }

        if edit_path_ui.changed() {
            let server_locked = self.server.lock();
            let Some(server) = server_locked.deref() else { return; };
            server.search(&mut locked_top);
        }

        let search_results = locked_top.search_result.clone();
        drop(locked_top);

        if search_results.is_empty() { return; }
        popup_below_widget(
            ui, popup_id, &edit_path_ui,
            PopupCloseBehavior::CloseOnClickOutside, |ui| {
                ui.vertical(|ui| {
                    Self::popup_file_path_ui(self, ui);
                })
            });
    }

    fn file_path_input_area(ui: &mut Ui, file_path: &mut String) -> Response {
        let id_for_input_remaining = Id::new("file_path_input_area_remaining");
        let max_width = ui.max_rect().width();
        let last_time_remaining = ui
            .data(|data| data.get_temp(id_for_input_remaining)
                .unwrap_or(max_width));
        let target_width_for_content = max_width - last_time_remaining;

        let edit_path_ui = TextEdit::singleline(file_path)
            .desired_width(target_width_for_content).show(ui).response;

        let remaining_width = ui.min_rect().width() - target_width_for_content;
        ui.data_mut(|data| {
            data.insert_temp(id_for_input_remaining, remaining_width);
        });
        edit_path_ui
    }

    fn popup_file_path_ui(&mut self, ui: &mut Ui) {
        let search_results = self.server_app.top().lock().search_result.clone();
        for result in search_results {
            let selectable_label = ui.selectable_label(false, result.to_string());
            if selectable_label.clicked() {
                let server_locked = self.server.lock();
                let Some(server) = server_locked.deref() else { return; };
                server.switch_or_open(&result, &self.server_app);
            }
        }
    }
}