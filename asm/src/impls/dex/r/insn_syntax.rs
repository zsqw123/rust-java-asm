#![allow(non_snake_case)]

use crate::dex::insn_syntax::*;
use crate::impls::dex::r::util::{destruct_u8};
use crate::impls::jvms::r::ReadContext;
use crate::impls::jvms::r::ReadFrom as Reader;
use crate::AsmResult;
use java_asm_macro::ReadFrom;
use crate::dex::{I4, U4};

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
        let literalB = I4(literalB.0);
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
