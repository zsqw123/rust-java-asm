use std::io::{BufReader, BufWriter, Read, Write};

use crate::err::{AsmErr, AsmResult};
use crate::impls::jvms::r::{FromReadContext, ReadContext};

use crate::impls::jvms::r::transform::transform_class_file;
use crate::impls::jvms::w::WriteContext;
use crate::jvms::element::ClassFile;
use crate::util::ToRc;

pub mod element;
pub mod attr;


pub struct JvmsClassReader {}

impl JvmsClassReader {
    pub fn read_class_file<T: Read>(read: T) -> AsmResult<ClassFile> {
        let mut reader = BufReader::new(read);
        let mut bytes = [];
        let read_result = reader.read(&mut bytes);
        if let Err(e) = read_result {
            return Err(AsmErr::ContentReadErr(e.rc()));
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


pub struct JvmsClassWriter {}

impl JvmsClassWriter {
    pub fn write_class_file<T: Write>(write: T, class_file: ClassFile) -> AsmResult<()> {
        let mut writer = BufWriter::new(write);
        let bytes = Self::write_class_bytes(vec![], class_file)?;
        match writer.write(bytes.as_slice()) {
            Ok(_) => { Ok(()) }
            Err(e) => { Err(AsmErr::ContentWriteErr(e.rc())) }
        }
    }

    pub fn write_class_bytes(bytes: Vec<u8>, class_file: ClassFile) -> AsmResult<Vec<u8>> {
        let mut write_context = WriteContext { bytes };
        write_context.push(class_file);
        Ok(write_context.bytes)
    }
}

