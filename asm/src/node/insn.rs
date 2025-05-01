use crate::node::element::LabelNode;
use crate::node::values::{ConstDynamic, ConstValue};
use crate::{InternalNameRef, StrRef};
use std::sync::Arc;

//noinspection SpellCheckingInspection
#[derive(Clone)]
pub enum InsnNode {
    FieldInsnNode {
        opcode: u8, // GETSTATIC, PUTSTATIC, GETFIELD, PUTFIELD
        owner: StrRef, // internal name of the field's owner class
        name: StrRef,
        desc: StrRef,
    },
    IIncInsnNode {
        var: u16, // index of the local variable to be incremented
        incr: i16,
    },
    NoOperand {
        // the opcode of the instruction to be constructed. This opcode must be NOP,
        // ACONST_NULL, ICONST_M1, ICONST_0, ICONST_1, ICONST_2, ICONST_3, ICONST_4, ICONST_5,
        // LCONST_0, LCONST_1, FCONST_0, FCONST_1, FCONST_2, DCONST_0, DCONST_1, IALOAD, LALOAD,
        // FALOAD, DALOAD, AALOAD, BALOAD, CALOAD, SALOAD, IASTORE, LASTORE, FASTORE, DASTORE,
        // AASTORE, BASTORE, CASTORE, SASTORE, POP, POP2, DUP, DUP_X1, DUP_X2, DUP2, DUP2_X1, DUP2_X2,
        // SWAP, IADD, LADD, FADD, DADD, ISUB, LSUB, FSUB, DSUB, IMUL, LMUL, FMUL, DMUL, IDIV, LDIV,
        // FDIV, DDIV, IREM, LREM, FREM, DREM, INEG, LNEG, FNEG, DNEG, ISHL, LSHL, ISHR, LSHR, IUSHR,
        // LUSHR, IAND, LAND, IOR, LOR, IXOR, LXOR, I2L, I2F, I2D, L2I, L2F, L2D, F2I, F2L, F2D, D2I,
        // D2L, D2F, I2B, I2C, I2S, LCMP, FCMPL, FCMPG, DCMPL, DCMPG, IRETURN, LRETURN, FRETURN,
        // DRETURN, ARETURN, RETURN, ARRAYLENGTH, ATHROW, MONITORENTER, or MONITOREXIT.
        opcode: u8,
    },
    BIPushInsnNode { operand: i8 },
    SIPushInsnNode { operand: i16 },
    InvokeDynamicInsnNode(ConstDynamic),
    JumpInsnNode {
        // the opcode of the type instruction to be constructed. This opcode must be IFEQ, IFNE,
        // IFLT, IFGE, IFGT, IFLE, IF_ICMPEQ, IF_ICMPNE, IF_ICMPLT, IF_ICMPGE, IF_ICMPGT,
        // IF_ICMPLE, IF_ACMPEQ, IF_ACMPNE, GOTO, JSR, IFNULL or IFNONNULL.
        opcode: u8,
        label: LabelNode,
    },
    LdcInsnNode(Arc<ConstValue>),
    TableSwitchInsnNode {
        default: LabelNode, // Beginning of the default handler block.
        min: i32, // The minimum key value.
        max: i32, // The maximum key value.
        labels: Vec<LabelNode>,
    },
    LookupSwitchInsnNode {
        default: LabelNode, // Beginning of the default handler block.
        keys: Vec<i32>,
        labels: Vec<LabelNode>,
    },
    MethodInsnNode {
        opcode: u8, // INVOKEVIRTUAL, INVOKESPECIAL, INVOKESTATIC, INVOKEINTERFACE
        owner: StrRef, // internal name of the method's owner class
        name: StrRef,
        desc: StrRef,
    },
    NewArrayInsnNode { array_type: u8 }, // T_BOOLEAN, T_CHAR, T_FLOAT, T_DOUBLE, T_BYTE, T_SHORT, T_INT, T_LONG
    MultiANewArrayInsnNode {
        array_type: InternalNameRef,
        dims: u8, // Number of dimensions of the array to allocate.
    },
    TypeInsnNode {
        opcode: u8, // NEW, ANEWARRAY, CHECKCAST or INSTANCEOF
        type_name: InternalNameRef,
    },
    VarInsnNode {
        opcode: u8, // ILOAD, LLOAD, FLOAD, DLOAD, ALOAD, ISTORE, LSTORE, FSTORE, DSTORE, ASTORE
        var_index: u16, // index of the local variable to load or store
    },
}
