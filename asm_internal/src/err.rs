use std::io;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum AsmErr {
    ContentReadErr(Rc<io::Error>),
    ContentWriteErr(Rc<io::Error>),
    IllegalArgument(String),
    ReadMUTF8(String),
    ReadUTF8(String),
    ResolveNode(String),
}

impl AsmErr {
    pub fn e<T>(self) -> AsmResult<T> {
        Err(self)
    }
    
    pub fn loge(self) {
        eprintln!("{:?}", self);
    }
}

pub type AsmResult<T> = Result<T, AsmErr>;

pub trait AsmResultRcExt<T> {
    fn clone_if_error(self) -> AsmResult<T>;
}

impl<T> AsmResultRcExt<T> for Result<T, Rc<AsmErr>> {
    fn clone_if_error(self) -> AsmResult<T> {
        self.map_err(|e| (*e).clone())
    }
}
