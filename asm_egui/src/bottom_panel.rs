use crate::app::EguiApp;
use egui::{ScrollArea, Ui};
use java_asm_server::ui::ToastKind;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum BottomWindow {
    Notifications,
    Log,
}

impl EguiApp {
    pub(crate) fn bottom_panel(&mut self, ui: &mut Ui) {
        egui::Panel::bottom("bottom_panel")
            .resizable(true)
            .min_size(0.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    Self::bottom_window_button(
                        ui, &mut self.bottom_window,
                        BottomWindow::Notifications, "Notifications",
                    );
                    Self::bottom_window_button(
                        ui, &mut self.bottom_window,
                        BottomWindow::Log, "Log / 日志",
                    );
                });

                let Some(bottom_window) = self.bottom_window else { return; };
                ui.separator();
                match bottom_window {
                    BottomWindow::Notifications => self.notifications_window(ui),
                    BottomWindow::Log => self.log_window(ui),
                }
            });
    }

    fn bottom_window_button(
        ui: &mut Ui, selected_window: &mut Option<BottomWindow>,
        window: BottomWindow, label: &str,
    ) {
        let selected = *selected_window == Some(window);
        if ui.selectable_label(selected, label).clicked() {
            *selected_window = (!selected).then_some(window);
        }
    }

    fn notifications_window(&self, ui: &mut Ui) {
        let toasts = self.ui_app.toasts().lock();
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                if toasts.is_empty() {
                    ui.weak("No notifications");
                    return;
                }
                for toast in toasts.iter().rev() {
                    let (prefix, color) = match toast.kind {
                        ToastKind::Success => ("✓", ui.visuals().text_color()),
                        ToastKind::Error => {
                            let color = if ui.visuals().dark_mode {
                                egui::Color32::from_rgb(255, 180, 180)
                            } else {
                                egui::Color32::from_rgb(150, 35, 35)
                            };
                            ("!", color)
                        }
                    };
                    ui.colored_label(color, format!("{prefix} {}", toast.message));
                }
            });
    }

    fn log_window(&self, ui: &mut Ui) {
        let current_records = self.log_holder.records.lock();
        let log_text = current_records
            .iter()
            .map(|log| format!("[{}] {}: {}", log.timestamp, log.level, log.message))
            .collect::<Vec<_>>()
            .join("\n");
        drop(current_records);

        if ui
            .add_enabled(!log_text.is_empty(), egui::Button::new("📋 Copy All"))
            .clicked()
        {
            ui.ctx().copy_text(log_text.clone());
        }
        ui.separator();

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                if log_text.is_empty() {
                    ui.weak("No log records");
                    return;
                }

                ui.add(egui::Label::new(log_text.as_str()).selectable(true));
            });
    }
}
