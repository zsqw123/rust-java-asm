use crate::app::EguiApp;
use egui::text::LayoutJob;
use egui::{ScrollArea, TextStyle};
use java_asm_server::ui::{Content, FileEntry, FileInfo, RawDirInfo, Tab};
use java_asm_server::AsmServer;
use std::ops::Deref;
use std::sync::Arc;

pub fn render_dir(ui: &mut egui::Ui, app: &mut EguiApp) {
    let server_app = &app.server_app;
    let mut server_app_left = server_app.left().lock();
    let mut server_app_content = server_app.content().lock();
    let entries = &mut server_app_left.root_node.visible_items();
    let server = app.server.lock();
    if let Some(server) = server.deref() {
        let content = &mut server_app_content;
        let row_height = ui.spacing().interact_size.y;
        ScrollArea::vertical().auto_shrink(false)
            .show_rows(ui, row_height, entries.len(), |ui, range| {
                for i in range {
                    let entry = &mut entries[i];
                    match entry {
                        FileEntry::Dir(raw_dir) => {
                            render_dir_raw(ui, raw_dir);
                        }
                        FileEntry::File(file_info) => {
                            render_file(ui, file_info, server, content);
                        }
                    }
                }
            });
    }
}

fn render_file(
    ui: &mut egui::Ui, file_info: &mut FileInfo,
    server: &AsmServer, content: &mut Content,
) {
    let FileInfo { title, file_key, level } = file_info;
    ui.horizontal(|ui| {
        ui.add_space((*level as f32) * 12.0);
        let layout_job = layout_string(ui, title.to_string());
        let label = ui.selectable_label(false, layout_job);
        if label.clicked() {
            let existed_tab = content.opened_tabs.iter().position(|tab| tab.file_key == *file_key);
            if let Some(existed_tab) = existed_tab {
                content.current = Some(existed_tab);
                return;
            }
            let smali = server.read_content(file_key);
            if let Some(smali) = smali {
                let current_tab = Tab {
                    selected: false,
                    file_key: Arc::clone(file_key),
                    title: Arc::clone(title),
                    content: smali,
                };
                let current = content.opened_tabs.len();
                content.opened_tabs.push(current_tab);
                content.current = Some(current);
            }
        }
    });
}

fn render_dir_raw(
    ui: &mut egui::Ui, dir_info: &mut RawDirInfo,
) {
    let RawDirInfo { opened, level, title } = dir_info;
    ui.horizontal(|ui| {
        ui.add_space((*level as f32) * 12.0);
        let font = TextStyle::Body.resolve(ui.style());
        let color = ui.style().visuals.text_color();
        let title = if *opened {
            format!("📂 {}", title)
        } else {
            format!("📁 {}", title)
        };
        let layout_job = LayoutJob::simple_singleline(title, font.clone(), color);
        let label = ui.selectable_label(false, layout_job);
        if label.clicked() {
            *opened = !*opened;
        }
    });
}

fn layout_string(
    ui: &mut egui::Ui, string: String,
) -> LayoutJob {
    let font = TextStyle::Body.resolve(ui.style());
    let color = ui.style().visuals.text_color();
    LayoutJob::simple_singleline(string, font.clone(), color)
}
