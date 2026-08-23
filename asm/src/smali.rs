use crate::dex::DexFileAccessor;
use crate::{ConstStr, StrRef};
use std::fmt::{self, Display, Formatter};
use std::sync::Arc;

/// A bytecode name with separate storage and presentation forms.
///
/// - `raw_name` is stored in the class/DEX file and is used for lookup.
/// - `display_name` is shown to users and initially equals `raw_name`.
/// - Importing a mapping may replace `display_name` with its pre-obfuscation name.
#[derive(Debug, Clone, Default, Hash, Eq, PartialEq)]
pub struct MappedName {
    pub raw_name: StrRef,
    pub display_name: StrRef,
}

impl MappedName {
    #[inline]
    pub fn new(raw_name: StrRef) -> Self {
        Self { display_name: Arc::clone(&raw_name), raw_name }
    }
}

impl From<StrRef> for MappedName {
    fn from(raw_name: StrRef) -> Self {
        Self::new(raw_name)
    }
}

impl Display for MappedName {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.display_name)
    }
}

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

    MemberName(MappedName),
    Descriptor(MappedName),
    SourceLine {
        raw_line: u32,
        display_line: u32,
    },
    Literal(StrRef),

    Other(StrRef),
}

#[inline]
pub fn stb() -> SmaliTokensBuilder {
    SmaliTokensBuilder::new()
}

#[derive(Default)]
pub struct SmaliTokensBuilder(Vec<SmaliToken>);

impl SmaliTokensBuilder {
    pub fn new() -> Self {
        Self::default()
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
        self.push(SmaliToken::MemberName(name.into()))
    }
    
    #[inline]
    pub fn d(self, desc: StrRef) -> Self {
        self.push(SmaliToken::Descriptor(desc.into()))
    }

    #[inline]
    pub fn source_line(self, raw_line: u32) -> Self {
        self.push(SmaliToken::SourceLine { raw_line, display_line: raw_line })
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

pub fn write_tokens_display(
    tokens: &[SmaliToken], output: &mut impl fmt::Write,
) -> fmt::Result {
    for (index, token) in tokens.iter().enumerate() {
        if index > 0 {
            output.write_char(' ')?;
        }
        write!(output, "{token}")?;
    }
    Ok(())
}

pub fn tokens_to_display(tokens: &[SmaliToken]) -> String {
    let mut output = String::new();
    let _ = write_tokens_display(tokens, &mut output);
    output
}

impl SmaliToken {
    #[inline]
    pub fn display_text(&self) -> String {
        self.to_string()
    }
}

impl Display for SmaliToken {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceInfo(source) => write!(formatter, "# source: {source}"),
            Self::Raw(tag) => formatter.write_str(tag),
            Self::Op(op) => formatter.write_str(op),
            Self::LineStartOffsetMarker { raw, .. } => formatter.write_str(raw),
            Self::Offset { relative, absolute } => {
                write!(formatter, "@{absolute}({relative:+})")
            }
            Self::Register(reg) => write!(formatter, "v{reg}"),
            Self::RegisterRange(start, end) => write!(formatter, "v{start}..v{end}"),
            Self::MemberName(name) | Self::Descriptor(name) => Display::fmt(name, formatter),
            Self::SourceLine { display_line, .. } => Display::fmt(display_line, formatter),
            Self::Literal(lit) | Self::Other(lit) => formatter.write_str(lit),
        }
    }
}


#[macro_export]
macro_rules! raw_smali {
    ($($arg:tt)*) => {
        $crate::smali::SmaliNode { content: vec![
            $crate::smali::SmaliToken::Other(format!($($arg)*).to_ref())
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
        stb().other(self.to_string().to_ref()).s()
    }
}

