use crate::err::AsmResult;

pub(crate) type ReadContext<'a> = (&'a [u8], &'a mut usize);

pub(crate) trait FromReadContext<T> {
    fn from_context(context: &ReadContext) -> AsmResult<T>;
}

pub(crate) trait UReader {
    fn read_u8(&self) -> u8;
    fn read_u16(&self) -> u16;
    fn read_u32(&self) -> u32;
    fn read_custom<T: FromReadContext<T>>(&self) -> AsmResult<T>;
    fn read_vec<T: FromReadContext<T>>(
        &self, vec_size: usize) -> AsmResult<Vec<T>>;
}

impl<'a> UReader for ReadContext<'a> {
    fn read_u8(&self) -> u8 {
        let (bytes, index) = *self;
        let content = bytes[*index];
        *index += 1;
        content
    }

    fn read_u16(&self) -> u16 {
        let (bytes, index) = *self;
        let h = (bytes[*index] as u16) << 8;
        let l = bytes[*index + 1] as u16;
        *index += 2;
        h | l
    }

    fn read_u32(&self) -> u32 {
        let (bytes, index) = *self;
        let a = (bytes[*index] as u32) << 24;
        let b = (bytes[*index + 1] as u32) << 16;
        let c = (bytes[*index + 2] as u32) << 8;
        let d = bytes[*index + 3] as u32;
        *index += 4;
        a | b | c | d
    }

    fn read_custom<T: FromReadContext<T>>(&self) -> AsmResult<T> {
        T::from_context(self)
    }

    fn read_vec<T: FromReadContext<T>>(
        &self, vec_size: usize) -> AsmResult<Vec<T>> {
        let mut vec = Vec::new();
        vec.reserve(vec_size);
        for i in 0..vec_size {
            vec[i] = self.read_custom()?;
        }
        Ok(vec)
    }
}
