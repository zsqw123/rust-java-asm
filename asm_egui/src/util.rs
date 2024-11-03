use egui::{RichText, WidgetText};
use java_asm::StrRef;

pub fn widget_text(str_ref: StrRef) -> WidgetText {
    WidgetText::RichText(RichText::new(&*str_ref))
}
