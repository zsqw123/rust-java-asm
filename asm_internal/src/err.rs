use std::io;
use std::string::FromUtf8Error;

#[derive(Debug)]
pub enum AsmErr {
    ContentReadErr(io::Error),
    ContentWriteErr(io::Error),
    IllegalArgument(String),
    ReadUTF8(FromUtf8Error),
}

pub type AsmResult<T> = Result<T, AsmErr>;
