use std::cell::RefCell;
use java_asm::smali::SmaliNode;
use java_asm::{DescriptorRef, StrRef};

pub mod server;

pub(crate) mod impls;

/// [A] should be a type that implements [Accessor]
pub struct AsmServer<A> {
    accessor: RefCell<A>,
}

pub trait Accessor {
    fn ensure_initialized(&mut self) -> Self;
    fn read_classes(&self) -> Vec<StrRef>;
    fn exist_class(&self, class_key: &str) -> bool;
    fn read_content(&self, class_key: &str) -> SmaliNode;
}

pub struct MethodMeta {
    pub class_key: StrRef,
    pub name: StrRef,
    pub descriptor: DescriptorRef,
}

pub struct FieldMeta {
    pub class_key: StrRef,
    pub name: StrRef,
}

