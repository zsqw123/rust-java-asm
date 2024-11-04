use egui::text::LayoutJob;
use egui::{Color32, FontId, TextFormat, TextStyle, Ui};
use java_asm::smali::{SmaliNode, SmaliToken};

pub fn smali_layout(ui: &mut Ui, smali_node: &SmaliNode) {
    let mut job = LayoutJob::default();
    let style = ui.ctx().style();
    let default_font = TextStyle::Monospace.resolve(&style);
    let normal_color = style.visuals.text_color();
    let smali_style = if ui.ctx().style().visuals.dark_mode {
        SmaliStyle::DARK
    } else {
        SmaliStyle::LIGHT
    };
    append_node(&default_font, normal_color, &smali_style, smali_node, &mut job, 0);
    ui.label(job);
}

fn append_node(
    font: &FontId, dft_color: Color32, smali_style: &SmaliStyle, node: &SmaliNode,
    job: &mut LayoutJob, indent: usize,
) {
    let SmaliNode { tag, content, offset_hint, children, end_tag } = node;
    if let Some(offset_hint) = offset_hint {
        append(job, &offset_hint.to_string(), font, smali_style.offset);
    }
    append_indent(job, font, smali_style, indent);
    if let Some(tag) = tag {
        append(job, tag, font, dft_color);
        append_space(job, font, smali_style);
    }
    for token in content {
        append_token(font, dft_color, smali_style, token, job);
        append_space(job, font, smali_style);
    }
    if let Some(end_tag) = end_tag {
        append(job, end_tag, font, dft_color);
    }
    for child in children {
        append(job, "\n", font, dft_color);
        append_node(font, dft_color, smali_style, child, job, indent + 1);
    }
}

fn append_token(
    font: &FontId, dft_color: Color32, smali_style: &SmaliStyle, token: &SmaliToken,
    job: &mut LayoutJob,
) {
    match token {
        SmaliToken::Raw(s) => append(job, s, font, dft_color),
        SmaliToken::Op(s) => append(job, s, font, smali_style.op),
        SmaliToken::Offset { relative, absolute } => {
            let text = format!("{absolute} ({relative:+})");
            append(job, &text, font, smali_style.offset);
        }
        SmaliToken::Register(s) => append(job, &s.to_string(), font, smali_style.register),
        SmaliToken::RegisterRange(start, end) => {
            let text = format!("{start}..{end}");
            append(job, &text, font, smali_style.register);
        }
        SmaliToken::Descriptor(s) => append(job, s, font, smali_style.desc),
        SmaliToken::Literal(s) => append(job, s, font, smali_style.literal),
        SmaliToken::Other(s) => append(job, s, font, dft_color),
    }
}

#[inline]
fn append_indent(job: &mut LayoutJob, font: &FontId, smali_style: &SmaliStyle, indent: usize) {
    let str = "  ".repeat(indent);
    append(job, &str, font, smali_style.offset);
}

#[inline]
fn append_space(job: &mut LayoutJob, font: &FontId, smali_style: &SmaliStyle) {
    append(job, " ", font, smali_style.offset);
}

#[inline]
fn append(job: &mut LayoutJob, text: &str, font: &FontId, color: Color32) {
    job.append(text, 0.0, TextFormat::simple(font.clone(), color));
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
