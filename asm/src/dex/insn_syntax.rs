#![allow(non_snake_case)]

use crate::dex::{Opcode, I4, U4};
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
