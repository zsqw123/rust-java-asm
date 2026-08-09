use eframe::epaint::Color32;
use egui::text::{LayoutJob, TextFormat};
use egui::containers::{Popup, PopupCloseBehavior, PopupKind};
use egui::{Align, Button, FontId, Id, Key, Modifiers, Response, ScrollArea, SetOpenCommand, TextEdit, TextStyle, Ui, Vec2};
use java_asm::smali::SmaliToken;
use java_asm::StrRef;
use java_asm_server::ui::{AppContainer, FindState, OpenFileMessage, SmaliLine, SmaliLineToken, UIMessage};
use java_asm_server::AsmServer;

pub fn smali_layout(
    ui: &mut Ui, server: &AsmServer, app: &AppContainer,
) {
    let mut content_locked = app.content().lock();
    let selected_tab_index = content_locked.selected;
    let Some(selected_tab_index) = selected_tab_index else { return; };

    let Some(selected_tab) = content_locked.opened_tabs.get_mut(selected_tab_index) else { return; };

    let style = ui.style();
    let font = TextStyle::Monospace.resolve(&style);
    let dft_color = style.visuals.text_color();
    let dark_mode = style.visuals.dark_mode;
    let smali_style = if dark_mode { SmaliStyle::DARK } else { SmaliStyle::LIGHT };

    let lines = selected_tab.rendered_lines.clone();
    let reveal_line = find_toolbar(
        ui, &mut selected_tab.find, selected_tab.file_key.as_ref(), &lines,
    );
    let row_height = ui.text_style_height(&TextStyle::Monospace);
    let spacing_y = ui.spacing().item_spacing.y;

    let mut scroll_area = ScrollArea::vertical()
        .auto_shrink(false)
        .id_salt(("asm_smali_scroll", selected_tab.file_key.as_ref()))
        .vertical_scroll_offset(selected_tab.scroll_offset);
    if let Some(reveal_line) = reveal_line {
        let row_height_with_spacing = row_height + spacing_y;
        let viewport_height = ui.available_height();
        let content_height = (lines.len() as f32 * row_height_with_spacing - spacing_y).max(0.0);
        let max_offset = (content_height - viewport_height).max(0.0);
        let target_offset = reveal_line as f32 * row_height_with_spacing
            - (viewport_height - row_height) / 2.0;
        scroll_area = scroll_area.vertical_scroll_offset(target_offset.clamp(0.0, max_offset));
    }

    let mut render_context = RenderContext {
        app: &app,
        server,
        font: &font,
        lines: lines.as_ref(),
        find: &selected_tab.find,
        find_open: selected_tab.find.open,
        reveal_line,
        smali_style: &smali_style,
        dft_color,
        row_height,
        spacing_y,
    };
    let scroll_output = scroll_area.show_rows(ui, row_height, lines.len(), |ui, range| {
        for i in range {
            render_context.render_line(ui, i);
        }
    });
    selected_tab.scroll_offset = scroll_output.state.offset.y;
}

fn find_toolbar(
    ui: &mut Ui, find: &mut FindState, file_key: &str, lines: &[SmaliLine],
) -> Option<usize> {
    let focus_requested = ui.input(|input| {
        input.key_pressed(Key::F) && input.modifiers.command
    });
    let mut reveal_line = None;
    if focus_requested {
        find.open = true;
        find.update_matches(lines.iter().map(|line| line.text.as_str()));
        reveal_line = find.current_match().map(|matched| matched.line);
    }
    if !find.open {
        return None;
    }

    let find_id = Id::new(("asm_find_input", file_key));
    let mut query = find.query.clone();
    let mut case_sensitive = find.case_sensitive;
    egui::Frame::popup(ui.style()).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label("Find");
            let edit_response = TextEdit::singleline(&mut query)
                .id(find_id)
                .hint_text("Search in current file...")
                .desired_width(280.0)
                .show(ui)
                .response;

            let case_response = ui.checkbox(&mut case_sensitive, "Aa");
            let input_changed = edit_response.changed() || case_response.changed();
            if input_changed {
                find.query = query;
                find.case_sensitive = case_sensitive;
                find.update_matches(lines.iter().map(|line| line.text.as_str()));
                reveal_line = find.current_match().map(|matched| matched.line);
            }

            let previous_pressed = ui.input_mut(|input| {
                input.consume_key(Modifiers::SHIFT, Key::Enter)
            });
            let next_pressed = !previous_pressed && ui.input_mut(|input| {
                input.consume_key(Modifiers::NONE, Key::Enter)
            });
            if previous_pressed || ui.button("↑").clicked() {
                reveal_line = find.previous();
            } else if next_pressed || ui.button("↓").clicked() {
                reveal_line = find.next();
            }

            let count = if find.matches.is_empty() {
                if find.query.is_empty() { "0/0".to_string() } else { "No results".to_string() }
            } else {
                format!("{}/{}", find.current + 1, find.matches.len())
            };
            ui.label(count);

            if ui.add(Button::new("×")).clicked()
                || ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Escape))
            {
                find.open = false;
            }

            if focus_requested {
                edit_response.request_focus();
            }
        });
    });
    reveal_line
}

struct RenderContext<'a> {
    pub app: &'a AppContainer,
    pub server: &'a AsmServer,
    pub lines: &'a [SmaliLine],
    pub find: &'a FindState,
    pub find_open: bool,
    pub reveal_line: Option<usize>,

    pub font: &'a FontId,
    pub smali_style: &'a SmaliStyle,
    pub dft_color: Color32,
    pub row_height: f32,
    pub spacing_y: f32,
}

impl<'a> RenderContext<'a> {
    pub fn render_line(&mut self, ui: &mut Ui, line_index: usize) {
        let line = &self.lines[line_index];
        let line_response = ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            for token_item in &line.tokens {
                self.token(ui, token_item, line_index);
            }
        });
        if self.reveal_line == Some(line_index) {
            line_response.response.scroll_to_me(Some(Align::Center));
        }
    }

    fn scroll_lines(&self, ui: &mut Ui, line_delta: usize) {
        let row_height_with_spacing = self.row_height + self.spacing_y;
        let y_delta = line_delta as f32 * row_height_with_spacing;
        let delta = Vec2::new(0.0, -y_delta);
        ui.scroll_with_delta(delta)
    }

    fn scroll_to_offset(&self, ui: &mut Ui, current_line: usize, target_offset: u32) {
        let start = current_line;
        let mut i = current_line;
        loop {
            let current = i;
            i += 1;
            let Some(current_line) = self.lines.get(current) else { continue; };
            let Some(first_node) = current_line.tokens.first() else { continue; };
            let SmaliToken::LineStartOffsetMarker { offset: Some(current_offset), .. } = &first_node.token else { continue; };
            if *current_offset >= target_offset {
                self.scroll_lines(ui, current - start);
                break;
            }
        }
    }


    fn token(
        &mut self, ui: &mut Ui, rendered_token: &SmaliLineToken, line_index: usize,
    ) -> Response {
        let token = &rendered_token.token;
        let dft_color = self.dft_color;
        match token {
            SmaliToken::SourceInfo(source_info) => {
                let mut text_ui = self.styled_text(
                    ui, line_index, rendered_token, self.smali_style.literal,
                );
                let popup_id = text_ui.id.with("source_info_click_popup");
                let click_popup_open = Popup::is_id_open(&text_ui.ctx, popup_id);
                if !click_popup_open && !text_ui.clicked() {
                    text_ui = text_ui.on_hover_ui(|ui| {
                        ui.style_mut().interaction.selectable_labels = true;
                        self.source_file_info_menu(ui, source_info);
                    });
                }
                Popup::from_response(&text_ui)
                    .id(popup_id)
                    .kind(PopupKind::Tooltip)
                    .open_memory(text_ui.clicked().then_some(SetOpenCommand::Toggle))
                    .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
                    .show(|ui| {
                        ui.style_mut().interaction.selectable_labels = true;
                        self.source_file_info_menu(ui, source_info);
                    });
                text_ui.context_menu(|ui| {
                    self.source_file_info_menu(ui, source_info);
                });
                text_ui
            },
            SmaliToken::Raw(_) => self.styled_text(ui, line_index, rendered_token, dft_color),
            SmaliToken::Op(_) => self.styled_text(ui, line_index, rendered_token, self.smali_style.op),
            SmaliToken::LineStartOffsetMarker { .. } => {
                self.styled_text(ui, line_index, rendered_token, dft_color)
            },
            SmaliToken::Offset { relative: _, absolute } => {
                let text_ui = self.styled_text(
                    ui, line_index, rendered_token, self.smali_style.offset,
                );
                if text_ui.clicked() {
                    self.scroll_to_offset(ui, line_index, *absolute);
                }
                text_ui
            },
            SmaliToken::Register(_) => {
                self.styled_text(ui, line_index, rendered_token, self.smali_style.register)
            },
            SmaliToken::RegisterRange(_, _) => {
                self.styled_text(ui, line_index, rendered_token, self.smali_style.register)
            },
            SmaliToken::MemberName(_) => {
                self.styled_text(ui, line_index, rendered_token, dft_color)
            },
            SmaliToken::Descriptor(s) => {
                let mut text_ui = self.styled_text(
                    ui, line_index, rendered_token, self.smali_style.desc,
                );
                let popup_id = text_ui.id.with("descriptor_click_popup");
                let click_popup_open = Popup::is_id_open(&text_ui.ctx, popup_id);
                if !click_popup_open && !text_ui.clicked() {
                    text_ui = text_ui.on_hover_ui(|ui| {
                        ui.style_mut().interaction.selectable_labels = true;
                        self.descriptor_menu(ui, s);
                    });
                }
                Popup::from_response(&text_ui)
                    .id(popup_id)
                    .kind(PopupKind::Tooltip)
                    .open_memory(text_ui.clicked().then_some(SetOpenCommand::Toggle))
                    .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
                    .show(|ui| {
                        ui.style_mut().interaction.selectable_labels = true;
                        self.descriptor_menu(ui, s);
                    });
                text_ui.context_menu(|ui| {
                    self.descriptor_menu(ui, s);
                });
                text_ui
            },
            SmaliToken::Literal(_) => {
                self.styled_text(ui, line_index, rendered_token, self.smali_style.literal)
            },
            SmaliToken::Other(_) => self.styled_text(ui, line_index, rendered_token, dft_color),
        }
    }

    fn styled_text(
        &self, ui: &mut Ui, line_index: usize, token: &SmaliLineToken, color: Color32,
    ) -> Response {
        ui.label(self.token_layout(line_index, token, color))
    }

    fn token_layout(
        &self, line_index: usize, token: &SmaliLineToken, color: Color32,
    ) -> LayoutJob {
        let mut job = LayoutJob::default();
        if !self.find_open {
            append_text(&mut job, &token.text, self.font, color, None);
            return job;
        }

        let mut cursor = 0usize;
        for (match_index, matched) in self.find.matches.iter().enumerate() {
            if matched.line != line_index
                || matched.end_byte <= token.start_byte
                || matched.start_byte >= token.end_byte
            {
                continue;
            }

            let start = matched.start_byte.max(token.start_byte) - token.start_byte;
            let end = matched.end_byte.min(token.end_byte) - token.start_byte;
            if start > cursor {
                append_text(&mut job, &token.text[cursor..start], self.font, color, None);
            }
            let background = if self.find.current == match_index {
                Some(self.smali_style.search_current)
            } else {
                Some(self.smali_style.search)
            };
            append_text(&mut job, &token.text[start..end], self.font, color, background);
            cursor = end;
        }

        if cursor < token.text.len() {
            append_text(&mut job, &token.text[cursor..], self.font, color, None);
        }
        job
    }

    fn source_file_info_menu(
        &mut self, ui: &mut Ui, source_file_info: &StrRef,
    ) {
        ui.horizontal(|ui| {
            let link = ui.link(format!("Export Source: {source_file_info}"));
            if link.clicked() {
                self.server.dialog_to_save_file(source_file_info);
            }
        });
    }

    fn descriptor_menu(
        &mut self, ui: &mut Ui, descriptor: &str,
    ) {
        ui.vertical(|ui| {
            if descriptor.starts_with('(') {
                self.descriptor_menu_for_fn(ui, descriptor);
            } else {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("type: ");
                    self.render_single_descriptor(ui, descriptor);
                });
            }
        });
    }

    // function descriptors, e.g. (Ljava/lang/String;I)V, show ui like:
    // arg1: Ljava/lang/String;
    // arg2: I
    // returned: V
    fn descriptor_menu_for_fn(
        &mut self, ui: &mut Ui, descriptor: &str,
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
                    self.render_single_descriptor(ui, arg);
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
                self.render_single_descriptor(ui, returned);
            })
        });
        None
    }

    fn render_single_descriptor(
        &mut self, ui: &mut Ui, descriptor: &str,
    ) {
        let existed = self.server.find_class(descriptor);
        if !existed {
            ui.label(format!("{descriptor}"));
        } else {
            let link = ui.link(descriptor);
            if link.clicked() {
                let file_open_message = UIMessage::OpenFile(
                    OpenFileMessage {
                        path: descriptor.into(),
                    }
                );
                self.app.send_message(file_open_message);
            }
        }
    }
}

fn append_text(
    job: &mut LayoutJob, text: &str, font: &FontId, color: Color32,
    background: Option<Color32>,
) {
    let mut format = TextFormat::simple(font.clone(), color);
    if let Some(background) = background {
        format.background = background;
    }
    job.append(text, 0.0, format);
}

#[derive(Copy, Clone, Debug)]
pub(crate) struct SmaliStyle {
    pub op: Color32,
    pub offset: Color32,
    pub register: Color32,
    pub desc: Color32,
    pub literal: Color32,
    pub highlight: Color32,
    pub search: Color32,
    pub search_current: Color32,
}

impl SmaliStyle {
    pub(crate) const LIGHT: SmaliStyle = SmaliStyle {
        op: Color32::from_rgb(235, 0, 0),
        offset: Color32::from_rgb(96, 96, 96),
        register: Color32::from_rgb(83, 141, 199),
        desc: Color32::from_rgb(153, 134, 255),
        literal: Color32::from_rgb(37, 203, 105),
        highlight: Color32::from_rgb(255, 199, 133),
        search: Color32::from_rgba_unmultiplied_const(255, 220, 80, 85),
        search_current: Color32::from_rgba_unmultiplied_const(255, 145, 45, 175),
    };

    pub(crate) const DARK: SmaliStyle = SmaliStyle {
        op: Color32::from_rgb(255, 100, 100),
        offset: SmaliStyle::LIGHT.offset,
        register: SmaliStyle::LIGHT.register,
        desc: SmaliStyle::LIGHT.desc,
        literal: SmaliStyle::LIGHT.literal,
        highlight: SmaliStyle::LIGHT.highlight,
        search: Color32::from_rgba_unmultiplied_const(180, 150, 0, 100),
        search_current: Color32::from_rgba_unmultiplied_const(255, 160, 40, 190),
    };
}
