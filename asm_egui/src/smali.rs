use eframe::epaint::Color32;
use egui::text::LayoutJob;
use egui::{FontId, Response, ScrollArea, TextStyle, Ui};
use java_asm::smali::{SmaliNode, SmaliToken};

pub fn smali_layout(ui: &mut Ui, smali_node: &SmaliNode) {
    let ctx = &mut ui.ctx();

    let style = ui.ctx().style();
    let font = TextStyle::Monospace.resolve(&style);
    let dft_color = style.visuals.text_color();
    let dark_mode = style.visuals.dark_mode;
    let smali_style = if dark_mode { SmaliStyle::DARK } else { SmaliStyle::LIGHT };

    let lines = smali_node.render_to_lines();
    let row_height = font.size + ui.spacing().item_spacing.y;
    ScrollArea::vertical().auto_shrink(false).show_rows(ui, row_height, lines.len(), |ui, range| {
        for i in range {
            let line = &lines[i];
            render_line(ui, &font, &smali_style, dft_color, line);
        }
    });
}

fn render_line(
    ui: &mut Ui, font: &FontId, smali_style: &SmaliStyle, dft_color: Color32, line: &[SmaliToken],
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        for token_item in line {
            token(ui, font, smali_style, dft_color, token_item);
        }
    });
}

fn token(
    ui: &mut Ui, font: &FontId, smali_style: &SmaliStyle,
    dft_color: Color32, token: &SmaliToken,
) -> Response {
    match token {
        SmaliToken::Raw(s) => simple_text(ui, s.to_string(), font, dft_color),
        SmaliToken::Op(s) => simple_text(ui, s.to_string(), font, smali_style.op),
        SmaliToken::Offset { relative, absolute } => {
            let text = format!("@{absolute}({relative:+})");
            simple_text(ui, text, font, smali_style.offset)
        }
        SmaliToken::Register(s) => simple_text(ui, format!("v{s}"), font, smali_style.register),
        SmaliToken::RegisterRange(start, end) => {
            let text = format!("v{start}..v{end}");
            simple_text(ui, text, font, smali_style.register)
        }
        SmaliToken::Descriptor(s) => {
            let text_ui = simple_text(ui, s.to_string(), font, smali_style.desc)
                .on_hover_ui(|ui| {
                    ui.style_mut().interaction.selectable_labels = true;
                    ui.horizontal(|ui| {
                        ui.label(format!("1descriptor: {s}"));
                        ui.label(format!("2descriptor: {s}"));
                    });
                });
            text_ui.context_menu(|ui| {
                ui.label(format!("descriptor: {s}"));
            });
            text_ui
        },
        SmaliToken::Literal(s) => simple_text(ui, s.to_string(), font, smali_style.literal),
        SmaliToken::Other(s) => simple_text(ui, s.to_string(), font, dft_color),
    }
}

fn simple_text(
    ui: &mut Ui, text: String, font: &FontId, color: Color32,
) -> Response {
    ui.label(LayoutJob::simple_singleline(text, font.clone(), color))
}

#[derive(Copy, Clone, Debug)]
struct SmaliStyle {
    op: Color32,
    offset: Color32,
    register: Color32,
    desc: Color32,
    literal: Color32,
}

impl SmaliStyle {
    const LIGHT: SmaliStyle = SmaliStyle {
        op: Color32::from_rgb(235, 0, 0),
        offset: Color32::from_rgb(96, 96, 96),
        register: Color32::from_rgb(83, 141, 199),
        desc: Color32::from_rgb(153, 134, 255),
        literal: Color32::from_rgb(37, 203, 105),
    };

    const DARK: SmaliStyle = SmaliStyle {
        op: Color32::from_rgb(255, 100, 100),
        offset: SmaliStyle::LIGHT.offset,
        register: SmaliStyle::LIGHT.register,
        desc: SmaliStyle::LIGHT.desc,
        literal: SmaliStyle::LIGHT.literal,
    };
}
