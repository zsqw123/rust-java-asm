use crate::impls::fuzzy::FuzzyMatchModel;
use crate::impls::server::{FileOpenContext, ServerMessage};
use crate::mapping::{Mapping, MappingError};
use crate::targets::{schedule_task, Instant};
use crate::rw_access::{ReadAccess, ReadError, WriteAccess};
use crate::ui::{AppContainer, Content, DirInfo, Left, SmaliLine, Tab, Top};
use crate::{Accessor, AccessorEnum, ArcVarOpt, AsmServer, ExportableSource, LoadingState, ServerMut};
use java_asm::smali::{MappedName, SmaliNode};
use java_asm::{AsmErr, StrRef};
use log::{error, info};
use std::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use zip::result::ZipError;

impl AsmServer {
    pub fn new() -> Self {
        Self {
            loading_state: LoadingState {
                in_loading: true,
                loading_progress: 0.0,
                loading_message: "Preparing to load...".into(),
                err: None,
            },
            accessor: Default::default(),
            classes: Default::default(),
            fuzzy: Default::default(),
            mapping: Default::default(),
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
        if self.fuzzy.lock().is_some() { return &self.fuzzy; }
        let load_start = Instant::now();
        let classes_locked = self.get_classes().lock();
        let Some(classes) = classes_locked.deref() else { return &self.fuzzy; };
        let display_classes = self.mapped_classes(classes);
        let display_names: Vec<_> = display_classes.into_iter()
            .map(|name| Arc::clone(&name.display_name))
            .collect();
        let fuzzy = FuzzyMatchModel::new(input, &display_names, 30);
        self.fuzzy.lock().get_or_insert(fuzzy);
        let load_end = Instant::now();
        info!(
            "trie loaded in {}ms",
            load_end.duration_since(load_start).as_millis()
        );
        &self.fuzzy
    }

    pub fn smart_open(server: ServerMut, read_access: ReadAccess, render_target: AppContainer) {
        Self::smart_open_many(server, vec![read_access], render_target);
    }

    pub fn smart_open_many(
        server: ServerMut, read_accesses: Vec<ReadAccess>, render_target: AppContainer,
    ) {
        let Some(first_access) = read_accesses.first() else { return; };
        let file_name = if read_accesses.len() == 1 {
            first_access.name()
        } else {
            format!("{} files", read_accesses.len())
        };
        let context = FileOpenContext { file_name, start_time: Instant::now() };
        schedule_task(async move {
            let new_server = AsmServer::new();
            *server.lock() = Some(new_server.clone());
            *render_target.content().lock() = Content::default();
            new_server.on_progress_update(&render_target);

            let sender = Self::create_message_handler(
                &server, &render_target,
            );

            let accessor = Arc::clone(&new_server.accessor);
            crate::impls::apk_load::send_progress(
                &sender, 0.0, "Reading input files...",
            ).await;
            let mut inputs = Vec::with_capacity(read_accesses.len());
            for read_access in read_accesses {
                let file_name = read_access.name();
                match read_access.read().await {
                    Ok(content) => {
                        let content = content.to_vec();
                        inputs.push((file_name, content));
                    }
                    Err(error) => {
                        let message = format!(
                            "Failed to read `{file_name}`: {}",
                            OpenFileError::ReadError(error),
                        );
                        error!("{message}");
                        let _ = sender.send(ServerMessage::Error(message)).await;
                        new_server.on_file_opened(&context, render_target);
                        return;
                    }
                }
            }
            let read_result = Self::read_files(inputs, sender.clone(), accessor).await;
            if let Err(e) = read_result {
                let message = format!("Failed to load input: {e}");
                error!("{message}");
                let _ = sender.send(ServerMessage::Error(message)).await;
            }
            new_server.on_file_opened(&context, render_target);
        });
    }

    pub fn switch_or_open(&self, file_key: &str, render_target: &AppContainer) {
        let accessor_locked = self.accessor.lock();
        let Some(accessor) = accessor_locked.deref() else { return; };
        let Some(file_key) = self.resolve_raw_class_name(file_key, accessor) else { return; };
        let mut left = render_target.left().lock();
        left.offset_key = Some(Arc::clone(&file_key));
        left.hint_key = Some(Arc::clone(&file_key));
        let mut content = render_target.content().lock();
        let mut top = render_target.top().lock();
        self.switch_or_open_lock_free(&file_key, accessor, &mut left, &mut content, &mut top);
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
        let Some(mut smali) = smali else {
            error!("content with key: `{file_key}` not found.");
            return;
        };
        if let Some(mapping) = self.mapping.lock().as_ref() {
            mapping.apply_to_smali(file_key, &mut smali);
        }
        let title = self.mapped_class_name(file_key.into());
        let rendered_lines = Arc::new(SmaliLine::from_node(&smali));
        let current_tab = Tab {
            selected: false,
            file_key: Arc::from(file_key),
            title: Arc::clone(&title.display_name),
            rendered_lines,
            find: Default::default(),
            scroll_offset: 0.0,
        };
        let current = content.opened_tabs.len();
        content.opened_tabs.push(current_tab);
        content.selected = Some(current);

        top.file_path = title.display_name.to_string();
    }

    // switch left side file tree to correct place.
    fn switch_file_tree(&self, left: &mut Left, file_key: &str) {
        let root_node = &mut left.root_node;
        let display_name = self.mapped_class_name(file_key.into());
        let parts: Vec<&str> = display_name.display_name.split('/').collect();
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
        let mut fuzzy_locked = self.get_or_create_fuzzy(Arc::clone(&query)).lock();
        let Some(fuzzy) = fuzzy_locked.deref_mut() else { return; };
        let search_result = fuzzy.search_with_new_input(query);
        top.search_result = search_result;
    }

    pub fn mapped_class_name(&self, raw_name: StrRef) -> MappedName {
        self.mapping.lock().as_ref()
            .map(|mapping| mapping.mapped_class_name(Arc::clone(&raw_name)))
            .unwrap_or_else(|| raw_name.into())
    }

    pub(crate) fn mapped_classes(&self, raw_names: &[StrRef]) -> Vec<MappedName> {
        let mapping = self.mapping.lock();
        raw_names.iter().map(|raw_name| {
            mapping.as_ref()
                .map(|mapping| mapping.mapped_class_name(Arc::clone(raw_name)))
                .unwrap_or_else(|| Arc::clone(raw_name).into())
        }).collect()
    }

    pub fn raw_class_name(&self, display_name: &str) -> Option<StrRef> {
        self.mapping.lock().as_ref()?.raw_class_name(display_name)
    }

    pub fn can_import_mapping(&self) -> bool {
        !self.loading_state.in_loading && self.accessor.lock().is_some()
    }

    fn resolve_raw_class_name(
        &self, name: &str, accessor: &AccessorEnum,
    ) -> Option<StrRef> {
        if accessor.exist_class(name) {
            return Some(name.into());
        }
        let raw_name = self.raw_class_name(name)?;
        accessor.exist_class(&raw_name).then_some(raw_name)
    }

    fn apply_mapping(&self, mapping: Mapping, render_target: &AppContainer) -> (usize, usize) {
        let accessor_locked = self.accessor.lock();
        let Some(accessor) = accessor_locked.as_ref() else { return (0, 0); };
        let raw_classes = accessor.read_classes();
        let matched = raw_classes.iter()
            .filter(|name| mapping.display_class_name(name).is_some())
            .count();
        let total = raw_classes.len();
        *self.mapping.lock() = Some(mapping);
        *self.fuzzy.lock() = None;

        let mapped_classes = self.mapped_classes(&raw_classes);
        let (offset_key, hint_key) = {
            let left = render_target.left().lock();
            (
                left.offset_key.as_ref().map(Arc::clone),
                left.hint_key.as_ref().map(Arc::clone),
            )
        };
        render_target.set_left(Left {
            root_node: DirInfo::from_classes(&mapped_classes),
            offset_key,
            hint_key,
        });

        let mut content = render_target.content().lock();
        for tab in &mut content.opened_tabs {
            let Some(mut smali) = accessor.read_content(&tab.file_key) else { continue; };
            if let Some(mapping) = self.mapping.lock().as_ref() {
                mapping.apply_to_smali(&tab.file_key, &mut smali);
            }
            let title = self.mapped_class_name(Arc::clone(&tab.file_key));
            tab.title = Arc::clone(&title.display_name);
            tab.rendered_lines = Arc::new(SmaliLine::from_node(&smali));
            tab.find = Default::default();
        }
        if let Some(selected) = content.selected
            && let Some(tab) = content.opened_tabs.get(selected)
        {
            let mut top = render_target.top().lock();
            top.file_path = tab.title.to_string();
            top.search_result = Default::default();
        }
        (matched, total)
    }

    pub fn import_mapping(
        &self, bytes: &[u8], render_target: &AppContainer,
    ) -> Result<(usize, usize), MappingError> {
        let mapping = Mapping::parse(bytes)?;
        Ok(self.apply_mapping(mapping, render_target))
    }
}

impl Default for AsmServer {
    fn default() -> Self {
        Self::new()
    }
}


/// I/O operation processing
impl AsmServer {
    pub fn dialog_to_open_file(server: ServerMut, render_target: AppContainer) {
        schedule_task(async {
            let dialog = rfd::AsyncFileDialog::new()
                .add_filter(
                    "Android packages / DEX",
                    &["apk", "apks", "xapk", "aab", "zip", "dex"],
                );
            let read_accesses = ReadAccess::new_multiple(dialog).await;
            let Some(read_accesses) = read_accesses else { return; };
            Self::smart_open_many(server, read_accesses, render_target);
        });
    }

    pub fn dialog_to_open_mapping(server: ServerMut, render_target: AppContainer) {
        schedule_task(async move {
            let dialog = rfd::AsyncFileDialog::new()
                .add_filter("R8 / ProGuard mapping", &["txt", "map", "mapping"]);
            let Some(read_access) = ReadAccess::new(dialog).await else { return; };
            let mapping_name = read_access.name();
            let bytes = match read_access.read().await {
                Ok(bytes) => bytes,
                Err(error) => {
                    render_target.error_toast(
                        format!("Failed to read `{mapping_name}`: {error}"),
                    );
                    return;
                }
            };
            let mapping = match Mapping::parse(&bytes) {
                Ok(mapping) => mapping,
                Err(error) => {
                    render_target.error_toast(
                        format!("Failed to import `{mapping_name}`: {error}"),
                    );
                    return;
                }
            };
            let server_locked = server.lock();
            let Some(server) = server_locked.as_ref() else { return; };
            let (matched, total) = server.apply_mapping(mapping, &render_target);
            render_target.success_toast(
                format!("Loaded `{mapping_name}`: {matched}/{total} classes matched"),
            );
        });
    }

    pub fn dialog_to_save_file(&self, source_key: &str) {
        let accessor_locked = self.accessor.lock();
        let Some(accessor) = accessor_locked.deref() else { return; };
        let Some(ExportableSource { exportable_name, source }) = accessor.peek_source(source_key) else { return; };
        let file_save_dialog = rfd::AsyncFileDialog::new()
            .set_file_name(exportable_name.to_string());
        // clone source key for async move
        let source_key = source_key.to_string();
        schedule_task(async move {
            let write_access = WriteAccess::new(file_save_dialog).await;
            let Some(write_access) = write_access else {
                error!("create write access of {source_key} failed when saving files.");
                return;
            };
            let write_result = write_access.write(&source).await;
            if let Err(e) = write_result {
                error!("save file {source_key} meets an error. {e:?}");
            };
            let saved_path = write_access.guess_path();
            crate::targets::reveal_parent(&saved_path);
        });
    }
}

#[derive(Debug)]
pub enum OpenFileError {
    Io(std::io::Error),
    ReadError(ReadError),
    LoadZip(ZipError),
    ResolveError(AsmErr),
    Custom(String),
}

impl Display for OpenFileError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "I/O error: {error}"),
            Self::ReadError(error) => write!(formatter, "read error: {error}"),
            Self::LoadZip(error) => write!(formatter, "invalid ZIP archive: {error}"),
            Self::ResolveError(error) => write!(formatter, "resolve error: {error:?}"),
            Self::Custom(message) => formatter.write_str(message),
        }
    }
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
            Some(accessor) => self.resolve_raw_class_name(class_key, accessor).is_some(),
        }
    }

    pub fn read_content(&self, class_key: &str) -> Option<SmaliNode> {
        let accessor_locked = self.accessor.lock();
        let accessor = accessor_locked.deref();
        match accessor {
            None => None,
            Some(accessor) => {
                let raw_name = self.resolve_raw_class_name(class_key, accessor)?;
                let mut smali = accessor.read_content(&raw_name)?;
                if let Some(mapping) = self.mapping.lock().as_ref() {
                    mapping.apply_to_smali(&raw_name, &mut smali);
                }
                Some(smali)
            }
        }
    }

    pub fn render_content_to_text(&self, class_key: &str) -> Option<String> {
        Some(self.read_content(class_key)?.render(0))
    }
}

#[cfg(test)]
mod tests {
    use super::AsmServer;
    use crate::mapping::Mapping;
    use crate::ui::{AbsFile, AppContainer};
    use std::sync::Arc;
    use tokio::sync::mpsc;

    #[test]
    fn mapping_updates_tree_search_tabs_and_smali() {
        let server = AsmServer::new();
        let (sender, _receiver) = mpsc::channel(16);
        let bytes = include_bytes!("../../asm/tests/res/dex/classes14.dex").to_vec();
        futures::executor::block_on(AsmServer::read_files(
            vec![("classes14.dex".to_owned(), bytes)], sender,
            Arc::clone(&server.accessor),
        )).unwrap();

        let app = AppContainer::default();
        server.switch_or_open("Lj$/time/Ser;", &app);
        {
            let mut content = app.content().lock();
            let tab = &mut content.opened_tabs[0];
            tab.find.open = true;
            tab.find.query = "writeExternal".to_owned();
            tab.find.update_matches(tab.rendered_lines.iter().map(|line| line.text.as_str()));
            assert!(!tab.find.matches.is_empty());
        }
        let mapping = Mapping::parse_str(r#"
com.example.SerializationProxy -> j$.time.Ser:
    byte kind -> type
    void write(java.io.ObjectOutput) -> writeExternal
"#).unwrap();
        {
            let mut left = app.left().lock();
            left.offset_key = Some("Lj$/time/Ser;".into());
            left.hint_key = Some("Lj$/time/Ser;".into());
        }
        let (matched, total) = server.apply_mapping(mapping, &app);
        assert_eq!(matched, 1);
        assert!(total > matched);
        {
            let left = app.left().lock();
            assert_eq!(left.offset_key.as_deref(), Some("Lj$/time/Ser;"));
            assert_eq!(left.hint_key.as_deref(), Some("Lj$/time/Ser;"));
            let Some(AbsFile::File(file)) = left.root_node
                .get_entry("Lcom/example/SerializationProxy;")
            else {
                panic!("mapped class is missing from the file tree")
            };
            assert_eq!(file.file_key.as_ref(), "Lj$/time/Ser;");
        }

        {
            let mut top = app.top().lock();
            top.file_path = "SerializationProxy".to_owned();
            server.search(&mut top);
            assert_eq!(
                top.search_result.items[0].content.as_ref(),
                "Lcom/example/SerializationProxy;",
            );
        }

        server.switch_or_open("Lcom/example/SerializationProxy;", &app);
        let content = app.content().lock();
        assert_eq!(content.opened_tabs.len(), 1);
        let tab = &content.opened_tabs[0];
        assert_eq!(tab.file_key.as_ref(), "Lj$/time/Ser;");
        assert_eq!(tab.title.as_ref(), "Lcom/example/SerializationProxy;");
        assert!(!tab.find.open);
        assert!(tab.find.query.is_empty());
        assert!(tab.find.matches.is_empty());
        let rendered_content = server.render_content_to_text(&tab.file_key).unwrap();
        assert!(rendered_content.contains("Lcom/example/SerializationProxy;"));
        assert!(rendered_content.lines()
            .any(|line| line.contains("write (Ljava/io/ObjectOutput;)V")));
    }
}
