use crate::impls::apk_load::read_apk;
use crate::ui::{App, DirInfo};
use crate::{Accessor, AsmServer};
use java_asm::smali::SmaliNode;
use java_asm::{AsmErr, StrRef};
use std::io::{Read, Seek};
use std::rc::Rc;
use zip::result::ZipError;
use zip::ZipArchive;

impl AsmServer {
    pub fn smart_open(path: &str) -> Result<Self, OpenFileError> {
        if path.ends_with(".apk") {
            let file = std::fs::File::open(path).map_err(OpenFileError::Io)?;
            Self::from_apk(file)
        } else {
            Err(OpenFileError::Custom("unsupported file type".to_string()))
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

    pub fn render_to_app(&self, app: &mut App) {
        let classes = self.read_classes();
        let dir_info = DirInfo::from_classes(Rc::from("Root"), &classes);
        app.left.root_node = dir_info;
    }

    pub fn read_classes(&self) -> Vec<StrRef> {
        (&self.accessor).read_classes()
    }

    pub fn find_class(&self, class_key: &str) -> bool {
        (&self.accessor).exist_class(class_key)
    }

    pub fn read_content(&self, class_key: &str) -> Option<SmaliNode> {
        (&self.accessor).read_content(class_key)
    }
}

#[derive(Debug)]
pub enum OpenFileError {
    Io(std::io::Error),
    LoadZip(ZipError),
    ResolveError(AsmErr),
    Custom(String),
}

