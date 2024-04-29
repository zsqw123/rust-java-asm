use std::io::{BufReader, Read};

use crate::err::{AsmErr, AsmResult};
use crate::jvms::ClassFile;
use crate::reader::bytes_reader::{FromReadContext, ReadContext, UReader};

struct JvmsClassReader {}

impl JvmsClassReader {
    fn from_readable<T: Read>(read: T) -> AsmResult<ClassFile> {
        let mut reader = BufReader::new(read);
        let mut str = String::new();
        let read_result = reader.read_to_string(&mut str);
        if let Err(e) = read_result {
            return Err(AsmErr::ContentReadErr { io_error: e });
        };
        let bytes = str.as_bytes();
        ClassFile::from_context(&(bytes, &mut 0))
    }
}


impl FromReadContext<ClassFile> for ClassFile {
    fn from_context(context: &ReadContext) -> AsmResult<ClassFile> {
        ClassFile {
            magic: context.read_u32(),
            minor_version: context.read_u16(),
            major_version: context.read_u16(),
            constant_pool_count: context.read_u16(),
            constant_pool: context
        }
        Err()
    }
}



