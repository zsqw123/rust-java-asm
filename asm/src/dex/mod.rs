pub mod elements;

pub use constant::*;
pub use util::*;

pub mod insn;
pub mod insn_syntax;

mod constant;
mod util;

pub type Opcode = u8;
