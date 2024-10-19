#![allow(non_snake_case)]

use std::io::Read;
use crate::dex::insn::DexInsn;
use crate::dex::insn_syntax::*;
use crate::dex::{I4, U4};
use crate::err::AsmResultOkExt;
use crate::impls::dex::r::util::destruct_u8;
use crate::impls::jvms::r::ReadContext;
use crate::impls::jvms::r::ReadFrom as Reader;
use crate::{AsmErr, AsmResult};
use java_asm_macro::ReadFrom;

macro_rules! simple_impl {
    ($type:ty, $($field:ident),*) => {
        impl Reader for $type {
            #[allow(unused_variables)]
            fn read_from(context: &mut ReadContext) -> AsmResult<Self> {
                $(let $field = context.read()?;)*
                Ok(Self { $($field),* })
            }
        }
    };
}

impl Reader for (U4, U4) {
    fn read_from(context: &mut ReadContext) -> AsmResult<Self> {
        let v = context.get_and_inc()?;
        Ok(destruct_u8(v))
    }
}

#[derive(Copy, Debug, Clone, PartialEq, Eq, ReadFrom)]
pub struct U16For1(u16);
#[derive(Copy, Debug, Clone, PartialEq, Eq, ReadFrom)]
pub struct U16For2(u16, u16);
#[derive(Copy, Debug, Clone, PartialEq, Eq, ReadFrom)]
pub struct U16For3(u16, u16, u16);
#[derive(Copy, Debug, Clone, PartialEq, Eq, ReadFrom)]
pub struct U16For4(u16, u16, u16, u16);
#[derive(Copy, Debug, Clone, PartialEq, Eq, ReadFrom)]
pub struct U16For5(u16, u16, u16, u16, u16);

simple_impl!(F00x,);
simple_impl!(F10x, stub, opcode);

impl Reader for F12x {
    fn read_from(context: &mut ReadContext) -> AsmResult<Self> {
        let (vB, vA) = context.read()?;
        let opcode = context.get_and_inc()?;
        Ok(F12x { opcode, vA, vB })
    }
}

impl Reader for F11n {
    fn read_from(context: &mut ReadContext) -> AsmResult<Self> {
        let (literalB, vA) = context.read()?;
        let opcode = context.get_and_inc()?;
        let literalB = I4::from_u4(literalB);
        Ok(F11n { opcode, vA, literalB })
    }
}

simple_impl!(F11x, vA, opcode);
simple_impl!(F10t, offsetA, opcode);

simple_impl!(F20t, stub, opcode, offsetA);
simple_impl!(F20bc, vA, opcode, constB);
simple_impl!(F22x, vA, opcode, vB);
simple_impl!(F21t, vA, opcode, offsetB);
simple_impl!(F21s, vA, opcode, literalB);
simple_impl!(F21h, vA, opcode, literalB);
simple_impl!(F21c, vA, opcode, constB);
simple_impl!(F23x, vA, opcode, vC, vB);
simple_impl!(F22b, vA, opcode, literalC, vB);

impl Reader for F22t {
    fn read_from(context: &mut ReadContext) -> AsmResult<Self> {
        let (vB, vA) = context.read()?;
        let opcode = context.read()?;
        let offsetC = context.read()?;
        Ok(F22t { opcode, vA, vB, offsetC })
    }
}

impl Reader for F22s {
    fn read_from(context: &mut ReadContext) -> AsmResult<Self> {
        let (vB, vA) = context.read()?;
        let opcode = context.read()?;
        let literalC = context.read()?;
        Ok(F22s { opcode, vA, vB, literalC })
    }
}

impl Reader for F22c {
    fn read_from(context: &mut ReadContext) -> AsmResult<Self> {
        let (vB, vA) = context.read()?;
        let opcode = context.read()?;
        let constC = context.read()?;
        Ok(F22c { opcode, vA, vB, constC })
    }
}

impl Reader for F22cs {
    fn read_from(context: &mut ReadContext) -> AsmResult<Self> {
        let (vB, vA) = context.read()?;
        let opcode = context.read()?;
        let constC = context.read()?;
        Ok(F22cs { opcode, vA, vB, constC })
    }
}

simple_impl!(F30t, stub, opcode, offsetA);
simple_impl!(F32x, stub, opcode, vA, vB);
simple_impl!(F31i, vA, opcode, literalB);
simple_impl!(F31t, vA, opcode, offsetB);
simple_impl!(F31c, vA, opcode, constB);

impl Reader for F35c {
    fn read_from(context: &mut ReadContext) -> AsmResult<Self> {
        let (vA, vG) = context.read()?;
        let opcode = context.read()?;
        let constB = context.read()?;
        let (vF, vE) = context.read()?;
        let (vD, vC) = context.read()?;
        Ok(F35c { opcode, vA, vC, vD, vE, vF, vG, constB })
    }
}

impl Reader for F35ms {
    fn read_from(context: &mut ReadContext) -> AsmResult<Self> {
        // similar format
        let F35c { opcode, vA, vC, vD, vE, vF, vG, constB } = context.read()?;
        Ok(F35ms { opcode, vA, vC, vD, vE, vF, vG, constB })
    }
}

impl Reader for F35mi {
    fn read_from(context: &mut ReadContext) -> AsmResult<Self> {
        // similar format
        let F35c { opcode, vA, vC, vD, vE, vF, vG, constB } = context.read()?;
        Ok(F35mi { opcode, vA, vC, vD, vE, vF, vG, constB })
    }
}

simple_impl!(F3rc, vA, opcode, vB, vC);
simple_impl!(F3rms, vA, opcode, vB, vC);
simple_impl!(F3rmi, vA, opcode, vB, vC);

impl Reader for F45cc {
    fn read_from(context: &mut ReadContext) -> AsmResult<Self> {
        let (vA, vG) = context.read()?;
        let opcode = context.read()?;
        let constB = context.read()?;
        let constH = context.read()?;
        let (vF, vE) = context.read()?;
        let (vD, vC) = context.read()?;
        Ok(F45cc { opcode, vA, vC, vD, vE, vF, vG, constB, constH })
    }
}

simple_impl!(F4rcc, literalA, opcode, constB, vC, constH);
simple_impl!(F51l, vA, opcode, literalB);

impl Reader for DexInsn {
    fn read_from(context: &mut ReadContext) -> AsmResult<Self> {
        let cur_index = context.index;
        let opcode = context.byte_at(cur_index + 1)?;
        if opcode == 0x00 {
            return read_payload(context)
        }
        macro_rules! match_opcodes {
            ($($opcode:pat = $variant:ident,)*) => {
                match opcode {
                    $(
                        $opcode => Ok(DexInsn::$variant(context.read()?)),
                    )*
                    _ => return Err(AsmErr::UnknownInsn(opcode)),
                }
            };
        }

        match_opcodes! {
            0x01 = Move,
            0x02 = MoveFrom16,
            0x03 = Move16,
            0x04 = MoveWide,
            0x05 = MoveWideFrom16,
            0x06 = MoveWide16,
            0x07 = MoveObject,
            0x08 = MoveObjectFrom16,
            0x09 = MoveObject16,
            0x0a = MoveResult,
            0x0b = MoveResultWide,
            0x0c = MoveResultObject,
            0x0d = MoveException,
            0x0e = ReturnVoid,
            0x0f = Return,
            0x10 = ReturnWide,
            0x11 = ReturnObject,
            0x12 = Const4,
            0x13 = Const16,
            0x14 = Const,
            0x15 = ConstHigh16,
            0x16 = ConstWide16,
            0x17 = ConstWide32,
            0x18 = ConstWide,
            0x19 = ConstWideHigh16,
            0x1a = ConstString,
            0x1b = ConstStringJumbo,
            0x1c = ConstClass,
            0x1d = MonitorEnter,
            0x1e = MonitorExit,
            0x1f = CheckCast,
            0x20 = InstanceOf,
            0x21 = ArrayLength,
            0x22 = NewInstance,
            0x23 = NewArray,
            0x24 = FilledNewArray,
            0x25 = FilledNewArrayRange,
            0x26 = FillArrayData,
            0x27 = Throw,
            0x28 = Goto,
            0x29 = Goto16,
            0x2a = Goto32,
            0x2b = PackedSwitch,
            0x2c = SparseSwitch,
            0x2d..=0x31 = Cmpkind,
            0x32..=0x37 = IfTest,
            0x38..=0x3d = IfTestz,
            0x3e..=0x43 = NotUsed, // not used
            0x44..=0x51 = ArrayOp,
            0x52..=0x5f = IInstanceOp,
            0x60..=0x6d = SInstanceOp,
            0x6e..=0x72 = InvokeKind,
            0x73 = NotUsed, // not used
            0x74..=0x78 = InvokeKindRange,
            0x79..=0x7a = NotUsed, // not used
            0x7b..=0x8f = Unop,
            0x90..=0xaf = Binop,
            0xb0..=0xcf = Binop2Addr,
            0xd0..=0xd7 = BinopLit16,
            0xd8..=0xe2 = BinopLit8,
            0xe3..=0xf9 = NotUsed, // not used
            0xfa = InvokePoly,
            0xfb = InvokePolyRange,
            0xfc = InvokeCustom,
            0xfd = InvokeCustomRange,
            0xfe = ConstMethodHandle,
            0xff = ConstMethodType,
        }
    }
}
fn read_payload(context: &mut ReadContext) -> AsmResult<DexInsn> {
    let ident = context.get_cur()?;
    match ident {
        0x01 => DexInsn::PackedSwitchPayload(context.read()?),
        0x02 => DexInsn::SparseSwitchPayload(context.read()?),
        0x03 => DexInsn::FillArrayDataPayload(context.read()?),
        _ => return Err(AsmErr::UnknownDexPayload(ident))
    }.ok()
}
