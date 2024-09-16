use std::rc::Rc;

use crate::impls::jvms::r::util::ToRcRef;
use crate::node::element::{AnnotationNode, LabelNode};

#[derive(Clone, Debug)]
pub enum ConstValue {
    Invalid,
    Class(InternalNameRef),
    Member {
        class: InternalNameRef,
        name: StrRef,
        desc: DescriptorRef,
    },
    String(StrRef),
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    NameAndType {
        name: StrRef,
        desc: DescriptorRef,
    },
    MethodHandle(Handle),
    MethodType(DescriptorRef),
    Dynamic {
        bootstrap_method_attr_index: u16,
        name: StrRef,
        desc: DescriptorRef,
    },
    Module(StrRef),
    Package(StrRef),
}

#[derive(Clone, Debug)]
pub enum AnnotationValue {
    Const(Rc<ConstValue>),
    Enum(StrRef, StrRef),
    Class(InternalNameRef),
    Annotation(AnnotationNode),
    Array(Vec<AnnotationValue>),
}

#[derive(Clone, Debug)]
pub enum FieldInitialValue {
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    String(StrRef),
}

#[derive(Clone, Debug)]
pub enum FrameAttributeValue {
    SameFrame {
        offset_delta: u8,
    },
    SameFrameExtended {
        offset_delta: u16,
    },
    SameLocals1StackItemFrame {
        offset_delta: u8,
        stack: FrameValue,
    },
    SameLocals1StackItemFrameExtended {
        offset_delta: u16,
        stack: FrameValue,
    },
    ChopFrame {
        chop_count: u8,
        offset_delta: u16,
    },
    AppendFrame {
        offset_delta: u16,
        append_locals: Vec<FrameValue>,
    },
    FullFrame {
        offset_delta: u16,
        locals: Vec<FrameValue>,
        stack: Vec<FrameValue>,
    },
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
    Object(StrRef),
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
    String(StrRef),
    Class(InternalNameRef),
    Handle(Handle),
}

#[derive(Clone, Debug)]
pub struct Handle {
    /// The kind of this handle. Should be one of the following value:
    /// [Opcodes::H_GETFIELD], [Opcodes::H_GETSTATIC], [Opcodes::H_PUTFIELD], [Opcodes::H_PUTSTATIC],
    /// [Opcodes::H_INVOKEVIRTUAL], [Opcodes::H_INVOKESTATIC], [Opcodes::H_INVOKESPECIAL],
    /// [Opcodes::H_NEWINVOKESPECIAL], [Opcodes::H_INVOKEINTERFACE].
    pub reference_kind: u8,
    // The internal name of the class to which the field or method belongs.
    pub owner: StrRef,
    pub name: StrRef,
    // The descriptor of the field or method.
    pub desc: StrRef,
}

#[derive(Clone, Debug)]
pub struct ConstDynamic {
    pub name: StrRef,
    pub desc: StrRef,
    pub bsm: Handle, // bootstrap method handle
    pub bsm_args: Vec<BootstrapMethodArgument>,
}

#[derive(Clone, Debug)]
pub struct LocalVariableInfo {
    pub start: LabelNode,
    pub length: u16,
    pub name: StrRef,
    pub desc: DescriptorRef,
    /// The value of the index item must be a valid index into the local variable array of the current frame. 
    /// The given local variable is at index in the local variable array of the current frame.
    /// If the given local variable is of type double or long, it occupies both index and index + 1.
    pub index: u16,
}

#[derive(Clone, Debug)]
pub struct LocalVariableTypeInfo {
    pub start: LabelNode,
    pub length: u16,
    pub name: StrRef,
    pub signature: StrRef,
    /// The value of the index item must be a valid index into the local variable array of the current frame. 
    /// The given local variable is at index in the local variable array of the current frame.
    /// If the given local variable is of type double or long, it occupies both index and index + 1.
    pub index: u16,
}

#[derive(Clone, Debug)]
pub struct ModuleAttrValue {
    pub name: StrRef,
    pub access: u16,
    pub version: Option<StrRef>,
    pub requires: Vec<ModuleRequireValue>,
    pub exports: Vec<ModuleExportValue>,
    pub opens: Vec<ModuleOpenValue>,
    pub uses: Vec<InternalNameRef>,
    pub provides: Vec<ModuleProvidesValue>,
}

#[derive(Clone, Debug)]
pub struct ModuleRequireValue {
    /// The fully qualified name (using dots) of the dependence.
    pub module: QualifiedNameRef,

    /// The access flags of the required module, valid values are [Opcodes::ACC_TRANSITIVE], 
    /// [Opcodes::ACC_STATIC_PHASE], [Opcodes::ACC_SYNTHETIC], [Opcodes::ACC_MANDATED]
    pub access: u16,

    /// The version of the required module. May be [None].
    pub version: Option<StrRef>,
}

#[derive(Clone, Debug)]
pub struct ModuleExportValue {
    /// The internal name of the exported package. (see [Type::get_internal_name]).
    pub package: InternalNameRef,

    /// The access flags of the exported package, valid values are [Opcodes::ACC_SYNTHETIC], 
    /// [Opcodes::ACC_MANDATED]
    pub access: u16,

    /// The list of modules that can access this exported package, 
    /// specified with fully qualified names (using dots)
    pub modules: Vec<QualifiedNameRef>,
}

#[derive(Clone, Debug)]
pub struct ModuleOpenValue {
    /// The internal name of the opened package. (see [Type::get_internal_name]).
    pub package: InternalNameRef,

    /// The access flags of the opened package, valid values are [Opcodes::ACC_SYNTHETIC], 
    /// [Opcodes::ACC_MANDATED]
    pub access: u16,

    /// The list of modules that can access this opened package, 
    /// specified with fully qualified names (using dots)
    pub modules: Vec<QualifiedNameRef>,
}

#[derive(Clone, Debug)]
pub struct ModuleProvidesValue {
    /// The internal name of the service interface. (see [Type::get_internal_name]).
    pub service: InternalNameRef,

    /// The internal names of the implementations of the service interface.
    pub providers: Vec<InternalNameRef>,
}

pub type StrRef = Rc<str>;

/// eg: java/lang/Class
pub type InternalNameRef = StrRef; 

/// eg: java.lang.Class
pub type QualifiedNameRef = StrRef;

pub type DescriptorRef = StrRef;

impl ToRcRef<str> for &str {
    #[inline]
    fn as_rc(&self) -> StrRef {
        Rc::from(self.as_ref())
    }
}

impl ToRcRef<str> for String {
    #[inline]
    fn as_rc(&self) -> StrRef {
        Rc::from(self.as_ref())
    }
}


