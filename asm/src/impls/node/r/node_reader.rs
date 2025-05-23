use crate::err::AsmErr;
use crate::jvms::element::ClassFile;
use crate::node::element::BootstrapMethodAttr;
use crate::node::values::ConstValue;
use crate::ComputableSizedVec;
use std::cell::OnceCell;
use std::fmt::Display;
use std::sync::Arc;

pub struct ConstPool {
    pub jvms_file: Arc<ClassFile>,
    pub pool: ComputableSizedVec<ConstValue>,
}

pub(crate) struct ClassNodeContext {
    pub jvms_file: Arc<ClassFile>,
    pub cp: Arc<ConstPool>,
    pub bootstrap_methods: OnceCell<Vec<BootstrapMethodAttr>>,
}

impl ClassNodeContext {
    pub fn new(jvms_file: Arc<ClassFile>) -> ClassNodeContext {
        let const_pool_size = jvms_file.constant_pool.len();
        let const_pool = ConstPool {
            jvms_file: Arc::clone(&jvms_file),
            pool: ComputableSizedVec::new(const_pool_size),
        };
        // attrs need to be read entirely, because we need to traverse the attributes
        // when constructing the class node, we just uses LazyCell for read it lazily.
        let cp = Arc::new(const_pool);
        let bootstrap_methods = OnceCell::default();
        ClassNodeContext {
            jvms_file: Arc::clone(&jvms_file),
            cp, bootstrap_methods,
        }
    }

    // unsafe function, but only for internal use.
    // we will ensure that bootstrap methods are correctly initialized before using it.
    #[inline]
    pub fn require_bms(&self) -> &Vec<BootstrapMethodAttr> {
        self.bootstrap_methods.get().expect("bootstrap methods not initialized yet.")
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
