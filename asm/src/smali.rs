use crate::dex::DexFileAccessor;
use crate::{ConstStr, StrRef};

#[derive(Debug, Clone, Default, Hash, Eq, PartialEq)]
pub struct SmaliNode {
    pub tag: Option<ConstStr>,
    pub content: Vec<SmaliToken>,
    pub offset_hint: Option<u32>,
    pub children: Vec<SmaliNode>,
    pub end_tag: Option<ConstStr>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum SmaliToken {
    SourceInfo(StrRef),
    
    Raw(ConstStr),
    Op(ConstStr),

    LineStartOffsetMarker {
        // the offset of this instruction.
        // None if this isn't a instruction.
        offset: Option<u32>,
        // rendered text for this marker.
        raw: String,
    },
    Offset {
        relative: i32,
        absolute: u32,
    },

    Register(u16),
    RegisterRange(u16, u16),

    MemberName(StrRef),
    Descriptor(StrRef),
    Literal(StrRef),

    Other(StrRef),
}

#[inline]
pub fn stb() -> SmaliTokensBuilder {
    SmaliTokensBuilder::new()
}

pub struct SmaliTokensBuilder(Vec<SmaliToken>);

impl SmaliTokensBuilder {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    #[inline]
    pub fn push(mut self, token: SmaliToken) -> Self {
        self.0.push(token);
        self
    }

    #[inline]
    pub fn append(mut self, tokens: Vec<SmaliToken>) -> Self {
        self.0.extend(tokens);
        self
    }

    pub fn raw(self, raw: ConstStr) -> Self {
        self.push(SmaliToken::Raw(raw))
    }

    // build the smali node with no children.
    #[inline]
    pub fn s(self) -> SmaliNode {
        SmaliNode { content: self.0, ..Default::default() }
    }

    // build the smali node with children.
    #[inline]
    pub fn s_with_children(self, children: Vec<SmaliNode>) -> SmaliNode {
        SmaliNode { content: self.0, children, ..Default::default() }
    }

    #[inline]
    pub fn into_smali(self, children: Vec<SmaliNode>, postfix: ConstStr) -> SmaliNode {
        SmaliNode { content: self.0, children, end_tag: Some(postfix), ..Default::default() }
    }

    #[inline]
    pub fn op(self, op: ConstStr) -> Self {
        self.push(SmaliToken::Op(op))
    }

    #[inline]
    pub fn off(self, current: impl Into<u32>, relative: impl Into<i32>) -> Self {
        let relative = relative.into();
        let absolute = (current.into() as i32 + relative) as u32;
        self.push(SmaliToken::Offset { relative, absolute })
    }

    #[inline]
    pub fn v(self, reg: impl Into<u16>) -> Self {
        self.push(SmaliToken::Register(reg.into()))
    }

    #[inline]
    pub fn vr(self, start: impl Into<u16>, end: impl Into<u16>) -> Self {
        self.push(SmaliToken::RegisterRange(start.into(), end.into()))
    }

    #[inline]
    pub fn mn(self, name: StrRef) -> Self {
        self.push(SmaliToken::MemberName(name))
    }
    
    #[inline]
    pub fn d(self, desc: StrRef) -> Self {
        self.push(SmaliToken::Descriptor(desc))
    }

    #[inline]
    pub fn l(self, lit: StrRef) -> Self {
        self.push(SmaliToken::Literal(lit))
    }

    #[inline]
    pub fn other(self, other: StrRef) -> Self {
        self.push(SmaliToken::Other(other))
    }
}

pub fn tokens_to_raw(tokens: &[SmaliToken]) -> String {
    tokens.iter().map(|token| token.raw()).collect::<Vec<_>>().join(" ")
}

impl SmaliToken {
    pub fn raw(&self) -> String {
        match self {
            Self::SourceInfo(source) => format!("# source: {source}"),
            Self::Raw(tag) => tag.to_string(),
            Self::Op(op) => op.to_string(),
            Self::LineStartOffsetMarker { raw, .. } => raw.clone(),
            Self::Offset { relative, absolute } => {
                format!("@{absolute}({relative:+})")
            }
            Self::Register(reg) => format!("v{reg}"),
            Self::RegisterRange(start, end) => format!("v{start}..v{end}"),
            Self::MemberName(name) => name.to_string(),
            Self::Descriptor(desc) => desc.to_string(),
            Self::Literal(lit) => lit.to_string(),
            Self::Other(other) => other.to_string(),
        }
    }
}


#[macro_export]
macro_rules! raw_smali {
    ($($arg:tt)*) => {
        crate::smali::SmaliNode { content: vec![
            crate::smali::SmaliToken::Other(format!($($arg)*).to_ref())
        ], ..Default::default() }
    }
}

pub use raw_smali;
use crate::impls::ToStringRef;

impl SmaliNode {
    const fn raw(s: ConstStr) -> SmaliNode {
        SmaliNode {
            tag: Some(s),
            content: vec![],
            offset_hint: None,
            children: vec![],
            end_tag: None,
        }
    }

    pub const NULL: SmaliNode = Self::raw("null");
    pub const TRUE: SmaliNode = Self::raw("true");
    pub const FALSE: SmaliNode = Self::raw("false");


    #[inline]
    pub fn empty() -> Self {
        Default::default()
    }

    #[deprecated]
    pub fn new(current: String) -> Self {
        Self { content: vec![SmaliToken::Other(current.to_ref())], ..Default::default() }
    }

    #[deprecated]
    pub fn new_with_children(
        current: String, children: Vec<SmaliNode>,
    ) -> Self {
        Self { content: vec![SmaliToken::Other(current.to_ref())], children, ..Default::default() }
    }
}

impl SmaliNode {
    #[inline]
    pub fn add_child(&mut self, child: SmaliNode) {
        self.children.push(child);
    }

    #[inline]
    pub fn render(&self, indent: usize) -> String {
        let mut result = String::new();
        self.render_internal(indent, &mut result);
        result
    }
    
    #[inline]
    pub fn render_to_lines(&self) -> Vec<Vec<SmaliToken>> {
        self.render_to_lines_internal()
    }
}

pub trait ToSmali {
    fn to_smali(&self) -> SmaliNode;
}

pub trait Dex2Smali {
    fn to_smali(&self, accessor: &DexFileAccessor) -> SmaliNode;
}

impl<T: ToString> ToSmali for T {
    #[inline]
    fn to_smali(&self) -> SmaliNode {
        SmaliNode::new(self.to_string())
    }
}

