use crate::impls::apk_load::ApkAccessor;
use enum_dispatch::enum_dispatch;
use java_asm::smali::SmaliNode;
use java_asm::{DescriptorRef, StrRef};
use parking_lot::Mutex;
use std::sync::Arc;

pub mod server;

pub(crate) mod impls;
pub mod ui;

// the server contains all information for single opened file.
#[derive(Clone)]
pub struct AsmServer {
    pub loading_state: LoadingState,
    // when in loading state, the accessor is None.
    pub accessor: AccessorMut,
}

pub type ServerMut = Arc<Mutex<Option<AsmServer>>>;
type AccessorMut = Arc<Mutex<Option<AccessorEnum>>>;

#[derive(Default, Clone, Debug)]
pub struct LoadingState {
    pub in_loading: bool,
    // file loading progress, 0.0 ~ 1.0
    pub loading_progress: f32,
    // when loading failed, the error will be set.
    pub err: Option<String>,
}

#[enum_dispatch]
pub enum AccessorEnum {
    Apk(ApkAccessor),
}

#[enum_dispatch(AccessorEnum)]
pub trait Accessor {
    fn read_classes(&self) -> Vec<StrRef>;

    /// return true if the class exists.
    /// the format of class_key is [DescriptorRef]
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

