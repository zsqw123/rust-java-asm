use crate::file_tree::render_dir;
use crate::font::inject_sys_font;
use crate::smali::smali_layout;
use eframe::{CreationContext, Frame};
use egui::{Context, DroppedFile, ScrollArea};
use egui_extras::{Size, StripBuilder};
use java_asm_server::ui::log::{inject_log, LogHolder};
use java_asm_server::ui::App;
use java_asm_server::AsmServer;
use std::sync::Arc;

#[derive(Default)]
pub struct EguiApp {
    pub server: Option<AsmServer>,
    pub log_holder: Arc<LogHolder>,
    pub server_app: App,
}

impl EguiApp {
    pub fn new(context: &CreationContext) -> Self {
        let log_holder = Default::default();
        inject_log(Arc::clone(&log_holder));
        inject_sys_font(context);
        Self { log_holder, ..Default::default() }
    }
}

impl EguiApp {
    fn top_bar(&mut self, ctx: &Context) {
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                if ui.button("📂 Open...").clicked() {}
            });
        });
    }

    fn left_bar(&mut self, ctx: &Context) {
        let available = ctx.available_rect().width();
        egui::SidePanel::left("left_bar")
            .resizable(true)
            .max_width(available * 0.75)
            .default_width(available * 0.25)
            .show(ctx, |ui| {
                StripBuilder::new(ui).size(Size::remainder()).horizontal(|mut strip| {
                    strip.cell(|ui| {
                        ScrollArea::horizontal().show(ui, |ui| {
                            ui.heading("File Tree");
                            render_dir(ui, self);
                        });
                    });
                });
            });
    }

    fn bottom_log_panel(&mut self, ctx: &Context) {
        egui::TopBottomPanel::bottom("bottom_log_panel").resizable(true)
            .show(ctx, |ui| {
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

    fn central_panel(&mut self, ctx: &Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let content = &mut self.server_app.content;
            let tabs = &content.opened_tabs;
            for tab in tabs {}

            let current_tab = &content.current;
            if let Some(current_tab) = current_tab {
                ui.heading(current_tab.title.to_string());
                ScrollArea::vertical().show(ui, |ui| {
                    smali_layout(ui, &current_tab.content);
                });
            }
        });
    }

    fn process_dropped_file(&mut self, ctx: &Context) {
        ctx.input(|input| {
            if let Some(DroppedFile { path, .. }) = input.raw.dropped_files.get(0) {
                if let Some(path) = path {
                    let path = path.display().to_string();
                    let server = AsmServer::smart_open(&path).ok();
                    if let Some(server) = server {
                        server.render_to_app(&mut self.server_app);
                        self.server = Some(server);
                    }
                }
            }
        })
    }
}

impl eframe::App for EguiApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        self.top_bar(ctx);
        self.bottom_log_panel(ctx);
        self.left_bar(ctx);
        self.central_panel(ctx);
        self.process_dropped_file(ctx);
    }
}


