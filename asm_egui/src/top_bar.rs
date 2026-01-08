use crate::app::EguiApp;
use bit_set::BitSet;
use egui::containers::PopupCloseBehavior;
use egui::text::LayoutJob;
use egui::{popup_below_widget, Context, Id, Response, TextEdit, TextFormat, TextStyle, Ui};
use java_asm_server::ui::{Content, OpenFileMessage, Tab, UIMessage};
use java_asm_server::AsmServer;
use std::ops::Deref;
use std::time::Duration;

impl EguiApp {
    pub(crate) fn top_bar(&mut self, ctx: &Context) {
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.vertical(|ui| {
                let server_app = &mut self.ui_app;
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
                    self.server.clone(), self.ui_app.clone(),
                );
            }
            if self.server.lock().is_some() {
                self.locate_button(ui);
                self.export_button(ui);
                // searchable file path
                self.file_path_input(ui);
            }
        });
    }

    fn file_path_input(&mut self, ui: &mut Ui) {
        let mut locked_top = self.ui_app.top().lock();
        let mut file_path = &mut locked_top.file_path;

        let edit_path_ui = Self::file_path_input_area(ui, &mut file_path);

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

        if search_results.items.is_empty() { return; }
        popup_below_widget(
            ui, popup_id, &edit_path_ui,
            PopupCloseBehavior::CloseOnClickOutside, |ui| {
                ui.vertical(|ui| {
                    Self::popup_file_path_ui(self, ui);
                })
            });
    }

    fn locate_button(&mut self, ui: &mut Ui) {
        let Some(current_tab) = self.get_current_tab() else { return };
        if !ui.button("Locate").clicked() { return }
        let current_path = current_tab.file_key;
        let message = UIMessage::OpenFile(
            OpenFileMessage { path: current_path }
        );
        self.ui_app.send_message(message);
    }

    fn export_button(&mut self, ui: &mut Ui) {
        ui.menu_button("Export", |ui| {
            let current_tab = self.get_current_tab();
            let label_text = "Copy current content";
            let Some(current_tab) = current_tab else {
                ui.weak(label_text);
                return;
            };
            if ui.selectable_label(false, label_text).clicked() {
                let smali_content = current_tab.content.render(0);
                ui.ctx().copy_text(smali_content);
                self.toasts.success(
                    format!("{} content copied!", current_tab.file_key)
                ).duration(Duration::from_secs(3).into());
            }
        });
    }

    fn get_current_tab(&self) -> Option<Tab> {
        let locked_content = self.ui_app.content().lock();
        let Content { opened_tabs, selected } = locked_content.deref();
        let Some(selected) = selected else { return None; };
        opened_tabs.get(*selected).map(|tab| tab.clone())
    }

    fn file_path_input_area(ui: &mut Ui, file_path: &mut String) -> Response {
        let id_for_input_remaining = Id::new("file_path_input_area_remaining");
        let max_width = ui.max_rect().width();
        let last_time_remaining = ui
            .data(|data| data.get_temp(id_for_input_remaining)
                .unwrap_or(max_width));
        let target_width_for_content = max_width - last_time_remaining;

        let edit_path_ui = TextEdit::singleline(file_path)
            .hint_text("Enter class name to search...")
            .desired_width(target_width_for_content).show(ui).response;

        let remaining_width = ui.min_rect().width() - target_width_for_content;
        ui.data_mut(|data| {
            data.insert_temp(id_for_input_remaining, remaining_width);
        });
        edit_path_ui
    }

    fn popup_file_path_ui(&mut self, ui: &mut Ui) {
        let search_results = self.ui_app.top().lock().search_result.clone();
        let style = ui.style();
        let font = TextStyle::Monospace.resolve(&style);

        let dark_mode = style.visuals.dark_mode;
        let smali_style = if dark_mode { crate::smali::SmaliStyle::DARK } else { crate::smali::SmaliStyle::LIGHT };

        let dft_color = style.visuals.text_color();
        let dft_text_format = TextFormat::simple(font.clone(), dft_color);
        let highlight_color = smali_style.desc;
        let highlight_text_format = TextFormat::simple(font, highlight_color);


        for result in search_results.items {
            let path = result.item.to_string();
            let sections = Self::get_highlight_sections(&path, result.indices);
            let mut text_layout_job = LayoutJob::default();
            for (section, highlighted) in sections {
                if highlighted {
                    text_layout_job.append(&section, 0.0, highlight_text_format.clone())
                } else {
                    text_layout_job.append(&section, 0.0, dft_text_format.clone())
                }
            }
            let selectable_label = ui.selectable_label(false, text_layout_job);
            if selectable_label.clicked() {
                let message = UIMessage::OpenFile(
                    OpenFileMessage { path: result.item }
                );
                self.ui_app.send_message(message);
                ui.memory_mut(|m| m.close_popup());
            }
        }
    }

    fn get_highlight_sections(path: &str, bits: BitSet) -> Vec<(String, bool)> {
        let mut current_section = String::new();
        let mut cur_highlighted = false;

        let mut sections = vec![];
        for (index, ch) in path.chars().enumerate() {
            let target_highlighted = bits.contains(index);
            if current_section.is_empty() {
                // first char in this section, init
                current_section.push(ch);
                cur_highlighted = target_highlighted;
            } else if cur_highlighted == target_highlighted {
                // same highlight
                current_section.push(ch);
            } else {
                // different color, start new section
                sections.push((current_section, cur_highlighted));
                current_section = ch.to_string();
                cur_highlighted = target_highlighted;
            }
        }

        // last section
        if !current_section.is_empty() {
            sections.push((current_section, cur_highlighted));
        }
        sections
    }
}