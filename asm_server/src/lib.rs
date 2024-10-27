use crate::impls::apk_load::ApkAccessor;
use enum_dispatch::enum_dispatch;
use java_asm::smali::SmaliNode;
use java_asm::{DescriptorRef, StrRef};

pub mod server;

pub(crate) mod impls;

pub struct AsmServer {
    accessor: AccessorEnum,
}

#[enum_dispatch]
pub enum AccessorEnum {
    Apk(ApkAccessor),
}

#[enum_dispatch(AccessorEnum)]
pub trait Accessor {
    fn read_classes(&self) -> Vec<StrRef>;
    fn exist_class(&self, class_key: &str) -> bool;
    fn read_content(&self, class_key: &str) -> Option<SmaliNode>;
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

