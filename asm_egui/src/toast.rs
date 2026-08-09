use egui::Context;
use java_asm_server::ui::{AppContainer, ToastKind};
use java_asm_server::{Duration, Instant};

pub(crate) fn ui(ctx: &Context, app: &AppContainer) {
    let toast = app.toasts().lock().last().cloned();
    let Some(toast) = toast else { return; };
    let expires_at = toast.created_at + Duration::from_secs(4);
    let remaining = expires_at.saturating_duration_since(Instant::now());
    if remaining.is_zero() {
        return;
    }

    let kind = toast.kind;
    let message = toast.message;
    ctx.request_repaint_after(remaining);
    egui::Area::new(egui::Id::new("asm_toast"))
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-16.0, 16.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| match kind {
            ToastKind::Success => {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.label(message);
                });
            }
            ToastKind::Error => {
                let (fill, stroke, text) = if ui.visuals().dark_mode {
                    (
                        egui::Color32::from_rgb(90, 45, 45),
                        egui::Color32::from_rgb(180, 90, 90),
                        egui::Color32::from_rgb(255, 220, 220),
                    )
                } else {
                    (
                        egui::Color32::from_rgb(255, 235, 235),
                        egui::Color32::from_rgb(230, 150, 150),
                        egui::Color32::from_rgb(130, 35, 35),
                    )
                };
                egui::Frame::popup(ui.style())
                    .fill(fill)
                    .stroke(egui::Stroke::new(1.0, stroke))
                    .inner_margin(egui::Margin::same(8))
                    .show(ui, |ui| {
                        ui.colored_label(text, message);
                    });
            }
        });
}
