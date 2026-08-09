use crate::file_tab::render_tabs;
use crate::file_tree::render_dir;
use crate::smali::smali_layout;
use eframe::{CreationContext, Frame};
use egui::{Context, ScrollArea, Ui};
use egui_extras::{Size, StripBuilder};
use java_asm_server::ui::AppContainer;
use java_asm_server::ui::log::{LogHolder, inject_log};
use java_asm_server::{Duration, Instant, ServerMut};
use std::sync::Arc;

struct Toast {
    message: String,
    expires_at: Instant,
}

pub struct EguiApp {
    pub server: ServerMut,
    pub log_holder: Arc<LogHolder>,
    pub ui_app: AppContainer,
    toast: Option<Toast>,
}

impl EguiApp {
    pub fn new(context: &CreationContext) -> Self {
        let log_holder = Default::default();
        inject_log(Arc::clone(&log_holder));
        crate::targets::configure_fonts(context);
        Self {
            log_holder,
            server: Default::default(),
            ui_app: Default::default(),
            toast: None,
        }
    }
}

impl EguiApp {
    pub(crate) fn notify_success(&mut self, message: impl Into<String>) {
        self.toast = Some(Toast {
            message: message.into(),
            expires_at: Instant::now() + Duration::from_secs(3),
        });
    }

    fn show_toast(&mut self, ctx: &Context) {
        let Some(toast) = self.toast.as_ref() else {
            return;
        };
        let remaining = toast.expires_at.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            self.toast = None;
            return;
        }
        let message = toast.message.clone();
        ctx.request_repaint_after(remaining);
        egui::Area::new(egui::Id::new("asm_toast"))
            .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-16.0, 16.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.label(message);
                });
            });
    }
}

impl EguiApp {
    fn left_bar(&mut self, ui: &mut Ui) {
        let available = ui.available_width();
        egui::Panel::left("left_bar")
            .resizable(true)
            .max_size(available * 0.75)
            .default_size(available * 0.25)
            .show(ui, |ui| {
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

    fn bottom_log_panel(&mut self, ui: &mut Ui) {
        egui::Panel::bottom("bottom_log_panel").resizable(true)
            .show(ui, |ui| {
            ui.collapsing("Log / 日志", |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    let current_records = self.log_holder.records.lock();
                    let current_records = current_records.iter();
                    for log in current_records {
                        ui.label(format!(
                            "[{}] {}: {}",
                            log.timestamp,
                            log.level,
                            log.message,
                        ));
                    }
                });
            });
        });
    }

    fn central_panel(&mut self, ui: &mut Ui) {
        egui::CentralPanel::default().show(ui, |ui| {
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

    fn handle_events(&mut self, ctx: &Context) {
        let mut mutex = self.server.lock();
        let Some(server) = mutex.as_mut() else {
            return;
        };

        // 1. process server messages
        self.ui_app.process_messages(server);
        // 2. process loading state
        let in_loading = server.loading_state.in_loading;
        if in_loading { // Keep the progress bar responsive.
            ctx.request_repaint_after(Duration::from_millis(150));
        }
    }
}

// action triggers
impl EguiApp {
    fn process_dropped_file(&mut self, ctx: &Context) {
        ctx.input(|input| {
            let Some(dropped_file) = input.raw.dropped_files.first().cloned() else {
                return;
            };
            crate::targets::open_dropped_file(dropped_file, self.server.clone(), self.ui_app.clone());
        })
    }
}

impl eframe::App for EguiApp {
    fn logic(&mut self, ctx: &Context, _frame: &mut Frame) {
        self.handle_events(ctx);
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut Frame) {
        self.top_bar(ui);
        self.bottom_log_panel(ui);
        self.left_bar(ui);
        self.central_panel(ui);
        self.process_dropped_file(ui.ctx());
        self.show_toast(ui.ctx());
    }
}
