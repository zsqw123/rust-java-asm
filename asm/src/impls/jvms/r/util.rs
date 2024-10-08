use crate::constants::Constants;
use crate::err::{AsmErr, AsmResult};
use crate::impls::{mutf8_to_string};
use crate::jvms::element::{Const, CPInfo};
use crate::StrRef;

pub fn read_utf8_from_cp(index: usize, cp: &Vec<CPInfo>) -> AsmResult<StrRef> {
    let cp_info = &cp[index];
    let CPInfo { tag, info } = cp_info;
    let tag = *tag;
    if tag != Constants::CONSTANT_Utf8 {
        return AsmErr::IllegalFormat(
            format!("cannot read utf8 from constant pool, current cp tag: {}, index: {}", tag, index)
        ).e();
    };
    if let Const::Utf8 { bytes, .. } = info {
        return mutf8_to_string(bytes);
    };
    AsmErr::IllegalFormat(
        format!("cannot read utf8 from constant pool, index: {}", index)
    ).e()
}
