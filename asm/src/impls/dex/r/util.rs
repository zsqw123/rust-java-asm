use crate::{AsmErr, AsmResult};
use crate::err::AsmResultOkExt;
use crate::impls::jvms::r::{ReadContext, ReadFrom};

// dex types
pub type DByte = i8;
pub type DUByte = u8;
pub type DShort = i16;
pub type DUShort = u16;
pub type DInt = i32;
pub type DUInt = u32;
pub type DLong = i64;
pub type DULong = u64;

pub struct DSleb128(u32);
pub struct DULeb128(u32);
pub struct DULeb128P1(u32);

impl DSleb128 {
    #[inline]
    pub const fn value(&self) -> i32 {
        self.0 as i32
    }
}

impl DULeb128 {
    #[inline]
    pub const fn value(&self) -> u32 {
        self.0
    }
}

impl DULeb128P1 {
    // -1 is used for representing null
    #[inline]
    pub const fn value(&self) -> Option<u32> {
        let internal = self.0;
        if internal == 0 { None } else { Some(internal - 1) }
    }
}

impl ReadFrom for DByte {
    #[inline]
    fn read_from(context: &mut ReadContext) -> AsmResult<Self> {
        u8::read_from(context).map(|v| v as i8)
    }
}

impl ReadFrom for DShort {
    #[inline]
    fn read_from(context: &mut ReadContext) -> AsmResult<Self> {
        u16::read_from(context).map(|v| v as i16)
    }
}

impl ReadFrom for DInt {
    #[inline]
    fn read_from(context: &mut ReadContext) -> AsmResult<Self> {
        u32::read_from(context).map(|v| v as i32)
    }
}

impl ReadFrom for DLong {
    #[inline]
    fn read_from(context: &mut ReadContext) -> AsmResult<Self> {
        u64::read_from(context).map(|v| v as i64)
    }
}

impl ReadFrom for DULeb128 {
    #[inline]
    fn read_from(context: &mut ReadContext) -> AsmResult<Self> {
        let mut result = 0u32;
        let mut shift = 0u8;
        let start_index = context.index;
        loop {
            let byte = u8::read_from(context)?;
            if context.endian {
                result <<= shift;
                result |= (byte & 0x7F) as u32;
            } else {
                result |= (byte & 0x7F) as u32;
                result <<= shift;
            }
            if byte & 0x80 == 0 {
                break;
            }
            shift += 7;
        }
        // android dex's LEB128 represent a single 32-bit value
        if shift > 32 {
            // in android dex format, the maximum length of LEB128 is 5 bytes
            return Err(AsmErr::InvalidLEB128(start_index));
        }
        DULeb128(result).ok()
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
