use crate::impls::apk_load::{read_apk, ApkAccessor};
use crate::{Accessor, AsmServer};
use java_asm::smali::SmaliNode;
use java_asm::{AsmErr, StrRef};
use std::cell::RefCell;
use std::io::{Read, Seek};
use zip::result::ZipError;
use zip::ZipArchive;

impl AsmServer<ApkAccessor> {
    pub fn from_apk(apk_content: impl Read + Seek) -> Result<Self, OpenFileError> {
        let zip = ZipArchive::new(apk_content)
            .map_err(OpenFileError::LoadZip)?;
        let accessor = RefCell::new(read_apk(zip)?);
        Ok(Self { accessor })
    }
}


impl<A: Accessor> AsmServer<A> {
    pub fn from_dex(dex_path: &str) -> Self {
        unimplemented!()
    }

    pub fn read_classes(&self) -> Vec<StrRef> {
        self.ensure_initialized();
        self.accessor.borrow().read_classes()
    }

    pub fn find_class(&self, class_key: &str) -> bool {
        self.ensure_initialized();
        self.accessor.borrow().exist_class(class_key)
    }

    pub fn read_content(&self, class_key: &str) -> SmaliNode {
        self.ensure_initialized();
        self.accessor.borrow().read_content(class_key)
    }

    fn ensure_initialized(&self) {
        self.accessor.borrow_mut().ensure_initialized();
    }
}

#[derive(Debug)]
pub enum OpenFileError {
    Io(std::io::Error),
    LoadZip(ZipError),
    ResolveError(AsmErr),
}

