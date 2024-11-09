use egui::text::LayoutJob;
use egui::util::cache::{ComputerMut, FrameCache};
use egui::{Color32, FontId, TextFormat, TextStyle, Ui};
use java_asm::smali::{SmaliNode, SmaliToken};

#[derive(Default)]
struct SmaliHighlighter;

pub fn smali_layout(ui: &mut Ui, smali_node: &SmaliNode) {
    let ctx = &mut ui.ctx();

    // font, dft_color, dark_mode, smali_node
    impl ComputerMut<(&FontId, Color32, bool, &SmaliNode), LayoutJob> for SmaliHighlighter {
        fn compute(&mut self, key: (&FontId, Color32, bool, &SmaliNode)) -> LayoutJob {
            let (font, dft_color, dark_mode, smali_node) = key;
            let mut job = LayoutJob::default();
            let smali_style = if dark_mode { SmaliStyle::DARK } else { SmaliStyle::LIGHT };
            let max_offset_len = max_offset_hint(smali_node).to_string().len();
            append_node(&font, dft_color, &smali_style, smali_node, &mut job, 0, max_offset_len);
            job
        }
    }

    type HighlightCache = FrameCache<LayoutJob, SmaliHighlighter>;

    let style = ui.ctx().style();
    let font = TextStyle::Monospace.resolve(&style);
    let dft_color = style.visuals.text_color();
    let dark_mode = style.visuals.dark_mode;
    let job = ctx.memory_mut(|mem| {
        mem.caches.cache::<HighlightCache>()
            .get( (&font, dft_color, dark_mode, smali_node))
    });
    ui.label(job);
}

fn append_node(
    font: &FontId, dft_color: Color32, smali_style: &SmaliStyle, node: &SmaliNode,
    job: &mut LayoutJob, indent: usize, max_offset_len: usize,
) {
    let SmaliNode { tag, content, children, end_tag, .. } = node;
    append_offset_or_stub(max_offset_len, node, job, font, smali_style);
    append_indent(job, font, smali_style, indent);
    if let Some(tag) = tag {
        append(job, tag, font, dft_color);
        append_space(job, font, smali_style);
    }
    for token in content {
        append_token(font, dft_color, smali_style, token, job);
        append_space(job, font, smali_style);
    }
    for child in children {
        append(job, "\n", font, dft_color);
        append_node(font, dft_color, smali_style, child, job, indent + 1, max_offset_len);
    }
    if children.len() > 0 {
        append(job, "\n", font, dft_color);
    }
    if let Some(end_tag) = end_tag {
        append_indent(job, font, smali_style, indent);
        append(job, end_tag, font, dft_color);
        append(job, "\n", font, dft_color);
    }
}

#[inline]
fn max_offset_hint(smali_node: &SmaliNode) -> u32 {
    let mut max = 0;
    for child in &smali_node.children {
        max = max.max(max_offset_hint(child));
    }
    if let Some(offset_hint) = smali_node.offset_hint {
        max = max.max(offset_hint);
    }
    max
}

fn append_offset_or_stub(
    max_offset_len: usize, smali_node: &SmaliNode, job: &mut LayoutJob,
    font: &FontId, smali_style: &SmaliStyle,
) {
    if let Some(offset_hint) = smali_node.offset_hint {
        let offset_str = format!("{:width$}", offset_hint, width = max_offset_len);
        append(job, &offset_str, font, smali_style.offset);
    } else {
        append(job, &" ".repeat(max_offset_len), font, smali_style.offset);
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
            let text = format!("@{absolute}({relative:+})");
            append(job, &text, font, smali_style.offset);
        }
        SmaliToken::Register(s) => append(job, &format!("v{s}"), font, smali_style.register),
        SmaliToken::RegisterRange(start, end) => {
            let text = format!("v{start}..v{end}");
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
