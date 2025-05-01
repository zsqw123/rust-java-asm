use std::io::{BufWriter, Read, Write};

use crate::err::{AsmErr, AsmResult};
use crate::impls::jvms::r::{ReadContext, ReadFrom};

use crate::impls::jvms::r::transform::transform_class_file;
use crate::impls::jvms::w::WriteContext;
use crate::impls::ToArc;
use crate::jvms::element::ClassFile;

pub mod element;
pub mod attr;

pub struct JvmsClassReader;

impl JvmsClassReader {
    pub fn read_class_file<T: Read>(read: T) -> AsmResult<ClassFile> {
        let mut reader = read;
        let mut bytes = vec![];
        reader.read_to_end(&mut bytes)
            .map_err(|e| AsmErr::IOReadErr(e.arc()))?;
        Self::read_class_bytes(&bytes)
    }

    pub fn read_class_bytes(bytes: &[u8]) -> AsmResult<ClassFile> {
        let raw_file = ClassFile::read_from(&mut ReadContext::big_endian(bytes))?;
        let transformed = transform_class_file(raw_file)?;
        Ok(transformed)
    }
}


pub struct JvmsClassWriter;

impl JvmsClassWriter {
    pub fn write_class_file<T: Write>(write: T, class_file: ClassFile) -> AsmResult<()> {
        let mut writer = BufWriter::new(write);
        let bytes = Self::write_class_bytes(vec![], class_file)?;
        match writer.write(bytes.as_slice()) {
            Ok(_) => { Ok(()) }
            Err(e) => { Err(AsmErr::IOWriteErr(e.arc())) }
        }
    }

    pub fn write_class_bytes(bytes: Vec<u8>, class_file: ClassFile) -> AsmResult<Vec<u8>> {
        let mut write_context = WriteContext { bytes };
        write_context.write(class_file);
        Ok(write_context.bytes)
    }
}

