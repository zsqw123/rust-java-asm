pub use computable::*;
pub use constants::*;
pub use err::*;
pub use opcodes::*;
pub use pub_jvms_utils::*;
pub use pub_refs::*;

pub mod opcodes;
pub mod constants;

/// jvms api for read & write bytecode.
/// - [JVMS4](https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html)
pub mod jvms;

/// node api for read & write bytecode. 
/// Quite similar with [ASM Tree API](https://asm.ow2.io/javadoc/org/objectweb/asm/tree/package-summary.html)
pub mod node;

/// dex interop for read dalvik bytecode.
/// - [DEX Format](https://source.android.com/docs/core/runtime/dex-format)
pub mod dex;

mod err;
mod pub_jvms_utils;
mod pub_refs;
mod computable;


pub(crate) mod impls;
mod smali;

