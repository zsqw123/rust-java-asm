use crate::impls::server::FileOpenContext;
use crate::impls::util::new_tokio_thread;
use crate::ui::{AppContainer, Content, Tab};
use crate::{Accessor, AccessorEnum, AsmServer, LoadingState, ServerMut};
use java_asm::smali::SmaliNode;
use java_asm::{AsmErr, StrRef};
use log::{error, info};
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
        }
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
        let mut content_locked = render_target.content().lock();
        let content = content_locked.deref_mut();
        self.switch_or_open_lock_free(file_key, accessor, content);
    }

    pub fn switch_or_open_lock_free(
        &self, file_key: &str, accessor: &AccessorEnum, content: &mut Content,
    ) {
        let existed_tab = content.opened_tabs.iter().position(|tab| *tab.file_key == *file_key);
        if let Some(existed_tab) = existed_tab {
            content.selected = Some(existed_tab);
            return;
        }

        let smali = accessor.read_content(file_key);
        let Some(smali) = smali else { return; };
        let current_tab = Tab {
            selected: false,
            file_key: Arc::from(file_key),
            title: Arc::from(file_key),
            content: smali,
        };
        let current = content.opened_tabs.len();
        content.opened_tabs.push(current_tab);
        content.selected = Some(current);
    }
}


/// input operation processing
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


