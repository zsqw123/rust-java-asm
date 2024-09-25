pub use constants::*;
pub use err::*;
pub use jvms::*;
pub use node::*;
pub use opcodes::*;

pub mod opcodes;
pub mod constants;

/// jvms api for read & write bytecode.
/// - [JVMS4](https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html)
pub mod jvms;

/// node api for read & write bytecode. 
/// Quite similar with [ASM Tree API](https://asm.ow2.io/javadoc/org/objectweb/asm/tree/package-summary.html)
pub mod node;
pub mod err;

pub(crate) mod impls;
