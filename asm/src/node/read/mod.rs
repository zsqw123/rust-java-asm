use std::io::Read;
use std::rc::Rc;
use java_asm_internal::err::AsmResult;
use crate::jvms::element::ClassFile;
use crate::jvms::read::JvmsClassReader;
use crate::node::element::ClassNode;
use crate::util::ToRc;

pub mod option;
pub mod node_reader;
mod const_reader;
mod attr_reader;
mod impls;

impl ClassNode {
    pub fn from_jvms(jvms_file: Rc<ClassFile>) -> AsmResult<ClassNode> {
        impls::from_jvms_internal(jvms_file)
    }

    pub fn from_read<T: Read>(read: T) -> AsmResult<ClassNode> {
        Self::from_jvms(JvmsClassReader::read_class_file(read)?.rc())
    }

    pub fn from_bytes(bytes: &[u8]) -> AsmResult<ClassNode> {
        Self::from_jvms(JvmsClassReader::read_class_bytes(bytes)?.rc())
    }
}
