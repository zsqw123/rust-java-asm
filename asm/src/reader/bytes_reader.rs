use crate::err::AsmResult;

pub(crate) struct ReadContext<'a> {
    pub bytes: &'a [u8],
    pub index: &'a mut usize,
}

impl ReadContext<'_> {
    pub fn copy(&mut self) -> ReadContext {
        ReadContext { bytes: self.bytes, index: self.index }
    }

    pub fn paired(&mut self) -> (&[u8], &mut usize) {
        (self.bytes, self.index)
    }
}

pub(crate) trait FromReadContext<T> {
    fn from_context(context: &mut ReadContext) -> AsmResult<T>;
}

impl ReadContext<'_> {
    pub fn read<T: FromReadContext<T>>(&mut self) -> AsmResult<T> {
        T::from_context(self)
    }

    pub fn read_vec<T: FromReadContext<T>>(&mut self, vec_size: usize) -> AsmResult<Vec<T>> {
        let mut vec = Vec::<T>::new();
        vec.reserve(vec_size);
        for i in 0..vec_size {
            vec[i] = self.copy().read()?;
        }
        Ok(vec)
    }
}
