use crate::{AsmErr, AsmResult, ComputableAccessor, ComputableOwner, ComputableSizedVec, ComputableSizedVecAccessor, ComputableSizedVecOwner};
use std::cell::UnsafeCell;
use std::rc::Rc;
use crate::computable::{Computable, InnerValue};

impl<V> Default for InnerValue<V> {
    fn default() -> Self { InnerValue::UnInit }
}



/// manually impl due to rust default impl of Default trait requires `V: Default` 
impl<V> Default for Computable<V> {
    fn default() -> Self {
        Self { inner_value: Default::default() }
    }
}

impl<V: Clone> Clone for Computable<V> {
    fn clone(&self) -> Self {
        let inner_value = unsafe { &mut *self.inner_value.get() };
        let cloned_inner = inner_value.clone();
        Self { inner_value: UnsafeCell::new(cloned_inner) }
    }
}



impl<T, V> ComputableAccessor<V> for T
where
    T: ComputableOwner<V>,
{
    fn force(&self) -> AsmResult<Rc<V>> {
        self.computable_ref().force_with_fn(|| self.compute())
    }
}

impl<V> Computable<V> {
    pub fn force_with_fn(&self, f: impl FnOnce() -> AsmResult<V>) -> AsmResult<Rc<V>> {
        let inner_value_ptr = self.inner_value.get();
        let inner = unsafe { &mut *inner_value_ptr };
        match inner {
            InnerValue::UnInit => {
                let value = Rc::new(f()?);
                let copied = Rc::clone(&value);
                *inner = InnerValue::Initialized(value);
                Ok(copied)
            }
            InnerValue::Initialized(value) => Ok(Rc::clone(value))
        }
    }
}

impl<V> ComputableSizedVec<V> {
    pub fn new(size: usize) -> Self {
        let mut vec = Vec::with_capacity(size);
        vec.resize_with(size, || Default::default());
        Self { vec_ref: vec }
    }
}

impl<T, V> ComputableSizedVecAccessor<V> for T
where
    T: ComputableSizedVecOwner<V>,
{
    fn get_or_compute(&self, index: usize) -> AsmResult<Rc<V>> {
        self.computable_vec().vec_ref.get(index)
            .ok_or_else(|| AsmErr::OutOfRange(index))?
            .force_with_fn(|| self.compute(index))
    }
}
