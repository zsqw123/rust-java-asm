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
    const V1_1: u32 = 3 << 16 | 45;
    const V1_2: u32 = 0 << 16 | 46;
    const V1_3: u32 = 0 << 16 | 47;
    const V1_4: u32 = 0 << 16 | 48;
    const V1_5: u32 = 0 << 16 | 49;
    const V1_6: u32 = 0 << 16 | 50;
    const V1_7: u32 = 0 << 16 | 51;
    const V1_8: u32 = 0 << 16 | 52;
    const V9: u32 = 0 << 16 | 53;
    const V10: u32 = 0 << 16 | 54;
    const V11: u32 = 0 << 16 | 55;
    const V12: u32 = 0 << 16 | 56;
    const V13: u32 = 0 << 16 | 57;
    const V14: u32 = 0 << 16 | 58;
    const V15: u32 = 0 << 16 | 59;
    const V16: u32 = 0 << 16 | 60;
    const V17: u32 = 0 << 16 | 61;
    const V18: u32 = 0 << 16 | 62;
    const V19: u32 = 0 << 16 | 63;
    const V20: u32 = 0 << 16 | 64;
    const V21: u32 = 0 << 16 | 65;
    const V22: u32 = 0 << 16 | 66;
    const V23: u32 = 0 << 16 | 67;

    /**
     * Version flag indicating that the class is using 'preview' features.
     *
     * <p>{@code version & V_PREVIEW == V_PREVIEW} tests if a version is flagged with {@code
     * V_PREVIEW}.
     */
    const V_PREVIEW: u32 = 0xFFFF0000;

    // Access flags values, defined in
    // - https://docs.oracle.com/javase/specs/jvms/se9/html/jvms-4.html#jvms-4.1-200-E.1
    // - https://docs.oracle.com/javase/specs/jvms/se9/html/jvms-4.html#jvms-4.5-200-A.1
    // - https://docs.oracle.com/javase/specs/jvms/se9/html/jvms-4.html#jvms-4.6-200-A.1
    // - https://docs.oracle.com/javase/specs/jvms/se9/html/jvms-4.html#jvms-4.7.25
    const ACC_PUBLIC: u16 = 0x0001; // class, field, method
    const ACC_PRIVATE: u16 = 0x0002; // class, field, method
    const ACC_PROTECTED: u16 = 0x0004; // class, field, method
    const ACC_STATIC: u16 = 0x0008; // field, method
    const ACC_FINAL: u16 = 0x0010; // class, field, method, parameter
    const ACC_SUPER: u16 = 0x0020; // class
    const ACC_SYNCHRONIZED: u16 = 0x0020; // method
    const ACC_OPEN: u16 = 0x0020; // module
    const ACC_TRANSITIVE: u16 = 0x0020; // module requires
    const ACC_VOLATILE: u16 = 0x0040; // field
    const ACC_BRIDGE: u16 = 0x0040; // method
    const ACC_STATIC_PHASE: u16 = 0x0040; // module requires
    const ACC_VARARGS: u16 = 0x0080; // method
    const ACC_TRANSIENT: u16 = 0x0080; // field
    const ACC_NATIVE: u16 = 0x0100; // method
    const ACC_INTERFACE: u16 = 0x0200; // class
    const ACC_ABSTRACT: u16 = 0x0400; // class, method
    const ACC_STRICT: u16 = 0x0800; // method
    const ACC_SYNTHETIC: u16 = 0x1000; // class, field, method, parameter, module *
    const ACC_ANNOTATION: u16 = 0x2000; // class
    const ACC_ENUM: u16 = 0x4000; // class(?) field inner
    const ACC_MANDATED: u16 = 0x8000; // field, method, parameter, module, module *
    const ACC_MODULE: u16 = 0x8000; // class

    // Possible values for the type operand of the NEWARRAY instruction.
    // See https://docs.oracle.com/javase/specs/jvms/se9/html/jvms-6.html#jvms-6.5.newarray.
    const T_BOOLEAN: u8 = 4;
    const T_CHAR: u8 = 5;
    const T_FLOAT: u8 = 6;
    const T_DOUBLE: u8 = 7;
    const T_BYTE: u8 = 8;
    const T_SHORT: u8 = 9;
    const T_INT: u8 = 10;
    const T_LONG: u8 = 11;

    // Possible values for the reference_kind field of CONSTANT_MethodHandle_info structures.
    // See https://docs.oracle.com/javase/specs/jvms/se9/html/jvms-4.html#jvms-4.4.8.
    const H_GETFIELD: u8 = 1;
    const H_GETSTATIC: u8 = 2;
    const H_PUTFIELD: u8 = 3;
    const H_PUTSTATIC: u8 = 4;
    const H_INVOKEVIRTUAL: u8 = 5;
    const H_INVOKESTATIC: u8 = 6;
    const H_INVOKESPECIAL: u8 = 7;
    const H_NEWINVOKESPECIAL: u8 = 8;
    const H_INVOKEINTERFACE: u8 = 9;

    // ASM specific stack map frame types, used in {@link ClassVisitor#visitFrame}.

    /** An expanded frame. See {@link ClassReader#EXPAND_FRAMES}. */
    const F_NEW: i8 = -1;

    /** A compressed frame with complete frame data. */
    const F_FULL: i8 = 0;

    /**
     * A compressed frame where locals are the same as the locals in the previous frame, except that
     * additional 1-3 locals are defined, and with an empty stack.
     */
    const F_APPEND: i8 = 1;

    /**
     * A compressed frame where locals are the same as the locals in the previous frame, except that
     * the last 1-3 locals are absent and with an empty stack.
     */
    const F_CHOP: i8 = 2;

    /**
     * A compressed frame with exactly the same locals as the previous frame and with an empty stack.
     */
    const F_SAME: i8 = 3;

    /**
     * A compressed frame with exactly the same locals as the previous frame and with a single value
     * on the stack.
     */
    const F_SAME1: i8 = 4;
    
    // Standard stack map frame element types, used in {@link ClassVisitor#visitFrame}.
    // todo


    // The JVM opcode values (with the MethodVisitor method name used to visit them in comment, and
    // where '-' means 'same method name as on the previous line').
    // See https://docs.oracle.com/javase/specs/jvms/se9/html/jvms-6.html.
    const NOP: u8 = 0; // visitInsn
    const ACONST_NULL: u8 = 1; // -
    const ICONST_M1: u8 = 2; // -
    const ICONST_0: u8 = 3; // -
    const ICONST_1: u8 = 4; // -
    const ICONST_2: u8 = 5; // -
    const ICONST_3: u8 = 6; // -
    const ICONST_4: u8 = 7; // -
    const ICONST_5: u8 = 8; // -
    const LCONST_0: u8 = 9; // -
    const LCONST_1: u8 = 10; // -
    const FCONST_0: u8 = 11; // -
    const FCONST_1: u8 = 12; // -
    const FCONST_2: u8 = 13; // -
    const DCONST_0: u8 = 14; // -
    const DCONST_1: u8 = 15; // -
    const BIPUSH: u8 = 16; // visitIntInsn
    const SIPUSH: u8 = 17; // -
    const LDC: u8 = 18; // visitLdcInsn
    const ILOAD: u8 = 21; // visitVarInsn
    const LLOAD: u8 = 22; // -
    const FLOAD: u8 = 23; // -
    const DLOAD: u8 = 24; // -
    const ALOAD: u8 = 25; // -
    const IALOAD: u8 = 46; // visitInsn
    const LALOAD: u8 = 47; // -
    const FALOAD: u8 = 48; // -
    const DALOAD: u8 = 49; // -
    const AALOAD: u8 = 50; // -
    const BALOAD: u8 = 51; // -
    const CALOAD: u8 = 52; // -
    const SALOAD: u8 = 53; // -
    const ISTORE: u8 = 54; // visitVarInsn
    const LSTORE: u8 = 55; // -
    const FSTORE: u8 = 56; // -
    const DSTORE: u8 = 57; // -
    const ASTORE: u8 = 58; // -
    const IASTORE: u8 = 79; // visitInsn
    const LASTORE: u8 = 80; // -
    const FASTORE: u8 = 81; // -
    const DASTORE: u8 = 82; // -
    const AASTORE: u8 = 83; // -
    const BASTORE: u8 = 84; // -
    const CASTORE: u8 = 85; // -
    const SASTORE: u8 = 86; // -
    const POP: u8 = 87; // -
    const POP2: u8 = 88; // -
    const DUP: u8 = 89; // -
    const DUP_X1: u8 = 90; // -
    const DUP_X2: u8 = 91; // -
    const DUP2: u8 = 92; // -
    const DUP2_X1: u8 = 93; // -
    const DUP2_X2: u8 = 94; // -
    const SWAP: u8 = 95; // -
    const IADD: u8 = 96; // -
    const LADD: u8 = 97; // -
    const FADD: u8 = 98; // -
    const DADD: u8 = 99; // -
    const ISUB: u8 = 100; // -
    const LSUB: u8 = 101; // -
    const FSUB: u8 = 102; // -
    const DSUB: u8 = 103; // -
    const IMUL: u8 = 104; // -
    const LMUL: u8 = 105; // -
    const FMUL: u8 = 106; // -
    const DMUL: u8 = 107; // -
    const IDIV: u8 = 108; // -
    const LDIV: u8 = 109; // -
    const FDIV: u8 = 110; // -
    const DDIV: u8 = 111; // -
    const IREM: u8 = 112; // -
    const LREM: u8 = 113; // -
    const FREM: u8 = 114; // -
    const DREM: u8 = 115; // -
    const INEG: u8 = 116; // -
    const LNEG: u8 = 117; // -
    const FNEG: u8 = 118; // -
    const DNEG: u8 = 119; // -
    const ISHL: u8 = 120; // -
    const LSHL: u8 = 121; // -
    const ISHR: u8 = 122; // -
    const LSHR: u8 = 123; // -
    const IUSHR: u8 = 124; // -
    const LUSHR: u8 = 125; // -
    const IAND: u8 = 126; // -
    const LAND: u8 = 127; // -
    const IOR: u8 = 128; // -
    const LOR: u8 = 129; // -
    const IXOR: u8 = 130; // -
    const LXOR: u8 = 131; // -
    const IINC: u8 = 132; // visitIincInsn
    const I2L: u8 = 133; // visitInsn
    const I2F: u8 = 134; // -
    const I2D: u8 = 135; // -
    const L2I: u8 = 136; // -
    const L2F: u8 = 137; // -
    const L2D: u8 = 138; // -
    const F2I: u8 = 139; // -
    const F2L: u8 = 140; // -
    const F2D: u8 = 141; // -
    const D2I: u8 = 142; // -
    const D2L: u8 = 143; // -
    const D2F: u8 = 144; // -
    const I2B: u8 = 145; // -
    const I2C: u8 = 146; // -
    const I2S: u8 = 147; // -
    const LCMP: u8 = 148; // -
    const FCMPL: u8 = 149; // -
    const FCMPG: u8 = 150; // -
    const DCMPL: u8 = 151; // -
    const DCMPG: u8 = 152; // -
    const IFEQ: u8 = 153; // visitJumpInsn
    const IFNE: u8 = 154; // -
    const IFLT: u8 = 155; // -
    const IFGE: u8 = 156; // -
    const IFGT: u8 = 157; // -
    const IFLE: u8 = 158; // -
    const IF_ICMPEQ: u8 = 159; // -
    const IF_ICMPNE: u8 = 160; // -
    const IF_ICMPLT: u8 = 161; // -
    const IF_ICMPGE: u8 = 162; // -
    const IF_ICMPGT: u8 = 163; // -
    const IF_ICMPLE: u8 = 164; // -
    const IF_ACMPEQ: u8 = 165; // -
    const IF_ACMPNE: u8 = 166; // -
    const GOTO: u8 = 167; // -
    const JSR: u8 = 168; // -
    const RET: u8 = 169; // visitVarInsn
    const TABLESWITCH: u8 = 170; // visiTableSwitchInsn
    const LOOKUPSWITCH: u8 = 171; // visitLookupSwitch
    const IRETURN: u8 = 172; // visitInsn
    const LRETURN: u8 = 173; // -
    const FRETURN: u8 = 174; // -
    const DRETURN: u8 = 175; // -
    const ARETURN: u8 = 176; // -
    const RETURN: u8 = 177; // -
    const GETSTATIC: u8 = 178; // visitFieldInsn
    const PUTSTATIC: u8 = 179; // -
    const GETFIELD: u8 = 180; // -
    const PUTFIELD: u8 = 181; // -
    const INVOKEVIRTUAL: u8 = 182; // visitMethodInsn
    const INVOKESPECIAL: u8 = 183; // -
    const INVOKESTATIC: u8 = 184; // -
    const INVOKEINTERFACE: u8 = 185; // -
    const INVOKEDYNAMIC: u8 = 186; // visitInvokeDynamicInsn
    const NEW: u8 = 187; // visitTypeInsn
    const NEWARRAY: u8 = 188; // visitIntInsn
    const ANEWARRAY: u8 = 189; // visitTypeInsn
    const ARRAYLENGTH: u8 = 190; // visitInsn
    const ATHROW: u8 = 191; // -
    const CHECKCAST: u8 = 192; // visitTypeInsn
    const INSTANCEOF: u8 = 193; // -
    const MONITORENTER: u8 = 194; // visitInsn
    const MONITOREXIT: u8 = 195; // -
    const MULTIANEWARRAY: u8 = 197; // visitMultiANewArrayInsn
    const IFNULL: u8 = 198; // visitJumpInsn
    const IFNONNULL: u8 = 199; // -
}

impl OpcodesConsts {
    const LDC_W: u8 = 19;
    const LDC2_W: u8 = 20;
    const ILOAD_0: u8 = 26;
    const ILOAD_1: u8 = 27;
    const ILOAD_2: u8 = 28;
    const ILOAD_3: u8 = 29;
    const LLOAD_0: u8 = 30;
    const LLOAD_1: u8 = 31;
    const LLOAD_2: u8 = 32;
    const LLOAD_3: u8 = 33;
    const FLOAD_0: u8 = 34;
    const FLOAD_1: u8 = 35;
    const FLOAD_2: u8 = 36;
    const FLOAD_3: u8 = 37;
    const DLOAD_0: u8 = 38;
    const DLOAD_1: u8 = 39;
    const DLOAD_2: u8 = 40;
    const DLOAD_3: u8 = 41;
    const ALOAD_0: u8 = 42;
    const ALOAD_1: u8 = 43;
    const ALOAD_2: u8 = 44;
    const ALOAD_3: u8 = 45;
    const ISTORE_0: u8 = 59;
    const ISTORE_1: u8 = 60;
    const ISTORE_2: u8 = 61;
    const ISTORE_3: u8 = 62;
    const LSTORE_0: u8 = 63;
    const LSTORE_1: u8 = 64;
    const LSTORE_2: u8 = 65;
    const LSTORE_3: u8 = 66;
    const FSTORE_0: u8 = 67;
    const FSTORE_1: u8 = 68;
    const FSTORE_2: u8 = 69;
    const FSTORE_3: u8 = 70;
    const DSTORE_0: u8 = 71;
    const DSTORE_1: u8 = 72;
    const DSTORE_2: u8 = 73;
    const DSTORE_3: u8 = 74;
    const ASTORE_0: u8 = 75;
    const ASTORE_1: u8 = 76;
    const ASTORE_2: u8 = 77;
    const ASTORE_3: u8 = 78;
    const WIDE: u8 = 196;
    const GOTO_W: u8 = 200;
    const JSR_W: u8 = 201;
}
