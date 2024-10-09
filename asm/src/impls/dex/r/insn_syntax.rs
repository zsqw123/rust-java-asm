#![allow(non_snake_case)]

use crate::dex::Opcode;
use crate::impls::dex::r::util::{destruct_u8, I4, U4};
use crate::impls::jvms::r::ReadContext;
use crate::impls::jvms::r::ReadFrom as Reader;
use crate::AsmResult;
use java_asm_macro::ReadFrom;

macro_rules! syntax {
    (
        $(
            $name:ident {
                $($field:ident: $t:ty),*
            };
        )*
    ) => {
        $(
            #[derive(Copy, Debug, Clone, PartialEq, Eq)]
            pub struct $name {
                $(pub $field: $t),*
            }
        )*
    }
}

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

fn read_u4_pair(context: &mut ReadContext) -> AsmResult<(U4, U4)> {
    let v = context.get_and_inc()?;
    Ok(destruct_u8(v))
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

syntax! {
    F00x {};
    // XX|op
    F10x { stub: u8, opcode: Opcode };
    // B|A|op
    F12x { opcode: Opcode, vA: U4, vB: U4 };
    F11n { opcode: Opcode, vA: U4, literalB: I4 };
    // AA|op
    F11x { opcode: Opcode, vA: u8 };
    F10t { opcode: Opcode, offsetA: i8 };
}

simple_impl!(F00x,);
simple_impl!(F10x, stub, opcode);

impl Reader for F12x {
    fn read_from(context: &mut ReadContext) -> AsmResult<Self> {
        let (vB, vA) = read_u4_pair(context)?;
        let opcode = context.get_and_inc()?;
        Ok(F12x { opcode, vA, vB })
    }
}

impl Reader for F11n {
    fn read_from(context: &mut ReadContext) -> AsmResult<Self> {
        let (literalB, vA) = read_u4_pair(context)?;
        let opcode = context.get_and_inc()?;
        let literalB = I4(literalB.0);
        Ok(F11n { opcode, vA, literalB })
    }
}

simple_impl!(F11x, vA, opcode);
simple_impl!(F10t, offsetA, opcode);

syntax! {
    // XX|op
    // AA AA
    F20t { stub: u8, opcode: Opcode, offsetA: i16 };
    // AA|op
    // BB BB
    F20bc { opcode: Opcode, vA: u8, constB: u16 };
    // AA|op
    // BB BB
    F22x { opcode: Opcode, vA: u8, vB: u16 };
    F21t { opcode: Opcode, vA: u8, offsetB: i16 };
    F21s { opcode: Opcode, vA: u8, literalB: i16 };
    F21h { opcode: Opcode, vA: u8, literalB: i16 };
    F21c { opcode: Opcode, vA: u8, constB: u16 };
    // AA|op
    // CC|BB
    F23x { opcode: Opcode, vA: u8, vB: u8, vC: u8 };
    F22b { opcode: Opcode, vA: u8, vB: u8, literalC: i8 };
    // B|A|op
    // C C CC
    F22t { opcode: Opcode, vA: U4, vB: U4, offsetC: i16 };
    F22s { opcode: Opcode, vA: U4, vB: U4, literalC: i16 };
    F22c { opcode: Opcode, vA: U4, vB: U4, constC: u16 };
    F22cs { opcode: Opcode, vA: U4, vB: U4, constC: u16 };
}

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
        let (vB, vA) = read_u4_pair(context)?;
        let opcode = context.read()?;
        let offsetC = context.read()?;
        Ok(F22t { opcode, vA, vB, offsetC })
    }
}

impl Reader for F22s {
    fn read_from(context: &mut ReadContext) -> AsmResult<Self> {
        let (vB, vA) = read_u4_pair(context)?;
        let opcode = context.read()?;
        let literalC = context.read()?;
        Ok(F22s { opcode, vA, vB, literalC })
    }
}

impl Reader for F22c {
    fn read_from(context: &mut ReadContext) -> AsmResult<Self> {
        let (vB, vA) = read_u4_pair(context)?;
        let opcode = context.read()?;
        let constC = context.read()?;
        Ok(F22c { opcode, vA, vB, constC })
    }
}

impl Reader for F22cs {
    fn read_from(context: &mut ReadContext) -> AsmResult<Self> {
        let (vB, vA) = read_u4_pair(context)?;
        let opcode = context.read()?;
        let constC = context.read()?;
        Ok(F22cs { opcode, vA, vB, constC })
    }
}

syntax! {
    // XX|op
    // AA AA (lo)
    // AA AA (hi)
    F30t { stub: u8, opcode: Opcode, offsetA: i32 };
    // XX|op
    // AA AA
    // BB BB
    F32x { stub: u8, opcode: Opcode, vA: u16, vB: u16 };
    // AA|op
    // BB BB (lo)
    // BB BB (hi)
    F31i { opcode: Opcode, vA: u8, literalB: i32 };
    F31t { opcode: Opcode, vA: u8, offsetB: i32 };
    F31c { opcode: Opcode, vA: u8, constB: u32 };
    // A|G|op
    // B B B B
    // F|E|D|C
    F35c { opcode: Opcode, vA: U4, vC: U4, vD: U4, vE: U4, vF: U4, vG: U4, constB: u16 };
    F35ms { opcode: Opcode, vA: U4, vC: U4, vD: U4, vE: U4, vF: U4, vG: U4, constB: u16 };
    F35mi { opcode: Opcode, vA: U4, vC: U4, vD: U4, vE: U4, vF: U4, vG: U4, constB: u16 };
    // AA|op
    // BB|BB
    // CC|CC
    //      N = CC CC + AA - 1
    F3rc { opcode: Opcode, vA: u8, vB: u16, vC: u16 };
    F3rms { opcode: Opcode, vA: u8, vB: u16, vC: u16 };
    F3rmi { opcode: Opcode, vA: u8, vB: u16, vC: u16 };
}

simple_impl!(F30t, stub, opcode, offsetA);
simple_impl!(F32x, stub, opcode, vA, vB);
simple_impl!(F31i, vA, opcode, literalB);
simple_impl!(F31t, vA, opcode, offsetB);
simple_impl!(F31c, vA, opcode, constB);

impl Reader for F35c {
    fn read_from(context: &mut ReadContext) -> AsmResult<Self> {
        let (vA, vG) = read_u4_pair(context)?;
        let opcode = context.read()?;
        let constB = context.read()?;
        let (vF, vE) = read_u4_pair(context)?;
        let (vD, vC) = read_u4_pair(context)?;
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

syntax! {
    // A|G|op
    // B B B B
    // F|E|D|C
    // H H H H
    F45cc { 
        opcode: Opcode, vA: U4,
        vC: U4, vD: U4, vE: U4, vF: U4, vG: U4,
        constB: u16, constH: u16
    };
    // AA|op
    // BB BB
    // CC CC
    // HH HH
    //      N = CC CC + AA - 1
    F4rcc { 
        opcode: Opcode, literalA: u8,
        constB: u16, constH: u16,
        vC: u16
    };
    // AA|op
    // BB BB (lo)
    // BB BB
    // BB BB
    // BB BB (hi)
    F51l { opcode: Opcode, vA: u8, literalB: i64 };
}

impl Reader for F45cc {
    fn read_from(context: &mut ReadContext) -> AsmResult<Self> {
        let (vA, vG) = read_u4_pair(context)?;
        let opcode = context.read()?;
        let constB = context.read()?;
        let constH = context.read()?;
        let (vF, vE) = read_u4_pair(context)?;
        let (vD, vC) = read_u4_pair(context)?;
        Ok(F45cc { opcode, vA, vC, vD, vE, vF, vG, constB, constH })
    }
}

simple_impl!(F4rcc, literalA, opcode, constB, vC, constH);
simple_impl!(F51l, vA, opcode, literalB);
