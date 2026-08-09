use java_asm::StrRef;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum AppMessage {
    FileDropped(PathBuf),
    SelectFile(StrRef),
}

#[derive(Debug, Clone)]
pub enum FindMessage {
    Open { file_key: StrRef },
    Close { file_key: StrRef },
    Update {
        file_key: StrRef,
        query: String,
        case_sensitive: bool,
    },
    Next { file_key: StrRef },
    Previous { file_key: StrRef },
}
