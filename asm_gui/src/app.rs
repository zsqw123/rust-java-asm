use crate::font::inject_sys_font;
use eframe::{CreationContext, Frame};
use egui::Context;
use java_asm_server::AsmServer;

#[derive(Default)]
pub struct AsmApp {
    pub current_path: Option<String>,
    pub server: Option<AsmServer>
}


impl AsmApp {
    pub fn new(context: &CreationContext) -> Self {
        inject_sys_font(context);
        Self { ..Default::default() }
    }
}

impl eframe::App for AsmApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("ASM GUI");
            ui.code("fn main() { ... }");
            ui.horizontal(|ui| {
                if let Some(path) = &mut self.current_path {
                    ui.label(format!("Current Path: {}", path));
                } else {
                    ui.label("Current Path: None");
                }
            });
            ui.horizontal(|ui| {
                if ui.button("Open").clicked() {
                    
                }
                if ui.button("Close").clicked() {
                    self.server = None;
                }
            });
        });
    }
}
