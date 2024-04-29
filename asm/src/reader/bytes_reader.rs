use crate::err::AsmResult;

pub(crate) type ReadContext<'a> = (&'a [u8], &'a mut usize);

pub(crate) trait FromReadContext<T> {
    fn from_context(context: &ReadContext) -> AsmResult<T>;
}

pub(crate) trait UReader {
    fn read<T: FromReadContext<T>>(&self) -> AsmResult<T>;
    fn read_vec<T: FromReadContext<T>>(
        &self, vec_size: usize) -> AsmResult<Vec<T>>;
}

impl<'a> UReader for ReadContext<'a> {
    fn read<T: FromReadContext<T>>(&self) -> AsmResult<T> {
        T::from_context(self)
    }

    fn read_vec<T: FromReadContext<T>>(
        &self, vec_size: usize) -> AsmResult<Vec<T>> {
        let mut vec = Vec::new();
        vec.reserve(vec_size);
        for i in 0..vec_size {
            vec[i] = self.read()?;
        }
        Ok(vec)
    }
}
