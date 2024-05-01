use std::io;

pub enum AsmErr {
    ContentReadErr { io_error: io::Error },
    IllegalArgument { info: String },
}

pub(crate) type AsmResult<T> = Result<T, AsmErr>;
