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
    pub fn byte_at(&self, index: usize) -> AsmResult<u8> {
        self.bytes.get(index).copied().ok_or_else(|| AsmErr::OutOfRange(index))
    }

    #[inline]
    pub fn get_cur(&self) -> AsmResult<u8> {
        self.byte_at(self.index)
    }

    #[inline]
    pub fn get_and_inc(&mut self) -> AsmResult<u8> {
        let current_index = self.index;
        let content = self.byte_at(current_index)?;
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

// Into<usize> might be implemented for u32 in future rust versions and rust compiler
// disallow us to implement trait like `IntoReadIndex` for `u32` and also implement
// for all `Into<usize>` types. So the best way is wrap this index into our struct.
//
// the compiler error is: upstream crates may add a new impl of trait `std::convert::From<u32>`
// for type `usize` in future versions
#[derive(Copy, Clone, Debug, PartialEq, Eq, ReadFrom, Default)]
pub struct U32BasedSize(pub u32);

impl Into<usize> for U32BasedSize {
    #[inline]
    fn into(self) -> usize {
        self.0 as usize
    }
}

macro_rules! impl_read_from {
    ($ty:ident, $size:literal) => {
        impl ReadFrom for $ty {
            #[inline]
            fn read_from(context: &mut ReadContext) -> AsmResult<Self> {
                let mut result: $ty = 0;
                for i in 0..$size {
                    let byte = context.get_and_inc()? as $ty;
                    if context.endian {
                        result = (result << 8) | byte;
                    } else {
                        result = byte << (8 * i) | result;
                    }
                }
                Ok(result)
            }
        }
    };
    ($ty:ident, $size:literal, $result_type:ty) => {
        impl ReadFrom for $ty {
            #[inline]
            fn read_from(context: &mut ReadContext) -> AsmResult<Self> {
                let mut result: $result_type = 0;
                for i in 0..$size {
                    let byte = context.get_and_inc()? as $result_type;
                    if context.endian {
                        result = (result << 8) | byte;
                    } else {
                        result = byte << (8 * i) | result;
                    }
                }
                Ok($ty(result))
            }
        }
    };
}


pub struct U24(pub u32);
pub struct U40(pub u64);
pub struct U48(pub u64);
pub struct U56(pub u64);

pub struct I24(pub i32);
pub struct I40(pub i64);
pub struct I48(pub i64);
pub struct I56(pub i64);

impl_read_from!(u16, 2);
impl_read_from!(U24, 3, u32);
impl_read_from!(u32, 4);
impl_read_from!(U40, 5, u64);
impl_read_from!(U48, 6, u64);
impl_read_from!(U56, 7, u64);
impl_read_from!(u64, 8);

impl_read_from!(i8, 1);
impl_read_from!(i16, 2);
impl_read_from!(I24, 3, i32);
impl_read_from!(i32, 4);
impl_read_from!(I40, 5, i64);
impl_read_from!(I48, 6, i64);
impl_read_from!(I56, 7, i64);
impl_read_from!(i64, 8);

impl ReadFrom for u8 {
    #[inline]
    fn read_from(context: &mut ReadContext) -> AsmResult<u8> {
        context.get_and_inc()
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
