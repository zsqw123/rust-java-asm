use crate::dex::{DUInt, Header};
use crate::dex::DexFileAccessor;
use crate::impls::jvms::r::{ReadContext, ReadFrom};
use crate::AsmResult;

impl<'a> DexFileAccessor<'a> {
    pub(crate) fn get_data_impl<T: ReadFrom>(
        &self, data_off: DUInt,
    ) -> AsmResult<T> {
        let endian = self.file.header.endian_tag;
        let mut read_context = if endian == Header::BIG_ENDIAN_TAG {
            ReadContext::big_endian(self.bytes)
        } else {
            ReadContext::little_endian(self.bytes)
        };
        read_context.index = data_off as usize;
        read_context.read()
    }
}
