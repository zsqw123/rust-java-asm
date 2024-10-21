use crate::StrRef;
use std::rc::Rc;


pub trait ToStringRef {
    fn to_ref(&self) -> StrRef;
}

impl<T: ToStringRef> ToStringRef for &T {
    #[inline]
    fn to_ref(&self) -> StrRef { (*self).to_ref() }
}

impl ToStringRef for StrRef {
    #[inline]
    fn to_ref(&self) -> StrRef { Rc::clone(self) }
}

impl ToStringRef for str {
    #[inline]
    fn to_ref(&self) -> StrRef { Rc::from(self) }
}

impl ToStringRef for String {
    #[inline]
    fn to_ref(&self) -> StrRef { Rc::from(self.as_str()) }
}

macro_rules! to_str_ref_impls {
    { $($ty:ty,)* } => {
        $(
            impl ToStringRef for $ty {
                #[inline]
                fn to_ref(&self) -> StrRef { Rc::from(self.to_string()) }
            }
        )*
    };
}

to_str_ref_impls!(
    i8, i16, i32, i64, i128, isize,
    u8, u16, u32, u64, u128, usize,
    f32, f64, char, bool,
);
