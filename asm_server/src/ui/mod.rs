pub mod log;
pub mod msg;
pub mod font;

use crate::ui::log::LogHolder;
use crate::ui::AbsFile::{Dir, File};
use crate::LoadingState;
use java_asm::smali::SmaliNode;
use java_asm::StrRef;
use ::log::Level;
use parking_lot::Mutex;
use std::collections::BTreeMap;
use std::iter::{Enumerate, Peekable};
use std::str::Split;
use std::sync::Arc;

#[derive(Default, Clone, Debug)]
pub struct App {
    pub top: Arc<Mutex<Top>>,
    pub left: Arc<Mutex<Left>>,
    pub content: Arc<Mutex<Content>>,
}

#[derive(Default, Clone, Debug)]
pub struct AppContainer(Arc<App>);

impl AppContainer {
    pub fn top(&self) -> &Arc<Mutex<Top>> { &self.0.top }

    pub fn left(&self) -> &Arc<Mutex<Left>> { &self.0.left }

    pub fn set_left(&self, left: Left) { *self.0.left.lock() = left; }

    pub fn content(&self) -> &Arc<Mutex<Content>> { &self.0.content }
}

#[derive(Default, Clone, Debug)]
pub struct Top {
    pub loading_state: LoadingState,
}

#[derive(Default, Clone, Debug)]
pub struct Left {
    pub root_node: DirInfo,
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
}

pub type DirMap = BTreeMap<StrRef, DirInfo>;
pub type FileMap = BTreeMap<StrRef, FileInfo>;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DirInfo {
    pub raw: RawDirInfo,
    pub dirs: DirMap,
    pub files: FileMap,
}

fn visible_items<'a, 'b>(dir_info: &'b mut DirInfo, container: &'a mut Vec<FileEntry<'b>>) {
    let opened = dir_info.raw.opened;
    container.push(Dir(&mut dir_info.raw));
    if !opened { return; }
    for (_, dir) in dir_info.dirs.iter_mut() {
        visible_items(dir, container);
    }
    for (_, file) in dir_info.files.iter_mut() {
        container.push(File(file));
    }
}

impl DirInfo {
    pub fn from_classes(title: StrRef, class_names: &[StrRef]) -> Self {
        let mut root_node = DirInfo { raw: RawDirInfo { title, ..Default::default() }, ..Default::default() };
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

    pub fn visible_items(&mut self) -> Vec<FileEntry> {
        let mut container = Vec::new();
        visible_items(self, &mut container);
        container
    }

    fn entry_parts(path: &str) -> Peekable<Enumerate<Split<char>>> {
        path[1..(path.len() - 1)].split('/').enumerate().peekable()
    }

    fn put_file_if_absent(&mut self, level: u16, file_key: StrRef, file_name: StrRef) -> &mut FileInfo {
        let title = file_name.clone();
        self.files.entry(file_name).or_insert_with(|| {
            FileInfo { title, level, file_key, ..Default::default() }
        })
    }

    fn put_dir_if_absent(&mut self, level: u16, folder_name: StrRef) -> &mut DirInfo {
        let title = folder_name.clone();
        self.dirs.entry(folder_name).or_insert_with(|| {
            let raw = RawDirInfo { title, level, ..Default::default() };
            DirInfo { raw, ..Default::default() }
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct Content {
    pub current: Option<usize>,
    pub opened_tabs: Vec<Tab>,
}

#[derive(Clone, Debug)]
pub struct Tab {
    pub selected: bool,
    pub file_key: StrRef,
    pub title: StrRef,
    pub content: SmaliNode,
}

pub struct LogDialog {
    pub selected_level: Level,
    pub filter: String,
    pub logs: LogHolder,
}
