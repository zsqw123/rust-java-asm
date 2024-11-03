use egui::CollapsingHeader;
use java_asm_server::ui::{DirInfo, FileInfo};

pub fn render_dir(ui: &mut egui::Ui, file_tree: &DirInfo) {
    CollapsingHeader::new(&*file_tree.title)
        .default_open(file_tree.opened)
        .show(ui, |ui| {
            for (_, child) in &file_tree.dirs {
                render_dir(ui, child);
            }
            for (_, file) in &file_tree.files {
                render_file(ui, file);
            }
        });
}

pub fn render_file(ui: &mut egui::Ui, file: &FileInfo) {
    ui.label(&*file.title);
}
