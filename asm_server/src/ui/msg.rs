use java_asm::StrRef;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum AppMessage {
    FileDropped(PathBuf),
    SelectFile(StrRef),
}
