use std::io::{BufWriter, Write};

use java_asm_internal::err::{AsmErr, AsmResult};
use java_asm_internal::write::jvms::WriteContext;

use crate::jvms::element::ClassFile;

pub struct JvmsClassWriter {}

impl JvmsClassWriter {
    pub fn write_class_file<T: Write>(write: T, class_file: ClassFile) -> AsmResult<()> {
        let mut writer = BufWriter::new(write);
        let bytes = Self::write_class_bytes(vec![], class_file)?;
        match writer.write(bytes.as_slice()) {
            Ok(_) => { Ok(()) }
            Err(io_err) => { Err(AsmErr::ContentWriteErr(io_err)) }
        }
    }

    pub fn write_class_bytes(bytes: Vec<u8>, class_file: ClassFile) -> AsmResult<Vec<u8>> {
        let mut write_context = WriteContext { bytes };
        write_context.push(class_file);
        Ok(write_context.bytes)
    }
}
