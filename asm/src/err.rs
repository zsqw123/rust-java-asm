use std::io;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum AsmErr {
    ContentReadErr(Rc<io::Error>),
    ContentWriteErr(Rc<io::Error>),
    IllegalArgument(String),
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

pub(crate) trait AsmResultOkExt<T> {
    fn ok(self) -> AsmResult<T>;
}

pub(crate) trait AsmResultExt<T> {
    fn ok_or_error(self, when_none: impl FnOnce() -> AsmResult<T>) -> AsmResult<T>;
}

impl<T> AsmResultOkExt<T> for T {
    #[inline]
    fn ok(self) -> AsmResult<T> {
        Ok(self)
    }
}

impl<T> AsmResultExt<T> for Option<T> {
    #[inline]
    fn ok_or_error(self, when_none: impl FnOnce() -> AsmResult<T>) -> AsmResult<T> {
        match self {
            Some(v) => Ok(v),
            None => when_none(),
        }
    }
}

pub trait AsmResultRcExt<T> {
    fn clone_if_error(self) -> AsmResult<T>;
}

impl<T> AsmResultRcExt<T> for Result<T, Rc<AsmErr>> {
    fn clone_if_error(self) -> AsmResult<T> {
        self.map_err(|e| (*e).clone())
    }
}
