use crate::file_tab::render_tabs;
use crate::file_tree::render_dir;
use crate::font::inject_sys_font;
use crate::smali::smali_layout;
use eframe::{CreationContext, Frame};
use egui::{Context, DroppedFile, ScrollArea};
use egui_extras::{Size, StripBuilder};
use egui_notify::Toasts;
use java_asm_server::rw_access::ReadAccess;
use java_asm_server::ui::log::{inject_log, LogHolder};
use java_asm_server::ui::AppContainer;
use java_asm_server::{AsmServer, ServerMut};
use std::sync::Arc;

pub struct EguiApp {
    pub server: ServerMut,
    pub log_holder: Arc<LogHolder>,
    pub ui_app: AppContainer,
    pub toasts: Toasts,
}

impl EguiApp {
    pub fn new(context: &CreationContext) -> Self {
        let log_holder = Default::default();
        inject_log(Arc::clone(&log_holder));
        inject_sys_font(context);
        Self {
            log_holder,
            server: Default::default(),
            ui_app: Default::default(),
            toasts: Toasts::default(),
        }
    }
}

impl EguiApp {
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
            ui.collapsing("Log / 日志", |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    let current_records = self.log_holder.records.lock();
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
            let server_locked = self.server.lock();
            let Some(server) = server_locked.as_ref() else {
                return;
            };
            let server_app = &self.ui_app;

            render_tabs(ui, server_app);

            ui.separator();

            smali_layout(ui, server, &self.ui_app);
        });
    }
}

// action triggers
impl EguiApp {
    fn process_dropped_file(&mut self, ctx: &Context) {
        ctx.input(|input| {
            let Some(dropped_file) = input.raw.dropped_files.get(0) else {
                return;
            };
            let Some(read_access) = create_read_access(dropped_file.clone()) else {
                return;
            };
            AsmServer::smart_open(self.server.clone(), read_access, self.ui_app.clone());
        })
    }
}
fn create_read_access(dropped_file: DroppedFile) -> Option<ReadAccess> {
    if let Some(bytes) = dropped_file.bytes {
        return Some(ReadAccess::from_raw(dropped_file.name, bytes));
    };
    if let Some(path) = dropped_file.path {
        return Some(ReadAccess::from_path(&path));
    };
    None
}

impl eframe::App for EguiApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        let mut mutex = self.server.lock();
        if let Some(server) = mutex.as_mut() {
            self.ui_app.process_messages(server);
        }
        drop(mutex);
        self.top_bar(ctx);
        self.bottom_log_panel(ctx);
        self.left_bar(ctx);
        self.central_panel(ctx);
        self.process_dropped_file(ctx);
        self.toasts.show(ctx);
    }
}


