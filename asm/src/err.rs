use std::io;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum AsmErr {
    // something wrong when trying to access invalid index.
    // e.g. `constant_pool_count` declared it is 3, but `constant_pool` has only 2 elements.
    // e.g. `this_class` declared it is 3, but `constant_pool` has only 2 elements.
    OutOfRange(usize),
    // io error while reading content.
    IOReadErr(Arc<io::Error>),
    // io error while writing content.
    IOWriteErr(Arc<io::Error>),
    // illegal format with custom messages.
    IllegalFormat(String),
    // illegal utf8 format when reading an utf8 character from the constant pool.
    ReadUTF8(String),
    // illegal format when resolve jvms file into a node file.
    ResolveNode(String),
    // unknown instruction.
    UnknownInsn(u8),
    // invalid leb128 format for dex at specific offset.
    InvalidLEB128(usize),
    // unknown dex payload format.
    UnknownDexPayload(u8),
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
