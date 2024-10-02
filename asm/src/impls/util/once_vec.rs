pub struct OnceAsmVec<T> {
    tag: &'static str,
    vec: OnceCell<Vec<T>>,
}

impl<T> OnceAsmVec<T> {
    pub fn new(tag: &'static str) -> Self {
        Self { tag, vec: OnceCell::new() }
    }

    pub fn to_vec(self) -> Vec<T> {
        self.vec.into_inner().unwrap_or_default()
    }
}

impl<T: Debug> OnceAsmVec<T> {
    pub fn put(&self, vec: Vec<T>) -> AsmResult<()> {
        self.vec.set(vec).map_err(|origin| {
            let err_msg = format!("{} already initialized with {:?}", self.tag, origin);
            AsmErr::IllegalFormat(err_msg)
        })
    }
}

macro_rules! once_vec_builder {
    {
        $(let $name:ident: $vecType:ty;)*
    } => {
        $(
            let $name = OnceAsmVec::<$vecType>::new(stringify!($name));
        )*
    };
}

macro_rules! once_vec_unpack {
    { $($name:ident),* } => {
        $(
            let $name = $name.to_vec();
        )*
    };
}

use std::cell::OnceCell;
use std::fmt::Debug;
pub(crate) use once_vec_builder;
pub(crate) use once_vec_unpack;
use crate::{AsmErr, AsmResult};
