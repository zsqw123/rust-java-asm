use std::rc::Rc;
use crate::AsmResult;

pub(crate) trait ToRc<T> {
    fn rc(self) -> Rc<T>;
}

impl<T> ToRc<T> for T {
    fn rc(self) -> Rc<T> { Rc::new(self) }
}

pub(crate) trait VecEx<T> {
    fn map_res<R>(&self, f: impl FnMut(&T) -> AsmResult<R>) -> AsmResult<Vec<R>>;
    fn map<R>(&self, f: impl FnMut(&T) -> R) -> Vec<R>;
}

impl<T> VecEx<T> for Vec<T> {
    #[inline]
    fn map_res<R>(&self, f: impl FnMut(&T) -> AsmResult<R>) -> AsmResult<Vec<R>> {
        self.iter().map(f).collect()
    }

    #[inline]
    fn map<R>(&self, f: impl FnMut(&T) -> R) -> Vec<R> {
        self.iter().map(f).collect()
    }
}
