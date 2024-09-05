use std::cell::LazyCell;
use std::fmt::Display;
use std::rc::Rc;

use java_asm_internal::err::{AsmErr, AsmResult};

use crate::impls::computable::ComputableMap;
use crate::jvms::element::{AttributeInfo, ClassFile, MethodInfo};
use crate::node::element::Attribute;
use crate::node::values::ConstValue;

pub struct ConstPool {
    pub jvms_file: Rc<ClassFile>,
    pub pool: Rc<ConstComputableMap>,
}

pub(crate) struct ClassNodeContext<AttrFn = fn() -> Attrs> {
    pub jvms_file: Rc<ClassFile>,
    pub cp: Rc<ConstPool>,
    pub attrs: LazyCell<Attrs, AttrFn>,
}

pub type Attrs = AsmResult<Vec<(AttributeInfo, Attribute)>>;
pub type ConstComputableMap = ComputableMap<u16, ConstValue, AsmErr>;

impl ClassNodeContext {
    pub fn new(jvms_file: Rc<ClassFile>) -> ClassNodeContext<impl FnOnce() -> Attrs> {
        let const_pool = ConstPool {
            jvms_file: Rc::clone(&jvms_file),
            pool: Default::default(),
        };
        // attrs need to be read entirely, because we need to traverse the attributes
        // when constructing the class node, we just uses LazyCell for read it lazily.
        let cp = Rc::new(const_pool);
        let cp_for_attr = Rc::clone(&cp);
        let attrs = LazyCell::new(move || {
            Rc::clone(&cp_for_attr);
            todo!()
        });
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
