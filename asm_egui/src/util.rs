use egui::{Id, Ui};

// content: (ui, target_width_for_content)
pub fn fill_width_in_parent(
    ui: &mut Ui, content_id_for_fill: Id, content: impl FnOnce(&mut Ui, f32),
) {
    let before_ui_available = ui.available_width();
    let last_time_allocated = ui
        .data(|data| data.get_temp(content_id_for_fill)
            .unwrap_or(before_ui_available));
    let target_width_for_content = before_ui_available - last_time_allocated;
    content(ui, target_width_for_content);

    let after_ui_available = ui.available_width();
    ui.data_mut(|data| {
        let allocated = before_ui_available - after_ui_available;
        data.insert_temp(content_id_for_fill, allocated);
    })
}

pub fn available_width_to_fill(ui: &mut Ui, content_id_for_fill: Id) -> f32 {
    let before_ui_available = ui.available_width();
    let last_time_allocated = ui
        .data(|data| data.get_temp(content_id_for_fill)
            .unwrap_or(before_ui_available));
    before_ui_available - last_time_allocated
}

pub fn record_remain_width_after_rendered(ui: &mut Ui, content_id_for_fill: Id) {
    let before_ui_available = ui.available_width();
    let after_ui_available = ui.available_width();
    let allocated = before_ui_available - after_ui_available;
    ui.data_mut(|data| {
        data.insert_temp(content_id_for_fill, allocated);
    })
}
