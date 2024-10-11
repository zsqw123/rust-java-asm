pub mod elements;

pub use constant::*;
pub use util::*;
use crate::dex::elements::DexFile;

pub mod insn;
pub mod insn_syntax;

mod constant;
mod util;

pub type Opcode = u8;

impl DexFile {
    pub fn resolve_from_bytes(bytes: &[u8]) -> Self {
        unimplemented!()
    }
}
