use crate::dex::CodeItem;
use crate::Computable;
use crate::{DescriptorRef, StrRef};

pub struct DexElement {
    
}

#[derive(Clone, Debug)]
pub struct ClassElement {
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
    pub code_item: Computable<CodeItem>,
}
