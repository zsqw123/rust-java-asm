use std::collections::HashMap;
use std::fmt::Display;
use std::io::Read;
use std::rc::Rc;

use java_asm_internal::err::{AsmErr, AsmResult};

use crate::jvms::element::{ClassFile, MethodInfo};
use crate::jvms::read::JvmsClassReader;
use crate::node::element::Attribute as NodeAttribute;
use crate::node::element::ClassNode;
use crate::node::read::impls::ClassNodeFactory;
use crate::node::values::ConstValue;

pub struct NodeReader {}

impl NodeReader {
    pub fn read_class_file<T: Read>(read: T) -> AsmResult<ClassNode> {
        Self::from_jvms(JvmsClassReader::read_class_file(read)?)
    }

    pub fn read_class_bytes(bytes: &[u8]) -> AsmResult<ClassNode> {
        Self::from_jvms(JvmsClassReader::read_class_bytes(bytes)?)
    }

    pub fn from_jvms(jvms_file: ClassFile) -> AsmResult<ClassNode> {
        ClassNodeFactory::from_jvms(Rc::new(jvms_file))
    }
}

pub(crate) struct ClassNodeContext {
    pub jvms_file: Rc<ClassFile>,
    pub(crate) cp_cache: HashMap<u16, Rc<ConstValue>>,
}

impl ClassNodeContext {
    pub fn new(jvms_file: Rc<ClassFile>) -> Self {
        Self {
            jvms_file,
            cp_cache: HashMap::new(),
        }
    }

    pub fn err<D: Display>(&mut self, msg: D) -> AsmErr {
        match self.name().ok() {
            Some(name) => AsmErr::ResolveNode(format!("class: {}, {}", name, msg)),
            None => AsmErr::ResolveNode(msg.to_string()),
        }
    }
}

pub(crate) struct MethodNodeContext {
    pub jvms_file: Rc<ClassFile>,
    pub method_info: Rc<MethodInfo>,
}
