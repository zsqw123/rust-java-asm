use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::rc::Rc;

use java_asm_internal::err::{AsmErr, AsmResult};

use crate::impls::computable::ComputableMap;
use crate::jvms::element::{ClassFile, MethodInfo};
use crate::node::values::ConstValue;

pub struct CpCache {
    pub jvms_file: Rc<ClassFile>,
    pub pool: HashMap<u16, Rc<ConstValue>>,
}

pub(crate) struct ClassNodeContext {
    pub jvms_file: Rc<ClassFile>,
    pub const_pool: ConstComputableMap,
    pub cp_cache: CpCache,
}

type ConstFn = fn(&u16) -> AsmResult<ConstValue>;
type ConstComputableMap = ComputableMap<u16, AsmResult<ConstValue>, ConstFn>;

impl ClassNodeContext {
    pub fn new(jvms_file: Rc<ClassFile>) -> Self {
        let const_pool = ComputableMap::new(|index| {
            let cp_info = &jvms_file.cp_info[index as usize];
            let cp = Rc::new(cp_info.to_const()?);
            Ok(cp)
        });
        Self {
            jvms_file: Rc::clone(&jvms_file),
            cp_cache: CpCache {
                jvms_file,
                pool: HashMap::new(),
            },
        }
    }
}

impl CpCache {
    pub(crate) fn err<D: Display>(&mut self, msg: D) -> AsmErr {
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
