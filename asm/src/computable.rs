use crate::AsmResult;
use std::cell::UnsafeCell;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum InnerValue<V> {
    UnInit,
    Initialized(V),
}


#[derive(Debug)]
pub struct Computable<V> {
    pub(crate) inner_value: UnsafeCell<InnerValue<Arc<V>>>,
}

pub trait ComputableOwner<V> {
    fn computable_ref(&self) -> &Computable<V>;
    fn compute(&self) -> AsmResult<V>;
}


pub trait ComputableAccessor<V> {
    fn force(&self) -> AsmResult<Arc<V>>;
}

pub struct ComputableSizedVec<V> {
    pub(crate) vec_ref: Vec<Computable<V>>,
}

pub trait ComputableSizedVecOwner<V> {
    fn computable_vec(&self) -> &ComputableSizedVec<V>;
    fn compute(&self, index: usize) -> AsmResult<V>;
}

pub trait ComputableSizedVecAccessor<V> {
    /// Get the value at the index, compute value if needed.
    /// Returns [AsmErr::OutOfRange] if the `index` is out of range. 
    fn get_or_compute(&self, index: usize) -> AsmResult<Arc<V>>;
}
