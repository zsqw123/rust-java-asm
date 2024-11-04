use std::rc::Rc;

pub type ConstStr = &'static str;

pub type StrRef = Rc<str>;

/// e.g.: java/lang/Class
pub type InternalNameRef = StrRef;

/// e.g.: java.lang.Class
pub type QualifiedNameRef = StrRef;

/// e.g.: Ljava/lang/Class;
pub type DescriptorRef = StrRef;

pub fn desc2fqn(type_descriptor: DescriptorRef) -> QualifiedNameRef {
    let mut fqn = type_descriptor.replace("/", ".");
    fqn.pop();
    fqn.remove(0);
    Rc::from(fqn)
}
