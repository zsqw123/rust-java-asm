#[derive(Copy, Debug, Clone, PartialEq, Eq)]
pub struct U4(pub u8); // top 4 bits is always be 0
#[derive(Copy, Debug, Clone, PartialEq, Eq)]
pub struct I4(pub i8); // top 5 bits should be the sign bits

impl I4 {
    pub(crate) fn from_u4(v: U4) -> Self {
        I4((v.0 | 0xF0) as i8)
    }
}
