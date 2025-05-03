use crate::dex::{DUInt, DexFileAccessor};
use crate::AsmResult;
use crate::{DescriptorRef, StrRef};

pub trait AsElement<E> {
    fn to_element(&self, accessor: &DexFileAccessor, previous_index: u32) -> AsmResult<(u32, E)>;
}

#[derive(Clone, Debug)]
pub struct ClassContentElement {
    pub static_fields: Vec<FieldElement>,
    pub instance_fields: Vec<FieldElement>,
    pub direct_methods: Vec<MethodElement>,
    pub virtual_methods: Vec<MethodElement>,
}

#[derive(Clone, Debug)]
pub struct FieldElement {
    pub access_flags: u32,
    pub name: StrRef,
    pub descriptor: DescriptorRef,
}

#[derive(Clone, Debug)]
pub struct MethodElement {
    pub access_flags: u32,
    pub name: StrRef,
    pub shorty_descriptor: DescriptorRef,
    pub return_type: DescriptorRef,
    pub parameters: Vec<DescriptorRef>,
    pub code_off: DUInt,
}
