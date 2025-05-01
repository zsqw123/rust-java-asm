use crate::AsmResult;
use std::sync::Arc;

pub(crate) trait ToArc<T> {
    fn arc(self) -> Arc<T>;
}

impl<T> ToArc<T> for T {
    fn arc(self) -> Arc<T> { Arc::new(self) }
}

pub(crate) trait VecEx<T> {
    fn map_res<R>(&self, f: impl FnMut(&T) -> AsmResult<R>) -> AsmResult<Vec<R>>;
}

impl<T> VecEx<T> for Vec<T> {
    #[inline]
    fn map_res<R>(&self, f: impl FnMut(&T) -> AsmResult<R>) -> AsmResult<Vec<R>> {
        self.iter().map(f).collect()
    }
}
