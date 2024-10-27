use crate::dex::insn_syntax::*;
use crate::dex::raw::{DUInt, DUShort};
use crate::impls::jvms::r::U32BasedSize;
use java_asm_macro::ReadFrom;
use crate::dex::DInt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DexInsn {
    Nop(F00x), // 0x00
    Move(F12x), // 0x01
    MoveFrom16(F22x), // 0x02
    Move16(F32x), // 0x03
    MoveWide(F12x), // 0x04
    MoveWideFrom16(F22x), // 0x05
    MoveWide16(F32x), // 0x06
    MoveObject(F12x), // 0x07
    MoveObjectFrom16(F22x), // 0x08
    MoveObject16(F32x), // 0x09
    MoveResult(F11x), // 0x0a
    MoveResultWide(F11x), // 0x0b
    MoveResultObject(F11x), // 0x0c
    MoveException(F11x), // 0x0d
    ReturnVoid(F10x), // 0x0e
    Return(F11x), // 0x0f
    ReturnWide(F11x), // 0x10
    ReturnObject(F11x), // 0x11
    Const4(F11n), // 0x12
    Const16(F21s), // 0x13
    Const(F31i), // 0x14
    ConstHigh16(F21h), // 0x15
    ConstWide16(F21s), // 0x16
    ConstWide32(F31i), // 0x17
    ConstWide(F51l), // 0x18
    ConstWideHigh16(F21h), // 0x19
    ConstString(F21c), // 0x1a
    ConstStringJumbo(F31c), // 0x1b
    ConstClass(F21c), // 0x1c
    MonitorEnter(F11x), // 0x1d
    MonitorExit(F11x), // 0x1e
    CheckCast(F21c), // 0x1f
    InstanceOf(F22c), // 0x20
    ArrayLength(F12x), // 0x21
    NewInstance(F21c), // 0x22
    NewArray(F22c), // 0x23
    FilledNewArray(F35c), // 0x24
    FilledNewArrayRange(F3rc), // 0x25
    FillArrayData(F31t), // 0x26
    Throw(F11x), // 0x27
    Goto(F10t), // 0x28
    Goto16(F20t), // 0x29
    Goto32(F30t), // 0x2a
    PackedSwitch(F31t), // 0x2b
    SparseSwitch(F31t), // 0x2c
    Cmpkind(F23x), // 0x2d..0x31
    IfTest(F22t), // 0x32..0x37
    IfTestz(F21t), // 0x38..0x3d  
    // for 3e..43, map to 10x
    ArrayOp(F23x), // 0x44..0x51
    IInstanceOp(F22c), // 0x52..0x5f
    SStaticOp(F21c), // 0x60..0x6d
    InvokeKind(F35c), // 0x6e..0x72
    // for 73, unused, map to 10x
    InvokeKindRange(F3rc), // 0x74..0x78 
    // for 79..7a, unused, map to 10x
    Unop(F12x), // 0x7b..0x8f
    Binop(F23x), // 0x90..0xaf
    Binop2Addr(F12x), // 0xb0..0xcf
    BinopLit16(F22s), // 0xd0..0xd7
    BinopLit8(F22b), // 0xd8..0xe2 
    // for e3..f9, unused, map to 10x
    InvokePoly(F45cc), // 0xfa
    InvokePolyRange(F4rcc), // 0xfb
    InvokeCustom(F35c), // 0xfc
    InvokeCustomRange(F3rc), // 0xfd
    ConstMethodHandle(F21c), // 0xfe
    ConstMethodType(F21c), // 0xff
    
    NotUsed(F10x),
    
    // payloads
    PackedSwitchPayload(PackedSwitchPayload),
    SparseSwitchPayload(SparseSwitchPayload),
    FillArrayDataPayload(FillArrayDataPayload),
}

macro_rules! insn_width_impl {
    ($($width:expr, [$($variant:ident),*],)*) => {
        impl DexInsn {
            pub fn insn_width(&self) -> usize {
                match self {
                    $(
                        $(DexInsn::$variant(_) => $width,)*
                    )*
                }
            }
        }
    };
}

insn_width_impl! {
    1, [Nop, Move, MoveWide, MoveObject, MoveResult, MoveResultWide, MoveResultObject, MoveException, 
        ReturnVoid, Return, ReturnWide, ReturnObject, Const4, MonitorEnter, MonitorExit, ArrayLength, 
        Throw, Goto, Unop, Binop2Addr, NotUsed],
    2, [MoveFrom16, MoveWideFrom16, MoveObjectFrom16, 
        Const16, ConstHigh16, ConstWide16, ConstWideHigh16, ConstString, ConstClass, 
        CheckCast, InstanceOf, NewInstance, NewArray,
        Goto16, Cmpkind, IfTest, IfTestz, ArrayOp, IInstanceOp, SStaticOp,
        Binop, BinopLit16, BinopLit8, ConstMethodHandle, ConstMethodType],
    3, [Move16, MoveWide16, MoveObject16, Goto32, Const, ConstWide32, ConstStringJumbo,
        FilledNewArray, FilledNewArrayRange, FillArrayData, PackedSwitch, SparseSwitch, 
        InvokeKind, InvokeKindRange, InvokeCustom, InvokeCustomRange],
    4, [InvokePoly, InvokePolyRange],
    5, [ConstWide],
    // payload didn't have real width and they didn't real occur in instruction list.
    0, [PackedSwitchPayload, SparseSwitchPayload, FillArrayDataPayload],
}

#[derive(Debug, Clone, PartialEq, Eq, ReadFrom)]
pub struct PackedSwitchPayload {
    pub ident: DUShort, // should always be 0x0100
    pub size: DUShort,
    pub first_key: DInt,
    #[index(size)]
    pub targets: Vec<DInt>,
}

#[derive(Debug, Clone, PartialEq, Eq, ReadFrom)]
pub struct SparseSwitchPayload {
    pub ident: DUShort, // should always be 0x0200
    pub size: DUShort,
    #[index(size)]
    pub keys: Vec<DInt>,
    #[index(size)]
    pub targets: Vec<DInt>,
}

#[derive(Debug, Clone, PartialEq, Eq, ReadFrom)]
pub struct FillArrayDataPayload {
    pub ident: DUShort, // should always be 0x0300
    pub element_width: DUShort,
    pub size: U32BasedSize,
    #[index(size)]
    pub data: Vec<u8>,
}

