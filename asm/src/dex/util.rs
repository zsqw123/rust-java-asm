use std::fmt::Display;

#[derive(Copy, Debug, Clone, PartialEq, Eq)]
pub struct U4(pub u8); // top 4 bits is always be 0
#[derive(Copy, Debug, Clone, PartialEq, Eq)]
pub struct I4(pub i8); // top 5 bits should be the sign bits

impl Into<u16> for U4 {
    #[inline]
    fn into(self) -> u16 {
        self.0 as u16
    }
}

impl Display for U4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:}", self.0)
    }
}

impl Display for I4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:}", self.0)
    }
}

impl I4 {
    pub(crate) fn from_u4(v: U4) -> Self {
        I4((v.0 | 0xF0) as i8)
    }
}
