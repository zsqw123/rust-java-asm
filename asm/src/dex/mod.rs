pub use raw::*;
pub mod element;

use crate::impls::jvms::r::ReadContext;
use crate::impls::ToRc;
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
            .map_err(|e| AsmErr::IOReadErr(e.rc()))?;
        Self::resolve_from_bytes(&bytes)
    }
    pub fn resolve_from_bytes(bytes: &[u8]) -> AsmResult<Self> {
        let mut context = ReadContext::little_endian(bytes);
        context.read()
    }
}

pub struct DexFileAccessor<'a> {
    pub file: DexFile,
    pub bytes: &'a [u8],
}

impl<'a> DexFileAccessor<'a> {
    pub fn get_class_data(&self, class_data_off: DUInt) -> AsmResult<ClassDataItem> {
        self.get_data_impl(class_data_off)
    }

    pub fn get_code_item(&self, code_off: DUInt) -> AsmResult<CodeItem> {
        self.get_data_impl(code_off)
    }
}
