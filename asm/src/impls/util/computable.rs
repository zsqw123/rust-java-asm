use crate::{AsmErr, AsmResult};
use std::cell::UnsafeCell;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub enum InnerValue<V> {
    UnInit,
    Initialized(V),
}

impl<V> Default for InnerValue<V> {
    fn default() -> Self { InnerValue::UnInit }
}

#[derive(Debug)]
pub struct Computable<V> {
    inner_value: UnsafeCell<InnerValue<Rc<V>>>,
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


pub trait ComputableAccessor<V> {
    fn force(&self) -> AsmResult<Rc<V>>;
}

pub trait ComputableOwner<V> {
    fn computable_ref(&self) -> &Computable<V>;
    fn compute(&self) -> AsmResult<V>;
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

pub struct ComputableSizedVec<V> {
    vec_ref: Vec<Computable<V>>,
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
    /// Returns [AsmErr::OutOfRange] if the `index` is out of range. 
    fn get_or_compute(&self, index: usize) -> AsmResult<Rc<V>>;
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
