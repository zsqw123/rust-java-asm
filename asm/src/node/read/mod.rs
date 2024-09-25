use std::io::Read;

use crate::err::AsmResult;

use crate::impls::node::r::impls::from_jvms_internal;
use crate::jvms::element::ClassFile;
use crate::JvmsClassReader;
use crate::node::element::ClassNode;

pub mod option;

impl ClassNode {
    pub fn from_jvms(jvms_file: ClassFile) -> AsmResult<ClassNode> {
        from_jvms_internal(jvms_file)
    }

    pub fn from_read<T: Read>(read: T) -> AsmResult<ClassNode> {
        Self::from_jvms(JvmsClassReader::read_class_file(read)?)
    }

    pub fn from_bytes(bytes: &[u8]) -> AsmResult<ClassNode> {
        Self::from_jvms(JvmsClassReader::read_class_bytes(bytes)?)
    }
}
