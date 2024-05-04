use std::io::{BufReader, Read};

use java_asm_internal::err::{AsmErr, AsmResult};
use java_asm_internal::read::jvms::{FromReadContext, ReadContext};

use crate::jvms::element::ClassFile;
use crate::jvms::read::transform::transform_class_file;

mod jvms_reader;
mod transform;
pub mod util;

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
