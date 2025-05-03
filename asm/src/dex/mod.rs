pub use raw::*;
pub mod element;

use crate::dex::element::{AsElement, ClassContentElement};
use crate::impls::jvms::r::{ReadContext, U32BasedSize};
use crate::impls::ToArc;
use crate::smali::SmaliNode;
use crate::{AsmErr, AsmResult};
pub use constant::*;
use std::io::Read;
pub use util::*;

pub mod insn;
pub mod insn_syntax;

mod raw;
mod constant;
mod util;

pub type Opcode = u8;

impl DexFile {
    pub fn resolve_from_read<T: Read>(reader: T) -> AsmResult<Self> {
        let mut reader = reader;
        let mut bytes = vec![];
        reader.read_to_end(&mut bytes)
            .map_err(|e| AsmErr::IOReadErr(e.arc()))?;
        Self::resolve_from_bytes(&bytes)
    }
    pub fn resolve_from_bytes(bytes: &[u8]) -> AsmResult<Self> {
        let mut context = ReadContext::little_endian(bytes);
        context.read()
    }
}

pub struct DexFileAccessor {
    pub file: DexFile,
    pub bytes: Vec<u8>,
    pub endian: bool,
    pub call_site_ids: Vec<CallSiteId>,
    pub method_handles: Vec<MethodHandle>,
}

impl DexFileAccessor {
    pub fn new(file: DexFile, bytes: Vec<u8>) -> Self {
        let endian = file.header.endian_tag == Header::BIG_ENDIAN_TAG;
        let map_list = Self::get_map_list(&bytes, &file.header, endian)
            .unwrap_or_default();
        let mut call_site_off = 0u32;
        let mut call_site_size = U32BasedSize::default();
        let mut method_handle_off = 0u32;
        let mut method_handle_size = U32BasedSize::default();
        for map_item in map_list.items {
            match map_item.type_value {
                MapListTypeConst::TYPE_CALL_SITE_ID_ITEM => {
                    call_site_off = map_item.offset;
                    call_site_size = map_item.size;
                }
                MapListTypeConst::TYPE_METHOD_HANDLE_ITEM => {
                    method_handle_off = map_item.offset;
                    method_handle_size = map_item.size;
                }
                _ => {}
            }
        }
        let call_site_ids = Self::get_call_site_ids(&bytes, call_site_off, call_site_size, endian);
        let method_handles = Self::get_method_handles(&bytes, method_handle_off, method_handle_size, endian);
        Self { file, bytes, endian, call_site_ids, method_handles }
    }

    pub fn get_class_element(&self, class_data_off: DUInt) -> AsmResult<ClassContentElement> {
        self.get_data_impl::<ClassDataItem>(class_data_off)?.to_element(&self)
    }

    pub fn get_class_smali(&self, class_def: ClassDef) -> AsmResult<SmaliNode> {
        class_def.to_smali(&self)
    }

    pub fn get_code_item(&self, code_off: DUInt) -> AsmResult<Option<CodeItem>> {
        if code_off == 0 { return Ok(None); }
        self.get_data_impl(code_off).map(Some)
    }
}
