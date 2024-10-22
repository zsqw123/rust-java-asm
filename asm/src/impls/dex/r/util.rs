use crate::dex::U4;
use crate::dex::*;
use crate::err::AsmResultOkExt;
use crate::impls::jvms::r::{ReadContext, ReadFrom};
use crate::{AsmErr, AsmResult};

#[inline]
pub fn destruct_u8(v: u8) -> (U4, U4) {
    (U4(v >> 4), U4(v & 0x0F))
}

impl ReadFrom for DULeb128 {
    #[inline]
    fn read_from(context: &mut ReadContext) -> AsmResult<Self> {
        let mut result = 0u64;
        let mut shift = 0u8;
        let start_index = context.index;
        loop {
            let byte = u8::read_from(context)?;
            let value = (byte & 0x7F) as u64;
            if context.endian {
                result |= value;
            } else {
                result |= value << shift;
            }
            if byte & 0x80 == 0 {
                break;
            }
            shift += 7;
            if context.endian {
                result <<= 7;
            }
        }
        // android dex's LEB128 represent a single 32-bit value
        if shift > 32 {
            // in android dex format, the maximum length of LEB128 is 5 bytes
            return Err(AsmErr::InvalidLEB128(start_index));
        }
        DULeb128(result as u32).ok()
    }
}

impl ReadFrom for DULeb128P1 {
    #[inline]
    fn read_from(context: &mut ReadContext) -> AsmResult<Self> {
        DULeb128P1(DULeb128::read_from(context)?.0).ok()
    }
}

impl ReadFrom for DSleb128 {
    #[inline]
    fn read_from(context: &mut ReadContext) -> AsmResult<Self> {
        Self(DULeb128::read_from(context)?.0).ok()
    }
}
