use crate::{AsmErr, AsmResult};
use std::cell::UnsafeCell;
use std::rc::Rc;

pub enum InnerValue<V> {
    UnInit,
    Initialized(V),
}

impl<V> Default for InnerValue<V> {
    fn default() -> Self { InnerValue::UnInit }
}

pub struct ComputableSizedVec<V> {
    vec_ref: Vec<UnsafeCell<InnerValue<Rc<V>>>>,
}

impl<V> ComputableSizedVec<V> {
    pub fn new(size: usize) -> Self {
        let mut vec = Vec::with_capacity(size);
        vec.resize_with(size, || Default::default());
        Self { vec_ref: vec }
    }
}

pub trait ComputableSizedVecOwner<V> {
    fn computable_vec(&self) -> &ComputableSizedVec<V>;
    fn compute(&self, index: usize) -> AsmResult<V>;
}

pub trait ComputableSizedVecAccessor<V> {
    /// Get the value at the index, compute value if needed.
    /// Returns None if the `index` is out of range. 
    fn get_or_compute(&self, index: usize) -> AsmResult<Rc<V>>;
}

impl<T, V> ComputableSizedVecAccessor<V> for T
where
    T: ComputableSizedVecOwner<V>,
{
    fn get_or_compute(&self, index: usize) -> AsmResult<Rc<V>> {
        let cell = self.computable_vec().vec_ref.get(index)
            .ok_or_else(|| AsmErr::OutOfRange(index))?;
        let inner = unsafe { &mut *cell.get() };
        match inner {
            InnerValue::UnInit => {
                let value = Rc::new(self.compute(index)?);
                let copied = Rc::clone(&value);
                *inner = InnerValue::Initialized(value);
                Ok(copied)
            }
            InnerValue::Initialized(value) => Ok(Rc::clone(value))
        }
    }
}
