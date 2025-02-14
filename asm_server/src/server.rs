use crate::impls::apk_load::read_apk;
use crate::ui::{App, DirInfo};
use crate::{Accessor, AsmServer};
use java_asm::smali::SmaliNode;
use java_asm::{AsmErr, StrRef};
use log::{error, info};
use std::io::{Read, Seek};
use std::rc::Rc;
use std::time::Instant;
use zip::result::ZipError;
use zip::ZipArchive;

/// Builders of [AsmServer]
impl AsmServer {
    pub fn smart_open(server: &mut Option<Self>, path: &str, render_target: &mut App) {
        let open_start = Instant::now();
        let res = if path.ends_with(".apk") {
            std::fs::File::open(path).map_err(OpenFileError::Io)
                .and_then(Self::from_apk)
        } else {
            let err = format!("unsupported file type: {:?}", path);
            error!("{}", err);
            Err(OpenFileError::Custom(err))
        };
        match res {
            Ok(res) => {
                info!("open file cost: {:?}", open_start.elapsed());
                res.render_to_app(render_target);
                *server = Some(res);
            }
            Err(e) => error!("{:?}", e),
        }
    }

    pub fn from_apk(apk_content: impl Read + Seek) -> Result<Self, OpenFileError> {
        let zip = ZipArchive::new(apk_content)
            .map_err(OpenFileError::LoadZip)?;
        let accessor = read_apk(zip)?.into();
        Ok(Self { accessor })
    }

    pub fn from_dex(dex_path: &str) -> Self {
        unimplemented!()
    }
}

/// input operation processing
impl AsmServer {
    pub fn dialog_to_open_file(server: &mut Option<Self>, render_target: &mut App) {
        rfd::FileDialog::new()
            .add_filter("APK", &["apk"])
            .pick_file()
            .map(|path| {
                if let Some(path) = path.to_str() {
                    Self::smart_open(server, path, render_target);
                }
            });
    }

    pub fn render_to_app(&self, app: &mut App) {
        let classes = self.read_classes();
        let start = Instant::now();
        let dir_info = DirInfo::from_classes(Rc::from("Root"), &classes);
        info!("resolve dir info cost: {:?}", start.elapsed());
        app.left.root_node = dir_info;
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
        let start = Instant::now();
        let classes = (&self.accessor).read_classes();
        info!("{} classes loaded from server in {:?}", classes.len(), start.elapsed());
        classes
    }

    pub fn find_class(&self, class_key: &str) -> bool {
        (&self.accessor).exist_class(class_key)
    }

    pub fn read_content(&self, class_key: &str) -> Option<SmaliNode> {
        (&self.accessor).read_content(class_key)
    }
}


