use std::io;

#[derive(Debug)]
pub enum AsmErr {
    ContentReadErr(io::Error),
    ContentWriteErr(io::Error),
    IllegalArgument(String),
    ReadMUTF8(String),
    ReadUTF8(String),
    ResolveNode(String),
}

impl AsmErr {
    pub fn e<T>(self) -> AsmResult<T> {
        Err(self)
    }
}

pub type AsmResult<T> = Result<T, AsmErr>;
