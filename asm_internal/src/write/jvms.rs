pub use java_asm_macro::IntoWriteContext;

pub struct WriteContext {
    pub bytes: Vec<u8>,
}

impl WriteContext {
    pub fn bytes(&mut self) -> &mut Vec<u8> { &mut self.bytes }

    pub fn push<T: IntoWriteContext<T>>(&mut self, from: T) {
        T::into_context(self, from)
    }
}

pub trait IntoWriteContext<T> {
    #[allow(clippy::wrong_self_convention)]
    fn into_context(context: &mut WriteContext, into: T);
}

impl<T: IntoWriteContext<T>> IntoWriteContext<Vec<T>> for Vec<T> {
    fn into_context(context: &mut WriteContext, from: Vec<T>) {
        for item in from { context.push(item); };
    }
}

impl IntoWriteContext<u32> for u32 {
    fn into_context(context: &mut WriteContext, from: u32) {
        let bytes = context.bytes();
        bytes.push((from >> 24) as u8);
        bytes.push((from >> 16) as u8);
        bytes.push((from >> 8) as u8);
        bytes.push(from as u8);
    }
}

impl IntoWriteContext<u16> for u16 {
    fn into_context(context: &mut WriteContext, from: u16) {
        let bytes = context.bytes();
        bytes.push((from >> 8) as u8);
        bytes.push(from as u8);
    }
}

impl IntoWriteContext<u8> for u8 {
    fn into_context(context: &mut WriteContext, from: u8) {
        let bytes = context.bytes();
        bytes.push(from);
    }
}



