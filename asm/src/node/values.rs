use std::rc::Rc;
use crate::asm_type::Type;
use crate::node::element::{AnnotationNode, LabelNode};

#[derive(Clone, Debug)]
pub enum ConstValue {
    Invalid,
    Class(Rc<InternalName>),
    Member {
        class: Rc<InternalName>,
        name: Rc<String>,
        desc: Rc<Descriptor>,
    },
    String(Rc<String>),
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    NameAndType {
        name: Rc<String>,
        desc: Rc<Descriptor>,
    },
    MethodHandle {
        reference_kind: u8,
        reference_index: u16,
    },
    MethodType(Rc<Descriptor>),
    Dynamic {
        bootstrap_method_attr_index: u16,
        name: Rc<String>,
        desc: Rc<Descriptor>,
    },
    Module(Rc<String>),
    Package(Rc<String>),
}

#[derive(Clone, Debug)]
pub enum AnnotationValue {
    Const(Rc<ConstValue>),
    Enum(Rc<String>, Rc<String>),
    Class(Rc<InternalName>),
    Annotation(AnnotationNode),
    Array(Vec<AnnotationValue>),
}

#[derive(Clone, Debug)]
pub enum FieldInitialValue {
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    String(String),
}

#[derive(Clone, Debug)]
pub enum FrameValue {
    Top,
    Integer,
    Float,
    Long,
    Double,
    Null,
    UninitializedThis,
    Object(String),
    // indicate the offset of the NEW instruction that created the uninitialized object
    // being stored in the location
    Uninitialized(u16),
}

#[derive(Clone, Debug)]
pub enum BootstrapMethodArgument {
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    String(String),
    Class(Type),
    Handle(Handle),
}

#[derive(Clone, Debug)]
pub struct Handle {
    /// The kind of this handle. Should be one of the following value:
    /// [Opcodes::H_GETFIELD], [Opcodes::H_GETSTATIC], [Opcodes::H_PUTFIELD], [Opcodes::H_PUTSTATIC],
    /// [Opcodes::H_INVOKEVIRTUAL], [Opcodes::H_INVOKESTATIC], [Opcodes::H_INVOKESPECIAL],
    /// [Opcodes::H_NEWINVOKESPECIAL], [Opcodes::H_INVOKEINTERFACE].
    pub tag: u8,
    // The internal name of the class to which the field or method belongs.
    pub owner: String,
    pub name: String,
    // The descriptor of the field or method.
    pub desc: String,
    pub is_interface: bool,
}

#[derive(Clone, Debug)]
pub enum LdcConst {
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    String(String),
    Class(Type),
    Handle(Handle),
    ConstDynamic(ConstDynamic),
}

#[derive(Clone, Debug)]
pub struct ConstDynamic {
    pub name: String,
    pub desc: String,
    pub bsm: Handle,
    pub bsm_args: Vec<BootstrapMethodArgument>,
}

#[derive(Clone, Debug)]
pub struct LocalVariableInfo {
    pub start: LabelNode,
    pub length: u16,
    pub name: Rc<String>,
    pub desc: Rc<Descriptor>,
    pub index: u16,
}

#[derive(Clone, Debug)]
pub struct LocalVariableTypeInfo {
    pub start: LabelNode,
    pub length: u16,
    pub name: Rc<String>,
    pub signature: Rc<String>,
    pub index: u16,
}

/// eg: java/lang/Class
pub type InternalName = String;

/// eg: java.lang.Class
pub type QualifiedName = String;

pub type Descriptor = String;
