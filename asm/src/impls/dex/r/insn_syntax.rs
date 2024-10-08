#![allow(non_snake_case)]

use crate::dex::Opcode;

pub struct U4(u8);
pub struct I4(u8);

/// vX -> register
/// lX -> literal
/// oX -> address offset
/// cX -> index of constant pool
enum InsnSyntax {
    F00x,
    F10x(Opcode),
    F12x { opcode: Opcode, vA: U4, vB: U4 },
    F11n { opcode: Opcode, vA: U4, lB: I4 },
    F11x { opcode: Opcode, vA: u8 },
    F10t { opcode: Opcode, oA: i8 },
    
    F20t { opcode: Opcode, oA: i16 },
    F20bc { opcode: Opcode, vA: u8, cB: u16 },
    F22x { opcode: Opcode, vA: u8, vB: u16 },
    F21t { opcode: Opcode, vA: u8, oB: i16 },
    F21s { opcode: Opcode, vA: u8, lB: i16 },
    F21h { opcode: Opcode, vA: u8, lB: i16 },
    F21c { opcode: Opcode, vA: u8, cB: u16 },
}
