pub mod log;
pub mod msg;
pub mod font;

use crate::ui::log::LogHolder;
use crate::ui::FileTree::{Dir, File};
use java_asm::smali::SmaliNode;
use java_asm::StrRef;
use ::log::Level;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Default, Clone, Debug)]
pub struct App {
    pub left: Left,
    pub content: Content,
}

#[derive(Default, Clone, Debug)]
pub struct Left {
    pub root_node: DirInfo,
}

#[derive(Clone, Debug)]
pub enum FileTree<F, D> {
    File(F),
    Dir(D),
}

#[derive(Clone, Debug, Default)]
pub struct FileInfo {
    pub selected: bool,
    pub title: StrRef,
    pub file_key: StrRef,
}

#[derive(Clone, Debug, Default)]
pub struct DirInfo {
    pub selected: bool,
    pub opened: bool,
    pub title: StrRef,
    pub dirs: HashMap<StrRef, DirInfo>,
    pub files: HashMap<StrRef, FileInfo>,
}

impl DirInfo {
    pub fn from_classes(title: StrRef, class_names: &[StrRef]) -> Self {
        let mut root_node = DirInfo { title, ..Default::default() };
        for class_name in class_names {
            root_node.put_entry_if_absent(Rc::clone(class_name));
        }
        root_node
    }

    pub fn get_entry(&self, path: &str) -> Option<FileTree<&FileInfo, &DirInfo>> {
        let mut parts = path.split('/').peekable();
        while let Some(part) = parts.next() {
            let part = Rc::from(part);
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
        let mut parts = path.split('/').peekable();
        let mut current = self;
        while let Some(part) = parts.next() {
            if parts.peek().is_none() {
                let file_key = Rc::clone(&path);
                let file_name = Rc::from(part);
                current.put_file_if_absent(file_key, file_name);
            } else {
                current = current.put_dir_if_absent(Rc::from(part));
            }
        }
    }

    fn put_file_if_absent(&mut self, file_key: StrRef, file_name: StrRef) -> &mut FileInfo {
        let title = Rc::clone(&file_name);
        self.files.entry(file_name).or_insert_with(|| {
            FileInfo { title, file_key, ..Default::default() }
        })
    }

    fn put_dir_if_absent(&mut self, folder_name: StrRef) -> &mut DirInfo {
        let title = Rc::clone(&folder_name);
        self.dirs.entry(folder_name).or_insert_with(|| {
            DirInfo { title, ..Default::default() }
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct Content {
    pub opened_tabs: Vec<Tab>,
}

#[derive(Clone, Debug)]
pub struct Tab {
    pub selected: bool,
    pub title: String,
    pub content: SmaliNode,
}

pub struct LogDialog {
    pub selected_level: Level,
    pub filter: String,
    pub logs: LogHolder,
}
