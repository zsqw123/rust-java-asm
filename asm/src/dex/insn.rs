use java_asm_macro::ReadFrom;
use crate::dex::elements::{DUInt, DUShort};
use crate::dex::insn_syntax::*;
use crate::impls::jvms::r::U32BasedSize;

pub type NopInsn = F10x; // 0x00
pub type MoveInsn = F12x; // 0x01
pub type MoveFrom16Insn = F22x; // 0x02
pub type Move16Insn = F32x; // 0x03
pub type MoveWideInsn = F12x; // 0x04
pub type MoveWideFrom16Insn = F22x; // 0x05
pub type MoveWide16Insn = F32x; // 0x06
pub type MoveObjectInsn = F12x; // 0x07
pub type MoveObjectFrom16Insn = F22x; // 0x08
pub type MoveObject16Insn = F32x; // 0x09
pub type MoveResultInsn = F11x; // 0x0a
pub type MoveResultWideInsn = F11x; // 0x0b
pub type MoveResultObjectInsn = F11x; // 0x0c
pub type MoveExceptionInsn = F11x; // 0x0d
pub type ReturnVoidInsn = F10x; // 0x0e
pub type ReturnInsn = F11x; // 0x0f
pub type ReturnWideInsn = F11x; // 0x10
pub type ReturnObjectInsn = F11x; // 0x11
pub type Const4Insn = F11n; // 0x12
pub type Const16Insn = F21s; // 0x13
pub type ConstInsn = F31i; // 0x14
pub type ConstHigh16Insn = F21h; // 0x15
pub type ConstWide16Insn = F21s; // 0x16
pub type ConstWide32Insn = F31i; // 0x17
pub type ConstWideInsn = F51l; // 0x18
pub type ConstWideHigh16Insn = F21h; // 0x19
pub type ConstStringInsn = F21c; // 0x1a
pub type ConstStringJumboInsn = F31c; // 0x1b
pub type ConstClassInsn = F21c; // 0x1c
pub type MonitorEnterInsn = F11x; // 0x1d
pub type MonitorExitInsn = F11x; // 0x1e
pub type CheckCastInsn = F21c; // 0x1f
pub type InstanceOfInsn = F22c; // 0x20
pub type ArrayLengthInsn = F12x; // 0x21
pub type NewInstanceInsn = F21c; // 0x22
pub type NewArrayInsn = F22c; // 0x23
pub type FilledNewArrayInsn = F35c; // 0x24
pub type FilledNewArrayRangeInsn = F3rc; // 0x25
pub type FillArrayDataInsn = F31t; // 0x26
pub type ThrowInsn = F11x; // 0x27
pub type GotoInsn = F10t; // 0x28
pub type Goto16Insn = F20t; // 0x29
pub type Goto32Insn = F30t; // 0x2a
pub type PackedSwitchInsn = F31t; // 0x2b
pub type SparseSwitchInsn = F31t; // 0x2c
pub type CmpkindInsn = F23x; // 0x2d..0x31
pub type IfTestInsn = F22t; // 0x32..0x37
pub type IfTestzInsn = F21t; // 0x38..0x3d
// for 3e..43, map to 10x
pub type ArrayOpInsn = F23x; // 0x44..0x51
pub type IInstanceOpInsn = F22c; // 0x52..0x5f
pub type SInstanceOpInsn = F21c; // 0x60..0x6d
pub type InvokeKindInsn = F35c; // 0x6e..0x72
// for 73, unused, map to 10x
pub type InvokeKindRangeInsn = F3rc; // 0x74..0x78
// for 79..7a, unused, map to 10x
pub type UnopInsn = F12x; // 0x7b..0x8f
pub type BinopInsn = F23x; // 0x90..0xaf
pub type Binop2AddrInsn = F12x; // 0xb0..0xcf
pub type BinopLit16Insn = F22s; // 0xd0..0xd7
pub type BinopLit8Insn = F22b; // 0xd8..0xe2
// for e3..f9, unused, map to 10x
pub type InvokePolyInsn = F45cc; // 0xfa
pub type InvokePolyRangeInsn = F4rcc; // 0xfb
pub type InvokeCustomInsn = F35c; // 0xfc
pub type InvokeCustomRangeInsn = F3rc; // 0xfd
pub type ConstMethodHandleInsn = F21c; // 0xfe
pub type ConstMethodTypeInsn = F21c; // 0xff

#[derive(Debug, Clone, PartialEq, Eq, ReadFrom)]
pub struct PackedSwitchPayload {
    pub ident: DUShort, // should always be 0x0100
    pub size: DUShort,
    pub first_key: DUInt,
    #[index(size)]
    pub targets: Vec<DUInt>,
}

#[derive(Debug, Clone, PartialEq, Eq, ReadFrom)]
pub struct SparseSwitchPayload {
    pub ident: DUShort, // should always be 0x0200
    pub size: DUShort,
    #[index(size)]
    pub keys: Vec<DUInt>,
    #[index(size)]
    pub targets: Vec<DUInt>,
}

#[derive(Debug, Clone, PartialEq, Eq, ReadFrom)]
pub struct FillArrayDataPayload {
    pub ident: DUShort, // should always be 0x0300
    pub element_width: DUShort,
    pub size: U32BasedSize,
    #[index(size)]
    pub data: Vec<u8>,
}

