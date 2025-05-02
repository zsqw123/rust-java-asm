use crate::impls::apk_load::read_apk;
use crate::ui::{AppContainer, DirInfo, Left};
use crate::{Accessor, AccessorEnum, AccessorMut, AsmServer, LoadingState, ServerMut};
use java_asm::smali::SmaliNode;
use java_asm::{AsmErr, StrRef};
use log::{error, info};
use std::fs::File;
use std::io::{Read, Seek};
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::runtime;
use zip::result::ZipError;
use zip::ZipArchive;

/// Builders of [AsmServer]
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
        std::thread::spawn(move || {
            let tokio_runtime = runtime::Builder::new_multi_thread()
                .enable_all().build().unwrap();
            tokio_runtime.block_on(async move {
                let new_server = AsmServer::new();
                let path = &context.path;
                let accessor = new_server.accessor.clone();
                if path.ends_with(".apk") {
                    let opened_file = File::open(path);
                    match opened_file {
                        Ok(opened_file) => {
                            let read_result = Self::from_apk(opened_file, accessor);
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
                *server.lock() = Some(new_server.clone());
                new_server.on_file_opened(&context, render_target);
            })
        });
    }

    fn on_file_opened(
        &self,
        context: &FileOpenContext,
        render_target: AppContainer,
    ) {
        let FileOpenContext { path, start_time } = context;
        info!("open file {path} cost: {:?}", start_time.elapsed());
        self.render_to_app(render_target);
    }

    pub fn from_apk(
        apk_content: impl Read + Seek,
        accessor: AccessorMut,
    ) -> Result<(), OpenFileError> {
        let zip = ZipArchive::new(apk_content)
            .map_err(OpenFileError::LoadZip)?;
        let apk_accessor = read_apk(zip)?;
        // safe unwrap, no other places in current thread will access it.
        *accessor.lock() = Some(AccessorEnum::Apk(apk_accessor));
        Ok(())
    }

    pub fn from_dex(dex_path: &str) -> Self {
        unimplemented!()
    }
}

struct FileOpenContext {
    pub path: String,
    pub start_time: Instant,
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

    pub fn render_to_app(&self, app: AppContainer) {
        let classes = self.read_classes();
        let start = Instant::now();
        let dir_info = DirInfo::from_classes(Arc::from("Root"), &classes);
        info!("resolve dir info cost: {:?}", start.elapsed());
        app.set_left(Left { root_node: dir_info });
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


