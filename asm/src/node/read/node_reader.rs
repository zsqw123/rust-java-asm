use std::io::Read;

use java_asm_internal::err::{AsmErr, AsmResult};

use crate::jvms::element::{ClassFile, Const, MethodInfo};
use crate::jvms::read::JvmsClassReader;
use crate::node::element::ClassNode;
use crate::node::read::impls::ClassNodeFactory;

pub struct NodeReader {}

impl NodeReader {
    pub fn read_class_file<T: Read>(read: T) -> AsmResult<ClassNode> {
        Self::from_jvms(JvmsClassReader::read_class_file(read)?)
    }

    pub fn read_class_bytes(bytes: &[u8]) -> AsmResult<ClassNode> {
        Self::from_jvms(JvmsClassReader::read_class_bytes(bytes)?)
    }

    pub fn from_jvms(jvms_file: ClassFile) -> AsmResult<ClassNode> {
        ClassNodeFactory::from_jvms(jvms_file)
    }
}

pub(crate) struct ClassNodeContext<'a> {
    pub jvms_file: &'a ClassFile,
}

impl ClassNodeContext<'_> {
    pub fn read_utf8(&self, index: u16) -> AsmResult<String> {
        let constant = self.read_const(index)?;
        let Const::Utf8 { length, bytes } = constant else {
            return Err(AsmErr::IllegalArgument(
                format!("cannot read utf8 from constant pool, cp_index: {}, attr: {:?}", index, constant)
            ));
        };
        let length = *length as usize;
        let mut current_offset = 0usize;
        let mut code_points = vec![];

        for _ in 0..length {
            let byte = bytes[current_offset] as u32;
            current_offset += 1;
            let code_point = if byte & 0x80 == 0 {
                byte & 0x7F
            } else if byte & 0xE0 == 0xC0 {
                current_offset += 1;
                let byte2 = bytes[current_offset] as u32;
                ((byte & 0x1F) << 6) | (byte2 & 0x3F)
            } else {
                let byte2 = bytes[current_offset + 1] as u32;
                let byte3 = bytes[current_offset + 2] as u32;
                current_offset += 2;
                ((byte & 0xF) << 12) | ((byte2 & 0x3F) << 6) | (byte3 & 0x3F)
            };
            code_points.push(code_point);
        }


        let Ok(string) = String::from_utf8(bytes.clone()) else {
            return Err(AsmErr::IllegalArgument(
                format!("cannot convert utf8 bytes to string, cp_index: {}, bytes: {:?}", index, bytes)
            ));
        };
        Ok(string)
    }

    pub fn read_const(&self, index: u16) -> AsmResult<&Const> {
        Ok(&self.jvms_file.constant_pool[index as usize].info)
    }
}

pub(crate) struct MethodNodeContext<'a> {
    pub jvms_file: &'a ClassFile,
    pub method_info: &'a MethodInfo,
}
