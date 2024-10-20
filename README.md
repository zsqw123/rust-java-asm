# java-asm

[![Crates.io Version](https://img.shields.io/crates/v/java_asm)](https://crates.io/crates/java_asm)

Java bytecode reader & writer, maybe a rust implementation for [ASM](https://gitlab.ow2.org/asm/asm)

There are some similar projects in GitHub, but they didn't actually implement all JVM Bytecode format, and also not
implements all ASM nodes/features in rust. 
So I want to build this library to fully read and write Java bytecode information.

This project supports much newer LTS Java version(Java 21 currently) than other rust implementations. Only supports 
`asm-tree` api currently, not supports visitor api because Tree API is much easier to use than visitor api.

## Current Stage

After version 0.0.6, you can try to use `ClassNode::from_jvms` to read a class file into a `ClassNode`, 
and it is pretty useful to now, check [tests](asm/tests/node/read_test.rs) in this project to 
see some examples.

- [x] Implement **Read** Java class file with **[JVMS](https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html)** format
- [x] Implement **Write** Java class file with **[JVMS](https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html)** format
- [ ] **WIP**, `ClassNode` reader
  - [x] Constant pool
  - [x] Attributes
  - [x] Class / Field / Method metadata
  - [x] Method instructions
  - [ ] Method frames (read as an attribute is available, need a better format)
  - [ ] Method local variables / stacks / try-catches (read as an attribute is available, need a better format)
- [ ] **Not Start**, Nodes writer (low priority currently)
  - [ ] Append constant pool if needed
  - [ ] Write back attributes into Class / Field / Method / Code
  - [ ] Method frames
- [ ] **Not Start**, Implement ASM features (eg. auto calculate frame/stack etc.)
- [ ] Smali liked output.
- [ ] Dex interop. (https://source.android.com/docs/core/runtime/dex-format)
  - [x] Basic structure for dex file metadata
  - [ ] Instructions
  - [ ] Annotations
  - [ ] Debug info
  - [ ] Other formats
  - [ ] Separation for metadata with real data (e.g. instructions)
- [ ] Isolate AsmResult as multiple different errors, and provide a better error message.

### Goals

1. Fully read and write Java class file with JVMS format.
2. Partially implement ASM features in rust, but provides some better operations in rust.
3. Support much newer Java version (higher priority for LTS, Java 21 currently).
4. Not depends on any of other rust libraries at runtime, build everything from std only. (but some proc marco's
   dependencies are used for generate some template codes. e.g. `quote` and `syn`)

---

Some similar projects:

- [rjvm](https://github.com/andreabergia/rjvm)
  - read jvm bytecode and run it in a rust vm
  - support JVM7
- [jvm-assembler](https://github.com/kenpratt/jvm-assembler)
- [Ka-Pi](https://github.com/ChAoSUnItY/Ka-Pi)
- [cfsp](https://github.com/ChAoSUnItY/cfsp)

