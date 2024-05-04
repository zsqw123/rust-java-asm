use crate::err::AsmResult;

pub struct WriteContext {
    pub bytes: Vec<u8>,
}

impl WriteContext {
    pub fn bytes(&mut self) -> &mut Vec<u8> { &mut self.bytes }

    pub fn push<T: IntoWriteContext<T>>(&mut self, from: T) -> AsmResult<()> {
        T::into_context(self, from)
    }
}

pub trait IntoWriteContext<T> {
    #[allow(clippy::wrong_self_convention)]
    fn into_context(context: &mut WriteContext, into: T) -> AsmResult<()>;
}

impl<T: IntoWriteContext<T>> IntoWriteContext<Vec<T>> for Vec<T> {
    fn into_context(context: &mut WriteContext, from: Vec<T>) -> AsmResult<()> {
        for item in from { context.push(item)?; };
        Ok(())
    }
}

impl IntoWriteContext<u32> for u32 {
    fn into_context(context: &mut WriteContext, from: u32) -> AsmResult<()> {
        let bytes = context.bytes();
        bytes.push((from >> 24) as u8);
        bytes.push((from >> 16) as u8);
        bytes.push((from >> 8) as u8);
        bytes.push(from as u8);
        Ok(())
    }
}

impl IntoWriteContext<u16> for u16 {
    fn into_context(context: &mut WriteContext, from: u16) -> AsmResult<()> {
        let bytes = context.bytes();
        bytes.push((from >> 8) as u8);
        bytes.push(from as u8);
        Ok(())
    }
}

impl IntoWriteContext<u8> for u8 {
    fn into_context(context: &mut WriteContext, from: u8) -> AsmResult<()> {
        let bytes = context.bytes();
        bytes.push(from);
        Ok(())
    }
}



