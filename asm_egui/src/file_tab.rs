use eframe::emath::Align;
use eframe::epaint::StrokeKind;
use egui::{Layout, Pos2, Rect, Response, Sense, TextStyle, Ui, Vec2, WidgetInfo, WidgetText, WidgetType};
use egui_flex::{item, Flex, FlexAlignContent};
use java_asm::StrRef;
use java_asm_server::ui::Tab;
use std::rc::Rc;

pub fn render_tabs(ui: &mut Ui, current: &mut Option<usize>, tabs: &mut Vec<Tab>, deleted_tab: &mut Option<usize>) {
    Flex::horizontal()
        .align_content(FlexAlignContent::Start)
        .w_full()
        .wrap(true)
        .show(ui, |flex| {
        for tab in tabs.iter_mut().enumerate() {
            flex.add_ui(item(), |ui: &mut Ui| {
                file_title(ui, current, deleted_tab, tab)
            });
        }
    });
}

fn file_title(ui: &mut Ui, current: &mut Option<usize>, deleted_tab: &mut Option<usize>, tab: (usize, &mut Tab)) {
    let (index, tab) = tab;
    let selected = current.map(|current| current == index).unwrap_or_default();
    let title = Rc::clone(&tab.title);
    let selectable_label = SelectableClosableLabel { selected, title };
    let response = selectable_label.ui(ui);
    if response.closed {
        *deleted_tab = Some(index);
        // recalculate current tab
        if let Some(cur) = *current {
            if cur == index {
                *current = None;
            } else if cur > index {
                *current = Some(cur - 1);
            }
        }
    } else if response.raw.clicked() {
        *current = Some(index);
    }
}

struct SelectableClosableLabel {
    pub selected: bool,
    pub title: StrRef,
}

struct SelectableClosableLabelResponse {
    pub raw: Response,
    pub closed: bool,
}

impl SelectableClosableLabel {
    pub fn ui(self, ui: &mut Ui) -> SelectableClosableLabelResponse {
        let Self { selected, title } = self;

        let text_style = TextStyle::Button;
        let text_height = text_style.resolve(ui.style()).size;
        let padding = ui.spacing().button_padding;
        let padding_x = padding.x;
        let padding_y = padding.y;

        // [ title x ]
        let close_btn_vec2 = Vec2::splat(text_height * 0.8);

        let content_max_width = ui.available_width() - padding_x - padding_x;
        let text_wrap_width = content_max_width - close_btn_vec2.x - padding.x;
        let galley = WidgetText::from(&*title).into_galley(ui, None, text_wrap_width, TextStyle::Button);

        let mut target_size = Vec2::new(
            padding_x + galley.size().x + padding_x + close_btn_vec2.x + padding_x,
            padding_y + galley.size().y.max(close_btn_vec2.y) + padding_y,
        );
        target_size.y = target_size.y.max(ui.spacing().interact_size.y);
        let (rect, response) = ui.allocate_at_least(target_size, Sense::click());

        // accessibility
        response.widget_info(|| {
            WidgetInfo::selected(
                WidgetType::SelectableLabel, ui.is_enabled(), selected, galley.text(),
            )
        });

        if !ui.is_rect_visible(rect) {
            return SelectableClosableLabelResponse { raw: response, closed: false };
        }

        let visuals = ui.style().interact_selectable(&response, selected);

        let text_start = Layout::left_to_right(Align::Center).with_main_align(Align::Min)
            .align_size_within_rect(galley.size(), rect.shrink2(padding)).min;
        if selected || response.hovered() {
            let hover_rect = rect.expand(visuals.expansion);
            ui.painter().rect(
                hover_rect, visuals.corner_radius, visuals.weak_bg_fill,
                visuals.bg_stroke, StrokeKind::Middle
            );
        }

        let close_btn_start = Pos2::new(
            text_start.x + galley.size().x + padding_x,
            text_start.y + (galley.size().y - close_btn_vec2.y) / 2.0,
        );

        ui.painter().galley(text_start, galley, visuals.text_color());

        let close_btn_rec = Rect::from_min_size(close_btn_start, close_btn_vec2);
        let close_btn_response = Self::close_btn(ui, close_btn_rec);

        if close_btn_response.clicked() {
            return SelectableClosableLabelResponse { raw: response, closed: true };
        }

        SelectableClosableLabelResponse { raw: response, closed: false }
    }

    fn close_btn(ui: &mut Ui, rect: Rect) -> Response {
        let close_id = ui.auto_id_with("asm_selectable_close_btn");
        let response = ui.interact(rect, close_id, Sense::click());
        ui.expand_to_include_rect(response.rect);

        let visuals = ui.style().interact(&response);
        let stroke = visuals.fg_stroke;
        ui.painter() // paints \
            .line_segment([response.rect.left_top(), response.rect.right_bottom()], stroke);
        ui.painter() // paints /
            .line_segment([response.rect.right_top(), response.rect.left_bottom()], stroke);
        response
    }
}

