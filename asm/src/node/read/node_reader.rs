use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;

use java_asm_internal::err::AsmErr;

use crate::jvms::element::{ClassFile, MethodInfo};
use crate::node::values::ConstValue;

pub(crate) struct ClassNodeContext {
    pub jvms_file: Rc<ClassFile>,
    pub cp_cache: HashMap<u16, Rc<ConstValue>>,
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
