pub use java_asm_macro::ReadFrom;

use crate::AsmErr;
use crate::err::AsmResult;

pub(crate) mod jvms_reader;
pub(crate) mod util;
pub(crate) mod transform;
pub(crate) mod frame;

pub struct ReadContext<'a> {
    pub bytes: &'a [u8],
    pub index: usize,
    pub endian: bool, // true: big endian, false: little endian
}

impl <'a> ReadContext<'a> {
    pub fn big_endian(bytes: &'a [u8]) -> Self {
        Self { bytes, index: 0, endian: true }
    }
    
    pub fn little_endian(bytes: &'a [u8]) -> Self {
        Self { bytes, index: 0, endian: false }
    }
}

impl ReadContext<'_> {
    #[inline]
    pub fn get_and_inc(&mut self) -> AsmResult<u8> {
        let current_index = self.index;
        let content = self.bytes.get(current_index).copied()
            .ok_or_else(|| AsmErr::OutOfRange(current_index))?;
        self.index = current_index + 1;
        Ok(content)
    }
}

pub trait ReadFrom where Self: Sized {
    fn read_from(context: &mut ReadContext) -> AsmResult<Self>;
}

impl ReadContext<'_> {
    #[inline]
    pub fn read<T: ReadFrom>(&mut self) -> AsmResult<T> {
        T::read_from(self)
    }

    #[inline]
    pub fn read_vec<T: ReadFrom>(&mut self, vec_size: usize) -> AsmResult<Vec<T>> {
        let mut vec = Vec::with_capacity(vec_size);
        for _ in 0..vec_size {
            vec.push(self.read()?);
        }
        Ok(vec)
    }
}

impl ReadFrom for u8 {
    #[inline]
    fn read_from(context: &mut ReadContext) -> AsmResult<u8> {
        context.get_and_inc()
    }
}

impl ReadFrom for u16 {
    #[inline]
    fn read_from(context: &mut ReadContext) -> AsmResult<u16> {
        let h = (context.get_and_inc()? as u16) << 8;
        let l = context.get_and_inc()? as u16;
        if context.endian {
            Ok(h | l)
        } else {
            Ok(l | h)
        }
    }
}

impl ReadFrom for u32 {
    #[inline]
    fn read_from(context: &mut ReadContext) -> AsmResult<u32> {
        let a = (context.get_and_inc()? as u32) << 24;
        let b = (context.get_and_inc()? as u32) << 16;
        let c = (context.get_and_inc()? as u32) << 8;
        let d = context.get_and_inc()? as u32;
        if context.endian {
            Ok(a | b | c | d)
        } else {
            Ok(d | c | b | a)
        }
    }
}

impl ReadFrom for u64 {
    #[inline]
    fn read_from(context: &mut ReadContext) -> AsmResult<u64> {
        let a = (context.get_and_inc()? as u64) << 56;
        let b = (context.get_and_inc()? as u64) << 48;
        let c = (context.get_and_inc()? as u64) << 40;
        let d = (context.get_and_inc()? as u64) << 32;
        let e = (context.get_and_inc()? as u64) << 24;
        let f = (context.get_and_inc()? as u64) << 16;
        let g = (context.get_and_inc()? as u64) << 8;
        let h = context.get_and_inc()? as u64;
        if context.endian {
            Ok(a | b | c | d | e | f | g | h)
        } else {
            Ok(h | g | f | e | d | c | b | a)
        }
    }
}

