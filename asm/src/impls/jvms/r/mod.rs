use crate::err::AsmResult;
use crate::AsmErr;
pub use java_asm_macro::ReadFrom;
use std::array::from_fn;
use std::fmt::Debug;

pub(crate) mod jvms_reader;
pub(crate) mod util;
pub(crate) mod transform;
pub(crate) mod frame;

pub struct ReadContext<'a> {
    pub bytes: &'a [u8],
    pub index: usize,
    pub endian: bool, // true: big endian, false: little endian
}

impl<'a> ReadContext<'a> {
    pub fn big_endian(bytes: &'a [u8]) -> Self {
        Self { bytes, index: 0, endian: true }
    }

    pub fn little_endian(bytes: &'a [u8]) -> Self {
        Self { bytes, index: 0, endian: false }
    }
}

impl ReadContext<'_> {
    /// Move current index cursor to the next alignment position or keep it unchanged if it's
    /// already positioned at an alignment position.
    /// An alignment position's index should be a multiple of `alignment_byte_size`.
    #[inline]
    pub fn align(&mut self, alignment_byte_size: u16) {
        if alignment_byte_size == 0 { return; }
        let alignment_size = alignment_byte_size as usize;
        let current_index = self.index;
        let align = current_index % alignment_size;
        if align == 0 { return; }
        self.index = current_index + alignment_size - align;
    }

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
    pub fn read_vec<T: ReadFrom>(&mut self, vec_size: impl Into<usize>) -> AsmResult<Vec<T>> {
        let vec_size = vec_size.into();
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

impl<T: ReadFrom, const N: usize> ReadFrom for [T; N] {
    #[inline]
    fn read_from(context: &mut ReadContext) -> AsmResult<Self> {
        let array = from_fn_res(|_| context.read())?;
        Ok(array)
    }
}

/// Create an array of size `N` by calling the closure `cb` with the index of the element.
/// Returns first `Err` if any `Err` is returned by the `F`.
fn from_fn_res<T, const N: usize, F, E>(mut cb: F) -> Result<[T; N], E>
where
    E: Clone + Debug,
    F: FnMut(usize) -> Result<T, E>,
{
    let mut first_err = None;
    let res_arr: [Result<T, E>; N] = from_fn(|i| {
        match cb(i) {
            Ok(v) => Ok(v),
            Err(e) => {
                if first_err.is_none() { first_err = Some(e.clone()); }
                Err(e)
            },
        }
    });
    if let Some(e) = first_err { return Err(e); }
    let array: [T; N] = res_arr.map(Result::unwrap);
    Ok(array)
}
