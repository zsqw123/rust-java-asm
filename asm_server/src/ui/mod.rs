pub mod log;
pub mod msg;
pub mod font;
pub mod find;

pub use find::{FindMatch, FindState};
use crate::impls::fuzzy::SearchResult;
use crate::ui::log::LogHolder;
use crate::ui::AbsFile::{Dir, File};
use crate::{AsmServer, Instant, LoadingState};
use java_asm::smali::{SmaliNode, SmaliToken};
use java_asm::StrRef;
use ::log::Level;
use parking_lot::Mutex;
use std::collections::BTreeMap;
use std::iter::{Enumerate, Peekable};
use std::str::Split;
use std::sync::Arc;

/// contains all states of the app.
/// It's not like the [AsmServer] which only contains the information of a file.
/// App will exists even no file opened. (no [AsmServer] exists)
#[derive(Default, Clone, Debug)]
pub struct App {
    pub top: Arc<Mutex<Top>>,
    pub left: Arc<Mutex<Left>>,
    pub content: Arc<Mutex<Content>>,
    pub messages: Arc<Mutex<Vec<UIMessage>>>,
    pub toasts: Arc<Mutex<Vec<Toast>>>,
}

#[derive(Clone, Debug)]
pub enum UIMessage {
    OpenFile(OpenFileMessage),
    CloseDir(StrRef),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToastKind {
    Success,
    Error,
}

#[derive(Clone, Debug)]
pub struct Toast {
    pub kind: ToastKind,
    pub message: String,
    pub created_at: Instant,
}

#[derive(Clone, Debug)]
pub struct OpenFileMessage {
    pub path: StrRef,
}

#[derive(Default, Clone, Debug)]
pub struct AppContainer(Arc<App>);

impl AppContainer {
    pub fn top(&self) -> &Arc<Mutex<Top>> { &self.0.top }

    pub fn left(&self) -> &Arc<Mutex<Left>> { &self.0.left }

    pub fn set_left(&self, left: Left) { *self.0.left.lock() = left; }

    pub fn content(&self) -> &Arc<Mutex<Content>> { &self.0.content }

    pub fn send_message(&self, message: UIMessage) {
        self.0.messages.lock().push(message);
    }

    pub fn toasts(&self) -> &Arc<Mutex<Vec<Toast>>> { &self.0.toasts }

    pub fn push_toast(&self, kind: ToastKind, message: impl Into<String>) {
        let mut toasts = self.0.toasts.lock();
        const MAX_TOASTS: usize = 200;
        if toasts.len() == MAX_TOASTS {
            toasts.remove(0);
        }
        toasts.push(Toast {
            kind,
            message: message.into(),
            created_at: Instant::now(),
        });
    }

}

impl AppContainer {
    pub fn process_messages(&mut self, server: &mut AsmServer) {
        let messages = self.0.messages.lock().drain(..).collect::<Vec<_>>();
        for message in messages {
            match message {
                UIMessage::OpenFile(message) => {
                    server.switch_or_open(&message.path, self);
                }
                UIMessage::CloseDir(path) => {
                    server.close_dir(&path, self);
                }
            }
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct Top {
    pub loading_state: LoadingState,
    pub file_path: String,
    pub search_result: SearchResult,
}

#[derive(Default, Clone, Debug)]
pub struct Left {
    pub root_node: DirInfo,
    pub offset_key: Option<StrRef>,
    pub hint_key: Option<StrRef>,
}

#[derive(Clone, Debug)]
pub enum AbsFile<F, D> {
    File(F),
    Dir(D),
}

pub type FileEntry<'a> = AbsFile<&'a mut FileInfo, &'a mut RawDirInfo>;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FileInfo {
    pub title: StrRef,
    pub level: u16,
    pub file_key: StrRef,
}

// raw data without children
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RawDirInfo {
    pub opened: bool,
    pub level: u16,
    pub title: StrRef,
    pub dir_key: StrRef,
}

pub type DirMap = BTreeMap<StrRef, DirInfo>;
pub type FileMap = BTreeMap<StrRef, FileInfo>;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DirInfo {
    pub raw: RawDirInfo,
    pub dirs: DirMap,
    pub files: FileMap,
}

fn visible_items<'a, 'b>(
    // input
    dir_info: &'b mut DirInfo, offset_key: &Option<StrRef>,
    // output
    container: &'a mut Vec<FileEntry<'b>>, offset: &mut usize,
) {
    let opened = dir_info.raw.opened;
    container.push(Dir(&mut dir_info.raw));
    if !opened { return; }
    for (_, dir) in dir_info.dirs.iter_mut() {
        visible_items(dir, offset_key, container, offset);
    }
    for (_, file) in dir_info.files.iter_mut() {
        if let Some(file_key) = offset_key {
            if *file_key == file.file_key {
                *offset = container.len();
            }
        }
        container.push(File(file));
    }
}

impl DirInfo {
    pub fn from_classes(class_names: &[StrRef]) -> Self {
        let root_raw_dir = RawDirInfo {
            title: Arc::from("Root"),
            dir_key: Arc::from(""),
            level: 0,
            opened: true,
        };
        let mut root_node = DirInfo { raw: root_raw_dir, ..Default::default() };
        for class_name in class_names {
            root_node.put_entry_if_absent(class_name.clone());
        }
        root_node
    }

    pub fn get_entry(&self, path: &str) -> Option<AbsFile<&FileInfo, &DirInfo>> {
        let mut parts = Self::entry_parts(&path);
        while let Some((_, part)) = parts.next() {
            let part = Arc::from(part);
            let dir = self.dirs.get(&part);
            if let Some(dir) = dir {
                let next = parts.peek();
                if next.is_none() { return next.map(|_| Dir(dir)); }
                continue;
            }
            let file = self.files.get(&part);
            return file.map(|file| File(file));
        }
        None
    }

    pub fn put_entry_if_absent(&mut self, path: StrRef) {
        let mut parts = Self::entry_parts(&path);
        let mut current = self;
        while let Some((index, part)) = parts.next() {
            let index = index as u16;
            if parts.peek().is_none() {
                let file_key = Arc::clone(&path);
                let file_name = Arc::from(part);
                current.put_file_if_absent(index, file_key, file_name);
            } else {
                current = current.put_dir_if_absent(index, Arc::from(part));
            }
        }
    }

    pub fn visible_items(&'_ mut self, offset_key: Option<StrRef>) -> FileTreeBuildResult<'_> {
        let mut container = Vec::new();
        let mut index_of_offset = 0usize;
        visible_items(self, &offset_key, &mut container, &mut index_of_offset);
        FileTreeBuildResult {
            entries: container,
            required_file_index: index_of_offset,
        }
    }

    fn entry_parts(path: &str) -> Peekable<Enumerate<Split<'_, char>>> {
        path.split('/').enumerate().peekable()
    }

    fn put_file_if_absent(&mut self, level: u16, file_key: StrRef, file_name: StrRef) -> &mut FileInfo {
        let title = file_name.clone();
        self.files.entry(file_name).or_insert_with(|| {
            FileInfo { title, level, file_key, ..Default::default() }
        })
    }

    fn put_dir_if_absent(&mut self, level: u16, folder_name: StrRef) -> &mut DirInfo {
        let title = folder_name.clone();
        let parent_dir_key = &self.raw.dir_key;
        let dir_key: StrRef;
        if parent_dir_key.is_empty() {
            // direct n
            dir_key = folder_name.into();
        } else {
            dir_key = format!("{}/{}", parent_dir_key, folder_name).into();
        }
        self.dirs.entry(title.clone()).or_insert_with(|| {
            let raw = RawDirInfo { title, level, dir_key, ..Default::default() };
            DirInfo { raw, ..Default::default() }
        })
    }
}

pub struct FileTreeBuildResult<'a> {
    pub entries: Vec<FileEntry<'a>>,
    // the index of required file which used for initial scrolling offset.
    pub required_file_index: usize,
}

#[derive(Clone, Debug, Default)]
pub struct Content {
    // the index of the selected tab in opened_tabs.
    pub selected: Option<usize>,
    // all opened tabs.
    pub opened_tabs: Vec<Tab>,
}

#[derive(Clone, Debug)]
pub struct Tab {
    pub selected: bool,
    pub file_key: StrRef,
    pub title: StrRef,
    pub rendered_lines: Arc<Vec<SmaliLine>>,
    pub exported_content: Arc<str>,
    pub find: FindState,
    pub scroll_offset: f32,
}

#[derive(Clone, Debug)]
pub struct SmaliLine {
    pub text: String,
    pub tokens: Vec<SmaliLineToken>,
}

#[derive(Clone, Debug)]
pub struct SmaliLineToken {
    pub token: SmaliToken,
    pub text: String,
    pub start_byte: usize,
    pub end_byte: usize,
}

impl SmaliLine {
    pub fn from_node(smali_node: &SmaliNode) -> Vec<Self> {
        smali_node
            .render_to_lines()
            .into_iter()
            .map(|tokens| {
                let mut text = String::new();
                let tokens = tokens
                    .into_iter()
                    .map(|token| {
                        let token_text = display_token_text(&token);
                        let start_byte = text.len();
                        text.push_str(&token_text);
                        let end_byte = text.len();
                        SmaliLineToken {
                            token,
                            text: token_text,
                            start_byte,
                            end_byte,
                        }
                    })
                    .collect();
                Self { text, tokens }
            })
            .collect()
    }
}

fn display_token_text(token: &SmaliToken) -> String {
    match token {
        SmaliToken::SourceInfo(source) => format!("#from: {source}"),
        SmaliToken::Raw(raw) => raw.to_string(),
        SmaliToken::Op(op) => op.to_string(),
        SmaliToken::LineStartOffsetMarker { raw, .. } => raw.clone(),
        SmaliToken::Offset { relative, absolute } => format!("@{absolute}({relative:+})"),
        SmaliToken::Register(reg) => format!("v{reg}"),
        SmaliToken::RegisterRange(start, end) => format!("v{start}..v{end}"),
        SmaliToken::MemberName(name) => name.to_string(),
        SmaliToken::Descriptor(descriptor) => descriptor.to_string(),
        SmaliToken::Literal(literal) => literal.to_string(),
        SmaliToken::Other(other) => other.to_string(),
    }
}

pub struct LogDialog {
    pub selected_level: Level,
    pub filter: String,
    pub logs: LogHolder,
}
