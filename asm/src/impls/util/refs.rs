use std::rc::Rc;
use crate::StrRef;


pub trait ToStringRef {
    fn to_ref(&self) -> StrRef;
}

impl ToStringRef for str {
    #[inline]
    fn to_ref(&self) -> StrRef {
        Rc::from(self)
    }
}

impl ToStringRef for String {
    #[inline]
    fn to_ref(&self) -> StrRef {
        Rc::from(self.as_ref())
    }
}

