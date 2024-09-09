pub(crate) mod jvms_reader;
pub(crate) mod util;
pub(crate) mod transform;
pub(crate) mod frame;

pub struct ReadContext<'a> {
    pub bytes: &'a [u8],
    pub index: &'a mut usize,
}

impl ReadContext<'_> {
    pub fn copy(&mut self) -> ReadContext {
        ReadContext { bytes: self.bytes, index: self.index }
    }

    #[inline]
    pub fn paired(&mut self) -> (&[u8], &mut usize) {
        (self.bytes, self.index)
    }
}

pub use java_asm_macro::FromReadContext;
use crate::err::AsmResult;

pub trait FromReadContext<T> {
    fn from_context(context: &mut ReadContext) -> AsmResult<T>;
}

impl ReadContext<'_> {
    pub fn read<T: FromReadContext<T>>(&mut self) -> AsmResult<T> {
        T::from_context(self)
    }

    pub fn read_vec<T: FromReadContext<T>>(&mut self, vec_size: usize) -> AsmResult<Vec<T>> {
        let mut vec = Vec::<T>::with_capacity(vec_size);
        for _ in 0..vec_size {
            vec.push(self.copy().read()?);
        }
        Ok(vec)
    }
}

impl FromReadContext<u8> for u8 {
    fn from_context(context: &mut ReadContext) -> AsmResult<u8> {
        let (bytes, index) = context.paired();
        let content = bytes[*index];
        *index += 1;
        Ok(content)
    }
}

impl FromReadContext<u16> for u16 {
    fn from_context(context: &mut ReadContext) -> AsmResult<u16> {
        let (bytes, index) = context.paired();
        let h = (bytes[*index] as u16) << 8;
        let l = bytes[*index + 1] as u16;
        *index += 2;
        Ok(h | l)
    }
}

impl FromReadContext<u32> for u32 {
    fn from_context(context: &mut ReadContext) -> AsmResult<u32> {
        let (bytes, index) = context.paired();
        let a = (bytes[*index] as u32) << 24;
        let b = (bytes[*index + 1] as u32) << 16;
        let c = (bytes[*index + 2] as u32) << 8;
        let d = bytes[*index + 3] as u32;
        *index += 4;
        Ok(a | b | c | d)
    }
}



