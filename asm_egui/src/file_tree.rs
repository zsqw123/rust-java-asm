use crate::app::EguiApp;
use egui::text::LayoutJob;
use egui::{ScrollArea, TextStyle};
use java_asm_server::ui::{AppContainer, FileEntry, FileInfo, OpenFileMessage, RawDirInfo, UIMessage};

pub fn render_dir(ui: &mut egui::Ui, app: &mut EguiApp) {
    let server_app = &app.ui_app;
    let mut left = server_app.left().lock();
    let entries = &mut left.root_node.visible_items();
    let server = app.server.lock();
    if server.is_none() { return; }
    let row_height = ui.spacing().interact_size.y;
    let scroll_area = ScrollArea::vertical().auto_shrink(false);
    // scroll_area.vertical_scroll_offset();
    scroll_area.show_rows(ui, row_height, entries.len(), |ui, range| {
        for i in range {
            let entry = &mut entries[i];
            match entry {
                FileEntry::Dir(raw_dir) => {
                    render_dir_raw(ui, raw_dir, server_app);
                }
                FileEntry::File(file_info) => {
                    render_file(ui, file_info, server_app);
                }
            }
        }
    });
}

fn render_file(
    ui: &mut egui::Ui, file_info: &mut FileInfo, app: &AppContainer,
) {
    let FileInfo { title, file_key, level } = file_info;
    ui.horizontal(|ui| {
        ui.add_space((*level as f32) * 12.0);
        let layout_job = layout_string(ui, title.to_string());
        let label = ui.selectable_label(false, layout_job);
        if label.clicked() {
            let message = UIMessage::OpenFile(
                OpenFileMessage {
                    path: file_key.clone(),
                }
            );
            app.send_message(message);
        }
    });
}

fn render_dir_raw(
    ui: &mut egui::Ui, dir_info: &mut RawDirInfo, app_container: &AppContainer,
) {
    let RawDirInfo { opened, level, title, dir_key } = dir_info;
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
            let new_opened = !*opened;
            if new_opened {
                *opened = true;
            } else {
                // closed, close all child dir
                app_container.send_message(UIMessage::CloseDir(dir_info.dir_key.clone()))
            }
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
