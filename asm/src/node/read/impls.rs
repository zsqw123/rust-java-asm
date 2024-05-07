use java_asm_internal::err::AsmResult;
use crate::jvms::element::{ClassFile, MethodInfo};
use crate::node::element::{ClassNode, MethodNode};
use crate::node::read::node_reader::{ClassNodeContext, MethodNodeContext};

pub struct ClassNodeFactory {}

impl ClassNodeFactory {
    pub fn from_jvms(jvms_file: ClassFile) -> AsmResult<ClassNode> {
        let class_context = ClassNodeContext { jvms_file: &jvms_file };
        let class_node = ClassNode {
            minor_version: jvms_file.minor_version,
            major_version: jvms_file.major_version,
            access: jvms_file.access_flags,
            name: jvms_file.this_class,
        };
        Ok(class_node)
    }
}

fn method_from_jvms(class_context: &ClassNodeContext, method_info: &MethodInfo) -> AsmResult<MethodNode> {
    let method_context = MethodNodeContext { jvms_file: class_context.jvms_file, method_info };
    let method_node = MethodNode {
        access: method_info.access_flags,
        name: method_info.name,
        descriptor: method_info.descriptor,
    };
    Ok(method_node)
}
