# java-asm

Java bytecode reader & writer, a rust implementation for [ASM](https://gitlab.ow2.org/asm/asm)

There are some similar projects in GitHub, but they didn't actually implement all ASM nodes/features in rust. 
So I want to build this library to fully read Java bytecode information.

This project supports much newer Java version(Java 21 currently) than other rust implementations. Only supports 
`asm-tree` api currently, not supports visitor api because Tree API is much easier to use than visitor api.

---

Some similar projects:

- [rjvm](https://github.com/andreabergia/rjvm)
  - read jvm bytecode and run it in a rust vm
  - support JVM7
- [jvm-assembler](https://github.com/kenpratt/jvm-assembler)
- [Ka-Pi](https://github.com/ChAoSUnItY/Ka-Pi)
- [cfsp](https://github.com/ChAoSUnItY/cfsp)

