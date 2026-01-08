use crate::impls::fuzzy::FuzzyMatchModel;
use crate::impls::server::FileOpenContext;
use crate::impls::util::new_tokio_thread;
use crate::ui::{AppContainer, Content, DirInfo, Left, Tab, Top};
use crate::{Accessor, AccessorEnum, ArcVarOpt, AsmServer, ExportableSource, LoadingState, ServerMut};
use java_asm::smali::SmaliNode;
use java_asm::{AsmErr, StrRef};
use log::{error, info};
use std::fs;
use std::fs::File;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::time::Instant;
use zip::result::ZipError;

impl AsmServer {
    pub fn new() -> Self {
        Self {
            loading_state: LoadingState {
                in_loading: true,
                loading_progress: 0.0,
                err: None,
            },
            accessor: Default::default(),
            classes: Default::default(),
            fuzzy: Default::default(),
        }
    }

    pub fn get_classes(&self) -> &ArcVarOpt<Vec<StrRef>> {
        let mut current = self.classes.lock();
        if current.is_some() { return &self.classes; }
        let accessor_locked = self.accessor.lock();
        let Some(accessor) = accessor_locked.deref() else { return &self.classes; };
        let classes = accessor.read_classes();
        current.replace(classes);
        &self.classes
    }

    fn get_or_create_fuzzy(&self, input: StrRef) -> &ArcVarOpt<FuzzyMatchModel> {
        let mut current = self.fuzzy.lock();
        if current.is_some() { return &self.fuzzy; }
        let load_start = Instant::now();
        let classes_locked = self.get_classes().lock();
        let Some(classes) = classes_locked.deref() else { return &self.fuzzy; };
        let fuzzy = FuzzyMatchModel::new(input, classes, 30);
        current.replace(fuzzy);
        let load_end = Instant::now();
        info!(
            "trie loaded in {}ms",
            load_end.duration_since(load_start).as_millis()
        );
        &self.fuzzy
    }

    pub fn smart_open(server: ServerMut, path: &str, render_target: AppContainer) {
        let context = FileOpenContext { path: path.to_string(), start_time: Instant::now() };
        new_tokio_thread(|runtime| async move {
            let new_server = AsmServer::new();
            *server.lock() = Some(new_server.clone());

            let sender = Self::create_message_handler(
                &server, &runtime, &render_target,
            );

            let path = &context.path;
            let accessor = new_server.accessor.clone();
            if path.ends_with(".apk") {
                let opened_file = File::open(path);
                match opened_file {
                    Ok(opened_file) => {
                        let read_result = Self::read_apk(opened_file, sender, accessor).await;
                        if let Err(e) = read_result {
                            error!("resolve file meets an error. {e:?}");
                        }
                    }
                    Err(e) => {
                        let error = OpenFileError::Io(e);
                        error!("read {path} meet an io error. {error:?}");
                    }
                }
            } else {
                error!("unsupported file type: {:?}", path);
            };
            new_server.on_file_opened(&context, render_target);
        });
    }

    pub fn switch_or_open(&self, file_key: &str, render_target: &AppContainer) {
        let accessor_locked = self.accessor.lock();
        let Some(accessor) = accessor_locked.deref() else { return; };
        let mut left = render_target.left().lock();
        left.offset_key = Some(file_key.into());
        left.hint_key = Some(file_key.into());
        let mut content = render_target.content().lock();
        let mut top = render_target.top().lock();
        self.switch_or_open_lock_free(file_key, accessor, &mut left, &mut content, &mut top);
    }

    pub fn close_dir(&self, file_key: &str, render_target: &AppContainer) {
        let mut left = render_target.left().lock();
        let mut current_node = &mut left.root_node;
        let path_parts: Vec<&str> = file_key.split('/').collect();
        for part in path_parts {
            let Some(dir) = current_node.dirs.get_mut(part) else {
                return;
            };
            if !dir.raw.opened {
                return;
            }
            current_node = dir;
        }
        current_node.raw.opened = false;
        let mut child_nodes: Vec<&mut DirInfo> = vec![current_node];
        while let Some(child_node) = child_nodes.pop() {
            for child in child_node.dirs.values_mut() {
                if !child.raw.opened { continue; }
                child.raw.opened = false;
                child_nodes.push(child);
            }
        }
    }

    pub fn switch_or_open_lock_free(
        &self, file_key: &str, accessor: &AccessorEnum,
        left: &mut Left, content: &mut Content, top: &mut Top,
    ) {
        self.switch_file_tree(left, file_key);
        let existed_tab = content.opened_tabs.iter().position(|tab| *tab.file_key == *file_key);
        if let Some(existed_tab) = existed_tab {
            content.selected = Some(existed_tab);
            return;
        }

        let smali = accessor.read_content(file_key);
        let Some(smali) = smali else {
            error!("content with key: `{file_key}` not found.");
            return;
        };
        let current_tab = Tab {
            selected: false,
            file_key: Arc::from(file_key),
            title: Arc::from(file_key),
            content: Arc::new(smali),
        };
        let current = content.opened_tabs.len();
        content.opened_tabs.push(current_tab);
        content.selected = Some(current);

        top.file_path = file_key.to_string();
    }

    // switch left side file tree to correct place.
    fn switch_file_tree(&self, left: &mut Left, file_key: &str) {
        let root_node = &mut left.root_node;
        let parts: Vec<&str> = file_key.split('/').collect();
        if parts.is_empty() { return; }

        let mut current_node = root_node;
        for part in parts {
            current_node.raw.opened = true;
            if let Some(child) = current_node.dirs.get_mut(part) {
                current_node = child;
            } else {
                break;
            }
        }
    }

    pub fn search(&self, top: &mut Top) {
        let query = &top.file_path;
        let query: StrRef = query.as_str().into();
        if query.len() > 255 { return; }
        let mut fuzzy_locked = self.get_or_create_fuzzy(query.clone()).lock();
        let Some(fuzzy) = fuzzy_locked.deref_mut() else { return; };
        let search_result = fuzzy.search_with_new_input(query);
        top.search_result = search_result;
    }
}


/// I/O operation processing
impl AsmServer {
    pub fn dialog_to_open_file(server: ServerMut, render_target: AppContainer) {
        rfd::FileDialog::new()
            .add_filter("APK", &["apk"])
            .pick_file()
            .map(|path| {
                if let Some(path) = path.to_str() {
                    Self::smart_open(server, path, render_target);
                }
            });
    }

    pub fn dialog_to_save_file(&self, source_key: &str) {
        let accessor_locked = self.accessor.lock();
        let Some(accessor) = accessor_locked.deref() else { return; };
        let Some(ExportableSource { exportable_name, source }) = accessor.peek_source(source_key) else { return; };
        let Some(saved_path) = rfd::FileDialog::new()
            .set_file_name(exportable_name.to_string())
            .save_file() else { return; };
        let result = fs::write(&saved_path, source);
        if let Err(e) = result {
            error!("save file {source_key} meets an error. {e:?}");
        };
        let parent_path = saved_path.parent();
        let Some(parent_path) = parent_path else { return; };
        if !parent_path.exists() { return; };
        open::that_in_background(&parent_path);
    }
}

#[derive(Debug)]
pub enum OpenFileError {
    Io(std::io::Error),
    LoadZip(ZipError),
    ResolveError(AsmErr),
    Custom(String),
}


/// Abilities when file loaded.
///
/// Such abilities just a wrapper of [Accessor], but provided a more convenient interface and
/// record some logs at the backend.
impl AsmServer {
    // read the input content (apk/dex/jar/class...)
    // return all class's internal names inside of this input.
    pub fn read_classes(&self) -> Vec<StrRef> {
        let accessor_locked = self.accessor.lock();
        let accessor = accessor_locked.deref();
        match accessor {
            None => Vec::new(),
            Some(accessor) => {
                let start = Instant::now();
                let classes = accessor.read_classes();
                info!("{} classes loaded from server in {:?}", classes.len(), start.elapsed());
                classes
            }
        }
    }

    /// see also: [AccessorEnum::exist_class]
    pub fn find_class(&self, class_key: &str) -> bool {
        let accessor_locked = self.accessor.lock();
        let accessor = accessor_locked.deref();
        match accessor {
            None => false,
            Some(accessor) => accessor.exist_class(class_key),
        }
    }

    pub fn read_content(&self, class_key: &str) -> Option<SmaliNode> {
        let accessor_locked = self.accessor.lock();
        let accessor = accessor_locked.deref();
        match accessor {
            None => None,
            Some(accessor) => accessor.read_content(class_key),
        }
    }
}


