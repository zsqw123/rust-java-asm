use std::rc::Rc;

use crate::asm_type::Type;
use crate::jvms::attr::{LineNumberTableInfo, StackMapFrame};
use crate::jvms::attr::annotation::type_annotation::{TypeAnnotationTargetInfo, TypeAnnotationTargetPath};
use crate::node::insn::InsnNode;
use crate::node::values::{AnnotationValue, ConstValue, Descriptor, FieldInitialValue, InternalName, LocalVariableInfo, LocalVariableTypeInfo, ModuleAttrValue, ModuleExportValue, ModuleOpenValue, ModuleProvidesValue, ModuleRequireValue, QualifiedName};
use crate::opcodes::Opcodes;

#[derive(Clone, Debug)]
pub struct ClassNode {
    /// The class version.
    pub minor_version: u16,
    pub major_version: u16,

    /// The class's access flags (see [Opcodes]).
    pub access: u16,

    /// The internal name of this class (see [Type::get_internal_name]).
    pub name: Rc<InternalName>,

    /// The signature of this class. May be [None].
    pub signature: Option<Rc<String>>,

    /// The internal of name of the super class (see [Type::get_internal_name]).
    /// For interfaces, the super class is `Object`. May be [None], but only for the
    /// `Object` class.
    pub super_name: Option<Rc<InternalName>>,

    /// The internal names of the interfaces directly implemented by this class (see [Type::get_internal_name])
    pub interfaces: Vec<Rc<InternalName>>,

    /// The name of the source file from which this class was compiled. May be [None].
    pub source_file: Option<Rc<String>>,

    /// The correspondence between source and compiled elements of this class. May be [None].
    pub source_debug: Option<Rc<String>>,

    /// The module stored in this class. May be [None].
    pub module: Option<ModuleNode>,

    /// The internal name of the enclosing class of this class (see [Type::get_internal_name]).
    /// Must be [None] if this class is not a local or anonymous class.
    pub outer_class: Option<Rc<InternalName>>,

    /// The name of the method that contains the class, or [None] if the class has no
    /// enclosing class, or is not enclosed in a method or constructor of its enclosing class (e.g. if
    /// it is enclosed in an instance initializer, static initializer, instance variable initializer,
    /// or class variable initializer).
    pub outer_method: Option<Rc<String>>,

    /// The descriptor of the method that contains the class, or [None] if the class has no
    /// enclosing class, or is not enclosed in a method or constructor of its enclosing class (e.g. if
    /// it is enclosed in an instance initializer, static initializer, instance variable initializer,
    /// or class variable initializer).
    pub outer_method_desc: Option<Rc<Descriptor>>,

    pub annotations: Vec<AnnotationNode>,

    pub type_annotations: Vec<TypeAnnotationNode>,

    /// The non-standard attributes of this class.
    pub attrs: Vec<UnknownAttribute>,

    /// The inner classes of this class.
    pub inner_classes: Vec<InnerClassNode>,

    /// The internal name of the nest host class of this class (see [Type::get_internal_name]). 
    /// May be [None].
    pub nest_host_class: Option<Rc<InternalName>>,

    /// The internal names of the nest members of this class (see [Type::get_internal_name]). 
    pub nest_members: Vec<Rc<InternalName>>,

    /// The internal names of the permitted subclasses of this class (see [Type::get_internal_name]).
    pub permitted_subclasses: Vec<Rc<InternalName>>,

    /// The record components of this class.
    pub record_components: Vec<RecordComponentNode>,

    /// The fields of this class.
    pub fields: Vec<FieldNode>,

    /// The methods of this class.
    pub methods: Vec<MethodNode>,
}

#[derive(Clone, Debug)]
pub struct MethodNode {
    /// The method's access flags (see [Opcodes]).
    pub access: u16,

    /// The method's name.
    pub name: Rc<String>,

    /// The method's descriptor (see [Type::get_method_descriptor]).
    pub desc: Rc<Descriptor>,

    /// The method's signature. May be [None].
    pub signature: Option<Rc<String>>,

    /// The internal names of the method's exceptions (see [Type::get_internal_name]).
    pub exceptions: Vec<Rc<InternalName>>,

    /// The method parameter info.
    pub parameters: Vec<ParameterNode>,

    pub annotations: Vec<AnnotationNode>,

    pub type_annotations: Vec<TypeAnnotationNode>,

    /// The i'th entry in the parameter_annotations table may, but is not required to, 
    /// correspond to the i'th parameter descriptor in the method descriptor (§4.3.3).
    /// 
    /// For example, a compiler may choose to create entries in the table corresponding 
    /// only to those parameter descriptors which represent explicitly declared parameters 
    /// in source code. In the Java programming language, a constructor of an inner 
    /// class is specified to have an implicitly declared parameter before its explicitly 
    /// declared parameters (JLS §8.8.1), so the corresponding <init> method in a class 
    /// file has a parameter descriptor representing the implicitly declared parameter 
    /// before any parameter descriptors representing explicitly declared parameters. 
    /// If the first explicitly declared parameter is annotated in source code, then 
    /// a compiler may create parameter_annotation at index 0 to store annotations 
    /// corresponding to the second parameter descriptor.
    pub parameter_annotations: Vec<Vec<AnnotationNode>>,

    /// The non-standard attributes of this method.
    pub attrs: Vec<UnknownAttribute>,

    /// The default value of this annotation interface method
    pub annotation_default: Option<AnnotationValue>,

    pub instructions: Vec<InsnNode>,

    pub try_catch_blocks: Vec<TryCatchBlockNode>,

    pub local_variables: Vec<LocalVariableNode>,
}

#[derive(Clone, Debug)]
pub struct InnerClassNode {
    /// The internal name of an inner class (see [Type::get_internal_name]).
    pub name: Rc<InternalName>,

    /// The internal name of the class to which the inner class belongs (see [Type::get_internal_name]).
    pub outer_name: Option<Rc<InternalName>>,

    /// The simple name of the inner class inside its enclosing class.
    pub inner_name: Rc<String>,

    /// The access flags of the inner class as originally declared in the enclosing class.
    pub access: u16,
}

#[derive(Clone, Debug)]
pub struct RecordComponentNode {
    /// The record component's name.
    pub name: Rc<String>,

    /// The record component's descriptor (see [Type::get_descriptor]).
    pub desc: Rc<String>,

    /// The record component's signature. May be [None].
    pub signature: Option<Rc<String>>,

    pub annotations: Vec<AnnotationNode>,
    pub type_annotations: Vec<TypeAnnotationNode>,
}

#[derive(Clone, Debug)]
pub struct ParameterNode {
    /// The parameter's name. May be [None].
    pub name: Option<Rc<String>>,

    /// The parameter's access flags. Valid values are [Opcodes::ACC_FINAL], [Opcodes::ACC_SYNTHETIC]
    pub access: u16,
}

#[derive(Clone, Debug)]
pub struct FieldNode {
    /// The field's access flags (see [Opcodes]).
    pub access: u16,

    /// The field's name.
    pub name: Rc<String>,

    /// The field's descriptor (see [Type::get_descriptor]).
    pub desc: Rc<Descriptor>,

    /// The field's signature. May be [None].
    pub signature: Option<Rc<String>>,

    /// The field's initial value. This field, which may be [None] if the field does not have an initial value, 
    /// must be an Integer, a Float, a Long, a Double or a String.
    pub value: Option<FieldInitialValue>,

    pub annotations: Vec<AnnotationNode>,

    pub type_annotations: Vec<TypeAnnotationNode>,

    /// The non-standard attributes of this field.
    pub attrs: Vec<UnknownAttribute>,
}

#[derive(Clone, Debug)]
pub struct ModuleNode {
    /// The name of the module.
    pub name: Rc<String>,

    /// The access flags of the module, valid values are [Opcodes::ACC_OPEN], [Opcodes::ACC_SYNTHETIC], [Opcodes::ACC_MANDATED]
    pub access: u16,

    /// The version of the module. May be [None].
    pub version: Option<Rc<String>>,

    /// The main class of the module. May be [None].
    pub main_class: Option<Rc<String>>,

    /// The packages of the module.
    pub packages: Vec<Rc<String>>,
    
    /// The dependencies of this module.
    pub requires: Vec<ModuleRequireValue>,
    
    /// The packages exported by this module
    pub exports: Vec<ModuleExportValue>,
    
    /// The packages opened by this module.
    pub opens: Vec<ModuleOpenValue>,
    
    /// The internal names of the services used by this module (see [Type::get_internal_name]).
    pub uses: Vec<Rc<InternalName>>,
    
    // The services provided by this module.
    pub provides: Vec<ModuleProvidesValue>,
}

#[derive(Clone, Debug)]
pub struct TypeAnnotationNode {
    pub target_info: TypeAnnotationTargetInfo,
    pub target_path: TypeAnnotationTargetPath,
    pub annotation_node: AnnotationNode,
}

#[derive(Clone, Debug)]
pub struct AnnotationNode {
    pub visible: bool,
    pub type_name: Rc<InternalName>,
    // attribute -> value pairs
    pub values: Vec<(Rc<String>, AnnotationValue)>,
}
#[derive(Clone, Debug)]
pub struct TryCatchBlockNode {
    /// The beginning of the exception handler's scope (inclusive).
    pub start: LabelNode,
    /// The end of the exception handler's scope (exclusive).
    pub end: LabelNode,
    /// The beginning of the exception handler's code.
    pub handler: LabelNode,
    /// The internal name of the type of exceptions handled by the exception handler, 
    /// or [None] to catch any exceptions (for "finally" blocks).
    pub catch_type: Option<Rc<InternalName>>,
    /// type annotations on the exception handler type.
    pub type_annotations: Vec<TypeAnnotationNode>,
}

#[derive(Clone, Debug)]
pub struct LocalVariableNode {
    /// The name of a local variable.
    pub name: Rc<String>,
    /// The type descriptor of this local variable.
    pub desc: Rc<String>,
    /// The signature of this local variable. May be [None].
    pub signature: Option<Rc<String>>,
    /// The first instructions corresponding to the continuous ranges 
    /// that make the scope of this local variable (inclusive)
    pub start: LabelNode,
    /// The last instructions corresponding to the continuous ranges 
    /// that make the scope of this local variable (exclusive).
    pub end: LabelNode,
    /// The local variable's index.
    pub index: u32,
    /// type annotations on the local variable type.
    pub type_annotations: Vec<TypeAnnotationNode>,
}

#[derive(Clone, Debug)]
pub struct UnknownAttribute {
    pub name: Rc<String>,
    pub info: Vec<u8>,
    pub index: u16, // index of the attribute in attributes table
}

#[derive(Clone, Debug)]
pub enum Attribute {
    Custom(Vec<u8>),
    ConstantValue(ConstValue),
    Code {
        max_stack: u16,
        max_locals: u16,
        code: Vec<u8>,
        exception_table: Vec<ExceptionTable>,
        attributes: Vec<Rc<Attribute>>,
    },
    StackMapTable(Vec<StackMapFrame>),
    Exceptions(Vec<Rc<InternalName>>),
    InnerClasses(Vec<InnerClassNode>),
    EnclosingMethod {
        class: Rc<InternalName>,
        method_name: Rc<String>,
        method_desc: Rc<Descriptor>,
    },
    Synthetic,
    Signature(Rc<String>),
    SourceFile(Rc<String>),
    SourceDebugExtension(Vec<u8>),
    LineNumberTable(Vec<LineNumberNode>),
    LocalVariableTable(Vec<LocalVariableInfo>),
    LocalVariableTypeTable(Vec<LocalVariableTypeInfo>),
    Deprecated,
    // annotations
    RuntimeVisibleAnnotations(Vec<AnnotationNode>),
    RuntimeInvisibleAnnotations(Vec<AnnotationNode>),
    RuntimeVisibleParameterAnnotations(Vec<Vec<AnnotationNode>>),
    RuntimeInvisibleParameterAnnotations(Vec<Vec<AnnotationNode>>),
    RuntimeVisibleTypeAnnotations(Vec<TypeAnnotationNode>),
    RuntimeInvisibleTypeAnnotations(Vec<TypeAnnotationNode>),
    AnnotationDefault(AnnotationValue),
    // 
    BootstrapMethods(Vec<BootstrapMethodNode>),
    MethodParameters(Vec<ParameterNode>),
    Module(ModuleAttrValue),
    ModulePackages(Vec<Rc<String>>),
    ModuleMainClass(Rc<InternalName>),
    NestHost(Rc<InternalName>),
    NestMembers(Vec<Rc<InternalName>>),
    Record(RecordComponentNode),
    PermittedSubclasses(Vec<Rc<InternalName>>),
}

#[derive(Clone, Debug)]
pub struct ExceptionTable {
    pub start: LabelNode, // [start_pc, end_pc)
    pub end: LabelNode,
    pub handler: LabelNode,
    pub catch_type: Option<Rc<InternalName>>,
}

pub type LineNumberNode = LineNumberTableInfo;

#[derive(Clone, Debug)]
pub struct BootstrapMethodNode {
    pub method_handle: Rc<ConstValue>, // ConstValue::MethodHandle
    pub arguments: Vec<Rc<ConstValue>>,
}

// each label contains a unique id in the method scope.
pub type LabelNode = u16;
