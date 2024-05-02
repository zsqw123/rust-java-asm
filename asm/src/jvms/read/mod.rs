use std::io::{BufReader, Read};
use crate::err::{AsmErr, AsmResult};
use crate::jvms::element::ClassFile;
use crate::jvms::read::bytes::{FromReadContext, ReadContext};
use crate::jvms::read::transforms::transform_class_file;

mod jvms_reader;
mod bytes;
mod transforms;
pub mod util;
mod attrs_reader;

pub struct JvmsClassReader {}

impl JvmsClassReader {
    pub fn read_class_file<T: Read>(read: T) -> AsmResult<ClassFile> {
        let mut reader = BufReader::new(read);
        let mut bytes = [];
        let read_result = reader.read(&mut bytes);
        if let Err(e) = read_result {
            return Err(AsmErr::ContentReadErr(e));
        };
        Self::read_class_bytes(&bytes)
    }

    pub fn read_class_bytes(bytes: &[u8]) -> AsmResult<ClassFile> {
        let index = &mut 0;
        let raw_file = ClassFile::from_context(&mut ReadContext { bytes, index })?;
        let transformed = transform_class_file(raw_file)?;
        Ok(transformed)
    }
}
