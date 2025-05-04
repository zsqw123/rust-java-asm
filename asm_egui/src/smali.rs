use eframe::epaint::Color32;
use egui::text::LayoutJob;
use egui::{FontId, Response, ScrollArea, TextStyle, Ui};
use java_asm::smali::SmaliToken;
use java_asm_server::ui::{AppContainer, Content};
use java_asm_server::AsmServer;
use std::ops::{Deref, DerefMut};

pub fn smali_layout(ui: &mut Ui, server: &AsmServer, app: &AppContainer) {
    let mut content_locked = app.content().lock();
    let selected_tab_index = content_locked.selected;
    let Some(selected_tab_index) = selected_tab_index else { return; };

    let opened_tabs = &content_locked.opened_tabs;
    let selected_tab = opened_tabs.get(selected_tab_index);
    let Some(selected_tab) = selected_tab else { return; };
    let smali_node = &selected_tab.content;

    let style = ui.ctx().style();
    let font = TextStyle::Monospace.resolve(&style);
    let dft_color = style.visuals.text_color();
    let dark_mode = style.visuals.dark_mode;
    let smali_style = if dark_mode { SmaliStyle::DARK } else { SmaliStyle::LIGHT };

    let lines = smali_node.render_to_lines();
    let row_height = font.size;

    let content_mut = content_locked.deref_mut();
    ScrollArea::vertical().auto_shrink(false).show_rows(ui, row_height, lines.len(), |ui, range| {
        for i in range {
            let line = &lines[i];
            render_line(ui, server, content_mut, line, &font, &smali_style, dft_color);
        }
    });
}

fn render_line(
    ui: &mut Ui, server: &AsmServer, content: &mut Content, line: &[SmaliToken],
    font: &FontId, smali_style: &SmaliStyle, dft_color: Color32,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        for token_item in line {
            token(ui, server, content, token_item, font, smali_style, dft_color);
        }
    });
}

fn token(
    ui: &mut Ui, server: &AsmServer, content: &mut Content, token: &SmaliToken,
    font: &FontId, smali_style: &SmaliStyle, dft_color: Color32,
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
                    descriptor_menu(ui, s, server, content);
                });
            text_ui.context_menu(|ui| {
                descriptor_menu(ui, s, server, content);
            });
            text_ui
        },
        SmaliToken::Literal(s) => simple_text(ui, s.to_string(), font, smali_style.literal),
        SmaliToken::Other(s) => simple_text(ui, s.to_string(), font, dft_color),
    }
}

fn descriptor_menu(
    ui: &mut Ui, descriptor: &str,
    server: &AsmServer, content: &mut Content,
) {
    ui.vertical(|ui| {
        if descriptor.starts_with('(') {
            descriptor_menu_for_fn(ui, descriptor, server, content);
        } else {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label("type: ");
                render_single_descriptor(ui, descriptor, server, content);
            });
        }
    });
}

// function descriptors, e.g. (Ljava/lang/String;I)V, show ui like:
// arg1: Ljava/lang/String;
// arg2: I
// returned: V
fn descriptor_menu_for_fn(
    ui: &mut Ui, descriptor: &str,
    server: &AsmServer, content: &mut Content,
) -> Option<()> {
    let descriptor = descriptor.strip_prefix('(')?;
    let mut split = descriptor.split(')');

    // Vec<(typeDescriptor, arrayLevel)>
    let mut args: Vec<(String, usize)> = vec![];
    let args_part: Vec<char> = split.next()?.to_string().chars().collect();
    let mut i = 0usize;
    let mut array_level = 0usize;
    while let Some(arg) = args_part.get(i) {
        if *arg == 'L' {
            let next_index = i + 1;
            let end_index = args_part[next_index..].iter()
                .position(|c| *c == ';')? + next_index;
            let arg = &args_part[i..end_index + 1];
            args.push((arg.iter().collect(), array_level));
            array_level = 0;
            i = end_index + 1;
        } else if *arg == '[' {
            array_level += 1;
            i += 1;
            continue;
        } else {
            args.push((arg.to_string(), array_level));
            array_level = 0;
            i += 1;
        }
    }
    let returned = split.next()?;
    let returned_array_level = returned.chars().filter(|c| *c == '[').count();

    ui.vertical(|ui| {
        for (arg_index, (arg, array_level)) in args.iter().enumerate() {
            let array_level = *array_level;
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                let text = if array_level > 0usize {
                    format!("arg{arg_index}: ") + &"[".repeat(array_level)
                } else {
                    format!("arg{arg_index}: ")
                };
                ui.label(text);
                render_single_descriptor(ui, arg, server, content);
            });
        }
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            let text = if returned_array_level > 0 {
                "returned: ".to_string() + &"[".repeat(returned_array_level)
            } else {
                "returned: ".to_string()
            };
            ui.label(text);
            render_single_descriptor(ui, returned, server, content);
        })
    });
    None
}

fn render_single_descriptor(
    ui: &mut Ui, descriptor: &str,
    server: &AsmServer, content: &mut Content,
) {
    let existed = server.find_class(descriptor);
    if !existed {
        ui.label(format!("{descriptor}"));
    } else {
        let accessor_locked = server.accessor.lock();
        let accessor = accessor_locked.deref();
        let Some(accessor) = accessor else { return };
        let link = ui.link(descriptor);
        if link.clicked() {
            server.switch_or_open_lock_free(descriptor, accessor, content)
        }
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
    highlight: Color32,
}

impl SmaliStyle {
    const LIGHT: SmaliStyle = SmaliStyle {
        op: Color32::from_rgb(235, 0, 0),
        offset: Color32::from_rgb(96, 96, 96),
        register: Color32::from_rgb(83, 141, 199),
        desc: Color32::from_rgb(153, 134, 255),
        literal: Color32::from_rgb(37, 203, 105),
        highlight: Color32::from_rgb(255, 199, 133),
    };

    const DARK: SmaliStyle = SmaliStyle {
        op: Color32::from_rgb(255, 100, 100),
        offset: SmaliStyle::LIGHT.offset,
        register: SmaliStyle::LIGHT.register,
        desc: SmaliStyle::LIGHT.desc,
        literal: SmaliStyle::LIGHT.literal,
        highlight: SmaliStyle::LIGHT.highlight,
    };
}
