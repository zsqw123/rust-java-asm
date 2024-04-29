use std::io;

pub enum AsmErr {
    ContentReadErr { io_error: io::Error },
}

pub(crate) type AsmResult<T> = Result<T, AsmErr>;
