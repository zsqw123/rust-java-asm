use std::rc::Rc;

use java_asm_internal::err::AsmResult;

use crate::jvms::element::{ClassFile, MethodInfo};
use crate::node::element::{ClassNode, MethodNode};
use crate::node::read::node_reader::{ClassNodeContext, MethodNodeContext};

pub struct ClassNodeFactory {}

impl ClassNodeFactory {
    pub fn from_jvms(jvms_file: ClassFile) -> AsmResult<ClassNode> {
        let jvms_file = Rc::new(jvms_file);
        let mut class_context = ClassNodeContext::new(Rc::clone(&jvms_file));
        let class_node = ClassNode {
            minor_version: jvms_file.minor_version,
            major_version: jvms_file.major_version,
            access: jvms_file.access_flags,
            name: class_context.read_class_info(jvms_file.this_class)?,
        };
        Ok(class_node)
    }
}

fn method_from_jvms(class_context: &mut ClassNodeContext, method_info: Rc<MethodInfo>) -> AsmResult<MethodNode> {
    let jvms_file = Rc::clone(&class_context.jvms_file);
    let method_context = MethodNodeContext { jvms_file, method_info: Rc::clone(&method_info) };
    let method_node = MethodNode {
        access: method_info.access_flags,
        name: class_context.read_utf8(method_info.name_index)?,
        desc: class_context.read_utf8(method_info.descriptor_index)?,
    };
    Ok(method_node)
}
