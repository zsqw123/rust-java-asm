use std::sync::Arc;

pub type ConstStr = &'static str;

/// for impl, check [crate::impls::util::refs::ToStringRef]
pub type StrRef = Arc<str>;

/// e.g.: java/lang/Class
pub type InternalNameRef = StrRef;

/// e.g.: java.lang.Class
pub type QualifiedNameRef = StrRef;

/// e.g.: Ljava/lang/Class;
pub type DescriptorRef = StrRef;
