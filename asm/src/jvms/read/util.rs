use crate::constants::Constants;
use java_asm_internal::err::{AsmErr, AsmResult};
use crate::jvms::element::{Const, CPInfo};
use crate::util::mutf8_to_utf8;

pub fn read_utf8_from_cp(index: usize, cp: &Vec<CPInfo>) -> AsmResult<String> {
    let cp_info = &cp[index];
    let CPInfo { tag, info } = cp_info;
    let tag = *tag;
    if tag != Constants::CONSTANT_Utf8 {
        return Err(AsmErr::IllegalArgument(
            format!("cannot read utf8 from constant pool, current cp tag: {}, index: {}", tag, index)
        ));
    };
    if let Const::Utf8 { bytes, .. } = info {
        let utf8 = mutf8_to_utf8(bytes)?;
        return match String::from_utf8(utf8) {
            Ok(str) => Ok(str),
            Err(e) => Err(AsmErr::ReadUTF8(e.to_string())),
        };
    };
    Err(AsmErr::IllegalArgument(
        format!("cannot read utf8 from constant pool, index: {}", index)
    ))
}