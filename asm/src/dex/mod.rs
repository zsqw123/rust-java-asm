pub mod raw;
pub mod element;

use crate::dex::raw::{ClassDef, DexFile};
use crate::impls::ToRc;
use crate::{AsmErr, AsmResult};
pub use constant::*;
use std::io::Read;
pub use util::*;
use crate::impls::jvms::r::ReadContext;

pub mod insn;
pub mod insn_syntax;

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

pub trait DexAccessor {
    fn get_classes(&self) -> Vec<ClassDef>;
}
