pub mod method;
pub mod class;
pub mod opcodes;
pub mod field;
pub mod annotation;
pub(crate) mod internal;
pub mod attribute;
pub mod label;
pub mod handle;
pub mod type_path;
pub mod constant_dynamic;
pub mod constants;
pub mod asm_type;

/// jvms api for read & write bytecode.
/// - [JVMS4](https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html)
pub mod jvms;

/// node api for read & write bytecode. 
/// Quite similar with [ASM Tree API](https://asm.ow2.io/javadoc/org/objectweb/asm/tree/package-summary.html)
pub mod node;
