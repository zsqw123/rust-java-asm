use std::io;
use std::string::FromUtf8Error;

pub enum AsmErr {
    ContentReadErr(io::Error),
    IllegalArgument(String),
    ReadUTF8(FromUtf8Error),
}

pub(crate) type AsmResult<T> = Result<T, AsmErr>;
