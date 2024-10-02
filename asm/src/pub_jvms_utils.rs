use crate::{AsmResult, StrRef};

pub fn mutf8_to_utf8(mutf8: &[u8]) -> AsmResult<Vec<u8>> {
    crate::impls::mutf8_to_utf8(mutf8)
}

pub fn mutf8_to_string(mutf8: &[u8]) -> AsmResult<StrRef> {
    crate::impls::mutf8_to_string(mutf8)
}

pub fn utf8_to_mutf8(utf8: &[u8]) -> AsmResult<Vec<u8>> {
    crate::impls::utf8_to_mutf8(utf8)
}

