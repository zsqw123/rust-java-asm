use egui::{Button, Ui, Widget};
use egui_flex::{item, Flex};
use java_asm_server::ui::Tab;

pub fn render_tabs(ui: &mut Ui, current: &mut Option<usize>, tabs: &mut Vec<Tab>, deleted_tab: &mut Option<usize>) {
    Flex::horizontal().show(ui, |ui| {
        for tab in tabs.iter_mut().enumerate() {
            ui.add_simple(item(), |ui: &mut Ui| {
                file_title(ui, current, deleted_tab, tab)
            });
        }
    });
}

fn file_title(ui: &mut Ui, current: &mut Option<usize>, deleted_tab: &mut Option<usize>, tab: (usize, &mut Tab)) {
    let (index, tab) = tab;
    ui.horizontal(|ui| {
        let selected = current
            .map(|current| current == index).unwrap_or_default();
        if ui.selectable_label(selected, &*tab.title).clicked() {
            *current = Some(index);
        }
        let close_bt = Button::new("x").small().frame(false).ui(ui);
        if close_bt.clicked() {
            *deleted_tab = Some(index);
            // recalculate current tab
            if let Some(cur) = *current {
                if cur == index {
                    *current = None;
                } else if cur > index {
                    *current = Some(cur - 1);
                }
            }
        }
    });
}
