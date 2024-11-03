use crate::font::inject_sys_font;
use eframe::{CreationContext, Frame};
use egui::{Context, DroppedFile, ScrollArea};
use egui_extras::syntax_highlighting::{code_view_ui, CodeTheme};
use java_asm_server::ui::log::{inject_log, LogHolder};
use java_asm_server::AsmServer;
use std::sync::Arc;

#[derive(Default)]
pub struct AsmApp {
    pub current_path: Option<String>,
    pub server: Option<AsmServer>,
    pub log_holder: Arc<LogHolder>,
}

impl AsmApp {
    pub fn new(context: &CreationContext) -> Self {
        let log_holder = Default::default();
        inject_log(Arc::clone(&log_holder));
        inject_sys_font(context);
        Self { log_holder, ..Default::default() }
    }
}

impl AsmApp {
    fn top_bar(&mut self, ctx: &Context) {
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                if ui.button("📂 Open...").clicked() {}
            });
        });
    }

    fn bottom_log_panel(&mut self, ctx: &Context) {
        egui::TopBottomPanel::bottom("bottom_log_panel").show(ctx, |ui| {
            ui.collapsing("Log 输出", |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    let current_records = self.log_holder.records.lock().unwrap();
                    let current_records = current_records.iter();
                    for log in current_records {
                        ui.label(format!("{}: {}", log.level, log.message));
                    }
                });
            });
        });
    }
}

impl eframe::App for AsmApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        self.top_bar(ctx);
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("ASM GUI");
            code_view_ui(ui, &CodeTheme::from_style(&ctx.style()), "fn main() { ... }", "rust");
            ui.horizontal(|ui| {
                if let Some(path) = &mut self.current_path {
                    ui.label(format!("Current Path: {}", path));
                } else {
                    ui.label("Current Path: None");
                }
            });
            ui.horizontal(|ui| {
                if ui.button("Open File").clicked() {
                    
                }
            });
            ctx.input(|input| {
                if let Some(DroppedFile { path, .. }) = input.raw.dropped_files.get(0) {
                    if let Some(path) = path {
                        self.current_path = Some(path.display().to_string());
                    }
                }
            })
        });
        self.bottom_log_panel(ctx);
    }
}


