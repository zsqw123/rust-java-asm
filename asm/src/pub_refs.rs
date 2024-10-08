use std::rc::Rc;

pub type StrRef = Rc<str>;

/// eg: java/lang/Class
pub type InternalNameRef = StrRef;

/// eg: java.lang.Class
pub type QualifiedNameRef = StrRef;

pub type DescriptorRef = StrRef;
