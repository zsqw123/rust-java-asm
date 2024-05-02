use std::io::{BufWriter, Write};

use crate::err::{AsmErr, AsmResult};
use crate::jvms::element::ClassFile;

mod jvms_writer;
mod bytes;

pub struct JvmsClassWriter {}

impl JvmsClassWriter {
    pub fn write_class_file<T: Write>(write: T, class_file: ClassFile) -> AsmResult<()> {
        let mut writer = BufWriter::new(write);
        // let read_result = writer.write(&mut str);
        // if let Err(e) = read_result {
        //     return Err(AsmErr::ContentReadErr(e));
        // };
        // Self::write_class_bytes(bytes)
        todo!()
    }

    pub fn write_class_bytes(bytes: &[u8], class_file: ClassFile) -> AsmResult<()> {
        todo!()
        // let index = &mut 0;
        // let raw_file = ClassFile::from_context(&mut crate::jvms::read::bytes::ReadContext { bytes, index })?;
        // let transformed = crate::jvms::read::transforms::transform_class_file(raw_file)?;
        // Ok(transformed)
    }
}

