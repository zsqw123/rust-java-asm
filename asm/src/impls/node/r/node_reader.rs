use std::cell::OnceCell;
use std::fmt::Display;
use std::rc::Rc;

use crate::err::{AsmErr, AsmResult};

use crate::impls::computable::ComputableMap;
use crate::jvms::element::{AttributeInfo, ClassFile, MethodInfo};
use crate::node::element::Attribute;
use crate::node::values::ConstValue;

pub struct ConstPool {
    pub jvms_file: Rc<ClassFile>,
    pub pool: Rc<ConstComputableMap>,
}

pub(crate) struct ClassNodeContext {
    pub jvms_file: Rc<ClassFile>,
    pub cp: Rc<ConstPool>,
    pub attrs: OnceCell<Attrs>,
}

pub type Attrs = Rc<Vec<(AttributeInfo, Attribute)>>;
pub type ConstComputableMap = ComputableMap<u16, ConstValue, AsmErr>;

impl ClassNodeContext {
    pub fn new(jvms_file: Rc<ClassFile>) -> ClassNodeContext {
        let const_pool = ConstPool {
            jvms_file: Rc::clone(&jvms_file),
            pool: Default::default(),
        };
        // attrs need to be read entirely, because we need to traverse the attributes
        // when constructing the class node, we just uses LazyCell for read it lazily.
        let cp = Rc::new(const_pool);
        let attrs = OnceCell::default();
        ClassNodeContext {
            jvms_file: Rc::clone(&jvms_file),
            cp,
            attrs,
        }
    }
}


impl ConstPool {
    pub(crate) fn err<D: Display>(&self, msg: D) -> AsmErr {
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
