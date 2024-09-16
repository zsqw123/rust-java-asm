/**
 * The JVM opcodes, access flags and array type codes. This interface does not define all the JVM
 * opcodes because some opcodes are automatically handled. For example, the xLOAD and xSTORE opcodes
 * are automatically replaced by xLOAD_n and xSTORE_n opcodes when possible. The xLOAD_n and
 * xSTORE_n opcodes are therefore not defined in this interface. Likewise for LDC, automatically
 * replaced by LDC_W or LDC2_W when necessary, WIDE, GOTO_W and JSR_W.
 *
 * @see <a href="https://docs.oracle.com/javase/specs/jvms/se11/html/jvms-6.html">JVMS 6</a>
 */
pub struct Opcodes;

pub struct OpcodesConsts;

// versions are temporary unused
// pub enum ASMVersion {
//     ASM4, ASM5, ASM6,
//     ASM7, ASM8, ASM9,
//     ASM10Experimental,
// }

impl Opcodes {
    // Java ClassFile versions (the minor version is stored in the 16 most significant bits, and the
    // major version in the 16 least significant bits).
    pub const V1_1: u32 = 3 << 16 | 45;
    pub const V1_2: u32 = 0 << 16 | 46;
    pub const V1_3: u32 = 0 << 16 | 47;
    pub const V1_4: u32 = 0 << 16 | 48;
    pub const V1_5: u32 = 0 << 16 | 49;
    pub const V1_6: u32 = 0 << 16 | 50;
    pub const V1_7: u32 = 0 << 16 | 51;
    pub const V1_8: u32 = 0 << 16 | 52;
    pub const V9: u32 = 0 << 16 | 53;
    pub const V10: u32 = 0 << 16 | 54;
    pub const V11: u32 = 0 << 16 | 55;
    pub const V12: u32 = 0 << 16 | 56;
    pub const V13: u32 = 0 << 16 | 57;
    pub const V14: u32 = 0 << 16 | 58;
    pub const V15: u32 = 0 << 16 | 59;
    pub const V16: u32 = 0 << 16 | 60;
    pub const V17: u32 = 0 << 16 | 61;
    pub const V18: u32 = 0 << 16 | 62;
    pub const V19: u32 = 0 << 16 | 63;
    pub const V20: u32 = 0 << 16 | 64;
    pub const V21: u32 = 0 << 16 | 65;
    pub const V22: u32 = 0 << 16 | 66;
    pub const V23: u32 = 0 << 16 | 67;

    /**
     * Version flag indicating that the class is using 'preview' features.
     *
     * <p>{@code version & V_PREVIEW == V_PREVIEW} tests if a version is flagged with {@code
     * V_PREVIEW}.
     */
    pub const V_PREVIEW: u32 = 0xFFFF0000;

    // Access flags values, defined in
    // - https://docs.oracle.com/javase/specs/jvms/se9/html/jvms-4.html#jvms-4.1-200-E.1
    // - https://docs.oracle.com/javase/specs/jvms/se9/html/jvms-4.html#jvms-4.5-200-A.1
    // - https://docs.oracle.com/javase/specs/jvms/se9/html/jvms-4.html#jvms-4.6-200-A.1
    // - https://docs.oracle.com/javase/specs/jvms/se9/html/jvms-4.html#jvms-4.7.25
    pub const ACC_PUBLIC: u16 = 0x0001; // class, field, method
    pub const ACC_PRIVATE: u16 = 0x0002; // class, field, method
    pub const ACC_PROTECTED: u16 = 0x0004; // class, field, method
    pub const ACC_STATIC: u16 = 0x0008; // field, method
    pub const ACC_FINAL: u16 = 0x0010; // class, field, method, parameter
    pub const ACC_SUPER: u16 = 0x0020; // class
    pub const ACC_SYNCHRONIZED: u16 = 0x0020; // method
    pub const ACC_OPEN: u16 = 0x0020; // module
    pub const ACC_TRANSITIVE: u16 = 0x0020; // module requires
    pub const ACC_VOLATILE: u16 = 0x0040; // field
    pub const ACC_BRIDGE: u16 = 0x0040; // method
    pub const ACC_STATIC_PHASE: u16 = 0x0040; // module requires
    pub const ACC_VARARGS: u16 = 0x0080; // method
    pub const ACC_TRANSIENT: u16 = 0x0080; // field
    pub const ACC_NATIVE: u16 = 0x0100; // method
    pub const ACC_INTERFACE: u16 = 0x0200; // class
    pub const ACC_ABSTRACT: u16 = 0x0400; // class, method
    pub const ACC_STRICT: u16 = 0x0800; // method
    pub const ACC_SYNTHETIC: u16 = 0x1000; // class, field, method, parameter, module *
    pub const ACC_ANNOTATION: u16 = 0x2000; // class
    pub const ACC_ENUM: u16 = 0x4000; // class(?) field inner
    pub const ACC_MANDATED: u16 = 0x8000; // field, method, parameter, module, module *
    pub const ACC_MODULE: u16 = 0x8000; // class

    // Possible values for the type operand of the NEWARRAY instruction.
    // See https://docs.oracle.com/javase/specs/jvms/se9/html/jvms-6.html#jvms-6.5.newarray.
    pub const T_BOOLEAN: u8 = 4;
    pub const T_CHAR: u8 = 5;
    pub const T_FLOAT: u8 = 6;
    pub const T_DOUBLE: u8 = 7;
    pub const T_BYTE: u8 = 8;
    pub const T_SHORT: u8 = 9;
    pub const T_INT: u8 = 10;
    pub const T_LONG: u8 = 11;

    // Possible values for the reference_kind field of CONSTANT_MethodHandle_info structures.
    // See https://docs.oracle.com/javase/specs/jvms/se9/html/jvms-4.html#jvms-4.4.8.
    pub const H_GETFIELD: u8 = 1;
    pub const H_GETSTATIC: u8 = 2;
    pub const H_PUTFIELD: u8 = 3;
    pub const H_PUTSTATIC: u8 = 4;
    pub const H_INVOKEVIRTUAL: u8 = 5;
    pub const H_INVOKESTATIC: u8 = 6;
    pub const H_INVOKESPECIAL: u8 = 7;
    pub const H_NEWINVOKESPECIAL: u8 = 8;
    pub const H_INVOKEINTERFACE: u8 = 9;

    // The JVM opcode values (with the MethodVisitor method name used to visit them in comment, and
    // where '-' means 'same method name as on the previous line').
    // See https://docs.oracle.com/javase/specs/jvms/se9/html/jvms-6.html.
    pub const NOP: u8 = 0; // visitInsn
    pub const ACONST_NULL: u8 = 1; // -
    pub const ICONST_M1: u8 = 2; // -
    pub const ICONST_0: u8 = 3; // -
    pub const ICONST_1: u8 = 4; // -
    pub const ICONST_2: u8 = 5; // -
    pub const ICONST_3: u8 = 6; // -
    pub const ICONST_4: u8 = 7; // -
    pub const ICONST_5: u8 = 8; // -
    pub const LCONST_0: u8 = 9; // -
    pub const LCONST_1: u8 = 10; // -
    pub const FCONST_0: u8 = 11; // -
    pub const FCONST_1: u8 = 12; // -
    pub const FCONST_2: u8 = 13; // -
    pub const DCONST_0: u8 = 14; // -
    pub const DCONST_1: u8 = 15; // -
    pub const BIPUSH: u8 = 16; // visitIntInsn
    pub const SIPUSH: u8 = 17; // -
    pub const LDC: u8 = 18; // visitLdcInsn
    pub const LDC_W: u8 = 19; // -
    pub const LDC2_W: u8 = 20; // -
    pub const ILOAD: u8 = 21; // visitVarInsn
    pub const LLOAD: u8 = 22; // -
    pub const FLOAD: u8 = 23; // -
    pub const DLOAD: u8 = 24; // -
    pub const ALOAD: u8 = 25; // -
    pub const ILOAD_0: u8 = 26;
    pub const ILOAD_1: u8 = 27;
    pub const ILOAD_2: u8 = 28;
    pub const ILOAD_3: u8 = 29;
    pub const LLOAD_0: u8 = 30;
    pub const LLOAD_1: u8 = 31;
    pub const LLOAD_2: u8 = 32;
    pub const LLOAD_3: u8 = 33;
    pub const FLOAD_0: u8 = 34;
    pub const FLOAD_1: u8 = 35;
    pub const FLOAD_2: u8 = 36;
    pub const FLOAD_3: u8 = 37;
    pub const DLOAD_0: u8 = 38;
    pub const DLOAD_1: u8 = 39;
    pub const DLOAD_2: u8 = 40;
    pub const DLOAD_3: u8 = 41;
    pub const ALOAD_0: u8 = 42;
    pub const ALOAD_1: u8 = 43;
    pub const ALOAD_2: u8 = 44;
    pub const ALOAD_3: u8 = 45;
    pub const IALOAD: u8 = 46; // visitInsn
    pub const LALOAD: u8 = 47; // -
    pub const FALOAD: u8 = 48; // -
    pub const DALOAD: u8 = 49; // -
    pub const AALOAD: u8 = 50; // -
    pub const BALOAD: u8 = 51; // -
    pub const CALOAD: u8 = 52; // -
    pub const SALOAD: u8 = 53; // -
    pub const ISTORE: u8 = 54; // visitVarInsn
    pub const LSTORE: u8 = 55; // -
    pub const FSTORE: u8 = 56; // -
    pub const DSTORE: u8 = 57; // -
    pub const ASTORE: u8 = 58; // -
    pub const ISTORE_0: u8 = 59;
    pub const ISTORE_1: u8 = 60;
    pub const ISTORE_2: u8 = 61;
    pub const ISTORE_3: u8 = 62;
    pub const LSTORE_0: u8 = 63;
    pub const LSTORE_1: u8 = 64;
    pub const LSTORE_2: u8 = 65;
    pub const LSTORE_3: u8 = 66;
    pub const FSTORE_0: u8 = 67;
    pub const FSTORE_1: u8 = 68;
    pub const FSTORE_2: u8 = 69;
    pub const FSTORE_3: u8 = 70;
    pub const DSTORE_0: u8 = 71;
    pub const DSTORE_1: u8 = 72;
    pub const DSTORE_2: u8 = 73;
    pub const DSTORE_3: u8 = 74;
    pub const ASTORE_0: u8 = 75;
    pub const ASTORE_1: u8 = 76;
    pub const ASTORE_2: u8 = 77;
    pub const ASTORE_3: u8 = 78;
    pub const IASTORE: u8 = 79; // visitInsn
    pub const LASTORE: u8 = 80; // -
    pub const FASTORE: u8 = 81; // -
    pub const DASTORE: u8 = 82; // -
    pub const AASTORE: u8 = 83; // -
    pub const BASTORE: u8 = 84; // -
    pub const CASTORE: u8 = 85; // -
    pub const SASTORE: u8 = 86; // -
    pub const POP: u8 = 87; // -
    pub const POP2: u8 = 88; // -
    pub const DUP: u8 = 89; // -
    pub const DUP_X1: u8 = 90; // -
    pub const DUP_X2: u8 = 91; // -
    pub const DUP2: u8 = 92; // -
    pub const DUP2_X1: u8 = 93; // -
    pub const DUP2_X2: u8 = 94; // -
    pub const SWAP: u8 = 95; // -
    pub const IADD: u8 = 96; // -
    pub const LADD: u8 = 97; // -
    pub const FADD: u8 = 98; // -
    pub const DADD: u8 = 99; // -
    pub const ISUB: u8 = 100; // -
    pub const LSUB: u8 = 101; // -
    pub const FSUB: u8 = 102; // -
    pub const DSUB: u8 = 103; // -
    pub const IMUL: u8 = 104; // -
    pub const LMUL: u8 = 105; // -
    pub const FMUL: u8 = 106; // -
    pub const DMUL: u8 = 107; // -
    pub const IDIV: u8 = 108; // -
    pub const LDIV: u8 = 109; // -
    pub const FDIV: u8 = 110; // -
    pub const DDIV: u8 = 111; // -
    pub const IREM: u8 = 112; // -
    pub const LREM: u8 = 113; // -
    pub const FREM: u8 = 114; // -
    pub const DREM: u8 = 115; // -
    pub const INEG: u8 = 116; // -
    pub const LNEG: u8 = 117; // -
    pub const FNEG: u8 = 118; // -
    pub const DNEG: u8 = 119; // -
    pub const ISHL: u8 = 120; // -
    pub const LSHL: u8 = 121; // -
    pub const ISHR: u8 = 122; // -
    pub const LSHR: u8 = 123; // -
    pub const IUSHR: u8 = 124; // -
    pub const LUSHR: u8 = 125; // -
    pub const IAND: u8 = 126; // -
    pub const LAND: u8 = 127; // -
    pub const IOR: u8 = 128; // -
    pub const LOR: u8 = 129; // -
    pub const IXOR: u8 = 130; // -
    pub const LXOR: u8 = 131; // -
    pub const IINC: u8 = 132; // visitIincInsn
    pub const I2L: u8 = 133; // visitInsn
    pub const I2F: u8 = 134; // -
    pub const I2D: u8 = 135; // -
    pub const L2I: u8 = 136; // -
    pub const L2F: u8 = 137; // -
    pub const L2D: u8 = 138; // -
    pub const F2I: u8 = 139; // -
    pub const F2L: u8 = 140; // -
    pub const F2D: u8 = 141; // -
    pub const D2I: u8 = 142; // -
    pub const D2L: u8 = 143; // -
    pub const D2F: u8 = 144; // -
    pub const I2B: u8 = 145; // -
    pub const I2C: u8 = 146; // -
    pub const I2S: u8 = 147; // -
    pub const LCMP: u8 = 148; // -
    pub const FCMPL: u8 = 149; // -
    pub const FCMPG: u8 = 150; // -
    pub const DCMPL: u8 = 151; // -
    pub const DCMPG: u8 = 152; // -
    pub const IFEQ: u8 = 153; // visitJumpInsn
    pub const IFNE: u8 = 154; // -
    pub const IFLT: u8 = 155; // -
    pub const IFGE: u8 = 156; // -
    pub const IFGT: u8 = 157; // -
    pub const IFLE: u8 = 158; // -
    pub const IF_ICMPEQ: u8 = 159; // -
    pub const IF_ICMPNE: u8 = 160; // -
    pub const IF_ICMPLT: u8 = 161; // -
    pub const IF_ICMPGE: u8 = 162; // -
    pub const IF_ICMPGT: u8 = 163; // -
    pub const IF_ICMPLE: u8 = 164; // -
    pub const IF_ACMPEQ: u8 = 165; // -
    pub const IF_ACMPNE: u8 = 166; // -
    pub const GOTO: u8 = 167; // -
    pub const JSR: u8 = 168; // -
    pub const RET: u8 = 169; // visitVarInsn
    pub const TABLESWITCH: u8 = 170; // visiTableSwitchInsn
    pub const LOOKUPSWITCH: u8 = 171; // visitLookupSwitch
    pub const IRETURN: u8 = 172; // visitInsn
    pub const LRETURN: u8 = 173; // -
    pub const FRETURN: u8 = 174; // -
    pub const DRETURN: u8 = 175; // -
    pub const ARETURN: u8 = 176; // -
    pub const RETURN: u8 = 177; // -
    pub const GETSTATIC: u8 = 178; // visitFieldInsn
    pub const PUTSTATIC: u8 = 179; // -
    pub const GETFIELD: u8 = 180; // -
    pub const PUTFIELD: u8 = 181; // -
    pub const INVOKEVIRTUAL: u8 = 182; // visitMethodInsn
    pub const INVOKESPECIAL: u8 = 183; // -
    pub const INVOKESTATIC: u8 = 184; // -
    pub const INVOKEINTERFACE: u8 = 185; // -
    pub const INVOKEDYNAMIC: u8 = 186; // visitInvokeDynamicInsn
    pub const NEW: u8 = 187; // visitTypeInsn
    pub const NEWARRAY: u8 = 188; // visitIntInsn
    pub const ANEWARRAY: u8 = 189; // visitTypeInsn
    pub const ARRAYLENGTH: u8 = 190; // visitInsn
    pub const ATHROW: u8 = 191; // -
    pub const CHECKCAST: u8 = 192; // visitTypeInsn
    pub const INSTANCEOF: u8 = 193; // -
    pub const MONITORENTER: u8 = 194; // visitInsn
    pub const MONITOREXIT: u8 = 195; // -
    pub const WIDE: u8 = 196;
    pub const MULTIANEWARRAY: u8 = 197; // visitMultiANewArrayInsn
    pub const IFNULL: u8 = 198; // visitJumpInsn
    pub const IFNONNULL: u8 = 199; // -
    pub const GOTO_W: u8 = 200;
    pub const JSR_W: u8 = 201;
}

impl OpcodesConsts {
    pub const LDC_W: u8 = 19;
    pub const LDC2_W: u8 = 20;
    pub const ILOAD_0: u8 = 26;
    pub const ILOAD_1: u8 = 27;
    pub const ILOAD_2: u8 = 28;
    pub const ILOAD_3: u8 = 29;
    pub const LLOAD_0: u8 = 30;
    pub const LLOAD_1: u8 = 31;
    pub const LLOAD_2: u8 = 32;
    pub const LLOAD_3: u8 = 33;
    pub const FLOAD_0: u8 = 34;
    pub const FLOAD_1: u8 = 35;
    pub const FLOAD_2: u8 = 36;
    pub const FLOAD_3: u8 = 37;
    pub const DLOAD_0: u8 = 38;
    pub const DLOAD_1: u8 = 39;
    pub const DLOAD_2: u8 = 40;
    pub const DLOAD_3: u8 = 41;
    pub const ALOAD_0: u8 = 42;
    pub const ALOAD_1: u8 = 43;
    pub const ALOAD_2: u8 = 44;
    pub const ALOAD_3: u8 = 45;
    pub const ISTORE_0: u8 = 59;
    pub const ISTORE_1: u8 = 60;
    pub const ISTORE_2: u8 = 61;
    pub const ISTORE_3: u8 = 62;
    pub const LSTORE_0: u8 = 63;
    pub const LSTORE_1: u8 = 64;
    pub const LSTORE_2: u8 = 65;
    pub const LSTORE_3: u8 = 66;
    pub const FSTORE_0: u8 = 67;
    pub const FSTORE_1: u8 = 68;
    pub const FSTORE_2: u8 = 69;
    pub const FSTORE_3: u8 = 70;
    pub const DSTORE_0: u8 = 71;
    pub const DSTORE_1: u8 = 72;
    pub const DSTORE_2: u8 = 73;
    pub const DSTORE_3: u8 = 74;
    pub const ASTORE_0: u8 = 75;
    pub const ASTORE_1: u8 = 76;
    pub const ASTORE_2: u8 = 77;
    pub const ASTORE_3: u8 = 78;
}
