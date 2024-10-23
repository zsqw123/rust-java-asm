use crate::impls::ToStringRef;
use crate::node::InsnNode;
use crate::StrRef;
use std::fmt::Debug;

#[derive(Clone)]
pub struct SmaliNode {
    pub prefix: StrRef,
    pub children: Vec<SmaliNode>,
    pub postfix: Option<StrRef>,
}


#[macro_export]
macro_rules! smali {
    ($($arg:tt)*) => {
        SmaliNode::new(format!($($arg)*))
    }
}

pub use smali;
use crate::dex::DexFileAccessor;

impl SmaliNode {
    pub fn new(current: impl ToStringRef) -> Self {
        Self { prefix: current.to_ref(), children: Vec::new(), postfix: None }
    }

    pub fn new_with_children(
        current: impl ToStringRef, children: Vec<SmaliNode>,
    ) -> Self {
        Self { prefix: current.to_ref(), children, postfix: None }
    }

    pub fn new_with_children_and_postfix(
        prefix: impl ToStringRef, children: Vec<SmaliNode>, postfix: impl ToStringRef,
    ) -> Self {
        Self { prefix: prefix.to_ref(), children, postfix: Some(postfix.to_ref()) }
    }
}

impl SmaliNode {
    #[inline]
    pub fn add_attr(&mut self, key: impl ToStringRef, value: impl ToStringRef) {
        let attribute_child = format!("{} = {}", key.to_ref(), value.to_ref());
        self.add_child(SmaliNode::new(attribute_child));
    }

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
    
    fn render_internal(&self, ident_level: usize, result: &mut String) {
        let indent_str = "    ".repeat(ident_level);
        result.push_str(&indent_str);
        result.push_str(&self.prefix);
        if self.children.is_empty() && self.postfix.is_none() {
            return;
        }
        for child in &self.children {
            result.push('\n');
            child.render_internal(ident_level + 1, result);
        }
        if let Some(postfix) = &self.postfix {
            result.push('\n');
            result.push_str(&indent_str);
            result.push_str(&postfix);
        }
    }
}

impl Debug for InsnNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_smali().render(0))
    }
}

pub trait ToSmali {
    fn to_smali(&self) -> SmaliNode;
}

pub trait Dex2Smali {
    fn to_smali(&self, accessor: &DexFileAccessor) -> SmaliNode;
}

impl<T: ToStringRef> ToSmali for T {
    #[inline]
    fn to_smali(&self) -> SmaliNode {
        SmaliNode::new(self)
    }
} 
