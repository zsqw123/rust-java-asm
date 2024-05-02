use crate::err::AsmResult;

pub(crate) struct WriteContext {
    pub bytes: Vec<u8>,
}

impl WriteContext {
    pub fn bytes(&mut self) -> &mut Vec<u8> { &mut self.bytes }

    pub fn push<T: FromWriteContext<T>>(&mut self, from: T) -> AsmResult<()> {
        T::from_context(self, from)
    }
}

pub(crate) trait FromWriteContext<T> {
    fn from_context(context: &mut WriteContext, from: T) -> AsmResult<()>;
}

impl<T: FromWriteContext<T>> FromWriteContext<Vec<T>> for Vec<T> {
    fn from_context(context: &mut WriteContext, from: Vec<T>) -> AsmResult<()> {
        for item in from { context.push(item)?; };
        Ok(())
    }
}

impl FromWriteContext<u32> for u32 {
    fn from_context(context: &mut WriteContext, from: u32) -> AsmResult<()> {
        let bytes = context.bytes();
        bytes.push((from >> 24) as u8);
        bytes.push((from >> 16) as u8);
        bytes.push((from >> 8) as u8);
        bytes.push(from as u8);
        Ok(())
    }
}

impl FromWriteContext<u16> for u16 {
    fn from_context(context: &mut WriteContext, from: u16) -> AsmResult<()> {
        let bytes = context.bytes();
        bytes.push((from >> 8) as u8);
        bytes.push(from as u8);
        Ok(())
    }
}

impl FromWriteContext<u8> for u8 {
    fn from_context(context: &mut WriteContext, from: u8) -> AsmResult<()> {
        let bytes = context.bytes();
        bytes.push(from);
        Ok(())
    }
}



