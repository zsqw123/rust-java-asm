pub struct WriteContext {
    pub bytes: Vec<u8>,
}

impl WriteContext {
    #[inline]
    pub fn bytes(&mut self) -> &mut Vec<u8> { &mut self.bytes }

    #[inline]
    pub fn write<T: WriteInto>(&mut self, from: T) {
        T::write_into(self, from)
    }
}

pub trait WriteInto where Self: Sized {
    fn write_into(context: &mut WriteContext, into: Self);
}

impl<T: WriteInto> WriteInto for Vec<T> {
    #[inline]
    fn write_into(context: &mut WriteContext, from: Vec<T>) {
        for item in from { context.write(item); };
    }
}

impl WriteInto for u32 {
    #[inline]
    fn write_into(context: &mut WriteContext, from: u32) {
        let bytes = context.bytes();
        bytes.push((from >> 24) as u8);
        bytes.push((from >> 16) as u8);
        bytes.push((from >> 8) as u8);
        bytes.push(from as u8);
    }
}

impl WriteInto for u16 {
    #[inline]
    fn write_into(context: &mut WriteContext, from: u16) {
        let bytes = context.bytes();
        bytes.push((from >> 8) as u8);
        bytes.push(from as u8);
    }
}

impl WriteInto for u8 {
    #[inline]
    fn write_into(context: &mut WriteContext, from: u8) {
        let bytes = context.bytes();
        bytes.push(from);
    }
}

