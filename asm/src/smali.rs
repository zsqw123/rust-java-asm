use crate::impls::ToStringRef;
use crate::StrRef;

pub struct SmaliNode {
    pub prefix: StrRef,
    pub children: Vec<SmaliNode>,
    pub postfix: Option<StrRef>,
}

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
        prefix: impl ToStringRef, children: Vec<SmaliNode>, postfix: Option<StrRef>,
    ) -> Self {
        Self { prefix: prefix.to_ref(), children, postfix }
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
}

pub trait ToSmali {
    fn to_smali(&self) -> SmaliNode;
}

impl<T: ToStringRef> ToSmali for T {
    #[inline]
    fn to_smali(&self) -> SmaliNode {
        SmaliNode::new(self)
    }
} 
