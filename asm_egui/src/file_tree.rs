use crate::app::EguiApp;
use egui::CollapsingHeader;
use egui_extras::{Column, TableBuilder};
use java_asm_server::ui::{Content, DirInfo, FileInfo, Tab};
use java_asm_server::AsmServer;
use std::rc::Rc;

pub fn render_dir(ui: &mut egui::Ui, app: &mut EguiApp) {
    let root = &app.server_app.left.root_node;
    let server = &app.server;
    let content = &mut app.server_app.content;
    if let Some(server) = server {
        render_dir_row(ui, root, &mut 0, server, content);
    }
}

fn render_dir_row(
    ui: &mut egui::Ui, dir: &DirInfo, index: &mut usize,
    server: &AsmServer, content: &mut Content,
) {
    *index += 1;
    CollapsingHeader::new(&*dir.title)
        .id_salt(*index)
        .default_open(dir.opened)
        .show(ui, |ui| {
            for (_, child) in &dir.dirs {
                render_dir_row(ui, child, index, server, content);
            }
            render_files(ui, dir, server, content);
        });
}

fn render_files(ui: &mut egui::Ui, dir: &DirInfo, server: &AsmServer, content: &mut Content) {
    let text_height = egui::TextStyle::Body
        .resolve(ui.style()).size
        .max(ui.spacing().interact_size.y);

    TableBuilder::new(ui).striped(true).resizable(true)
        .vscroll(false)
        .column(Column::auto().resizable(true))
        .body(|ui| {
            let mut file_keys = dir.files.iter().collect::<Vec<_>>();
            file_keys.sort_by_key(|(k, _)| Rc::clone(k));
            ui.rows(text_height, file_keys.len(), |mut row| {
                let index = row.index();
                let (_, file_info) = &file_keys[index];
                row.col(|ui| {
                    let FileInfo { selected, title, file_key } = file_info;
                    let label = ui.selectable_label(*selected, &**title);
                    if label.clicked() {
                        let smali = server.read_content(file_key);
                        if let Some(smali) = smali {
                            let current_tab = Tab {
                                selected: false,
                                file_key: Rc::clone(file_key),
                                title: Rc::clone(title),
                                content: smali,
                            };
                            let current_tab = Rc::new(current_tab);
                            content.current = Some(Rc::clone(&current_tab));
                        }
                    }
                });
            });
        });
}
