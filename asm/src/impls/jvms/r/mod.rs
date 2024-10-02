pub use java_asm_macro::ReadFrom;

use crate::AsmErr;
use crate::err::AsmResult;

pub(crate) mod jvms_reader;
pub(crate) mod util;
pub(crate) mod transform;
pub(crate) mod frame;

pub struct ReadContext<'a> {
    pub bytes: &'a [u8],
    pub index: &'a mut usize,
}

impl ReadContext<'_> {
    #[inline]
    pub fn get_and_inc(&mut self) -> AsmResult<u8> {
        let current_index = *self.index;
        let content = self.bytes.get(current_index).copied()
            .ok_or_else(|| AsmErr::OutOfRange(current_index))?;
        *self.index += 1;
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
        Ok(h | l)
    }
}

impl ReadFrom for u32 {
    #[inline]
    fn read_from(context: &mut ReadContext) -> AsmResult<u32> {
        let a = (context.get_and_inc()? as u32) << 24;
        let b = (context.get_and_inc()? as u32) << 16;
        let c = (context.get_and_inc()? as u32) << 8;
        let d = context.get_and_inc()? as u32;
        Ok(a | b | c | d)
    }
}



