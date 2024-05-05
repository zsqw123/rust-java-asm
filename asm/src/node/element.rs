use crate::asm_type::Type;
use crate::jvms::attr::annotation::type_annotation::{TypeAnnotationTargetInfo, TypeAnnotationTargetPath};
use crate::node::values::AnnotationValue;
use crate::opcodes::Opcodes;

#[derive(Clone, Debug)]
pub struct ClassNode {
    /// The class version. The minor version is stored in the 16 most significant bits, and the major
    /// version in the 16 least significant bits.
    pub version: u32,

    /// The class's access flags (see [Opcodes]).
    pub access: u32,

    /// The internal name of this class (see [Type::get_internal_name]).
    pub name: String,

    /// The signature of this class. May be [None].
    pub signature: Option<String>,

    /// The internal of name of the super class (see [Type::get_internal_name]).
    /// For interfaces, the super class is `Object`. May be [None], but only for the
    /// `Object` class.
    pub super_name: Option<String>,

    /// The internal names of the interfaces directly implemented by this class (see [Type::get_internal_name])
    pub interfaces: Vec<String>,

    /// The name of the source file from which this class was compiled. May be [None].
    pub source_file: Option<String>,

    /// The correspondence between source and compiled elements of this class. May be [None].
    pub source_debug: Option<String>,

    /// The module stored in this class. May be [None].
    pub module: Option<ModuleNode>,

    /// The internal name of the enclosing class of this class (see [Type::get_internal_name]).
    /// Must be [None] if this class is not a local or anonymous class.
    pub outer_class: Option<String>,

    /// The name of the method that contains the class, or [None] if the class has no
    /// enclosing class, or is not enclosed in a method or constructor of its enclosing class (e.g. if
    /// it is enclosed in an instance initializer, static initializer, instance variable initializer,
    /// or class variable initializer).
    pub outer_method: Option<String>,

    /// The descriptor of the method that contains the class, or [None] if the class has no
    /// enclosing class, or is not enclosed in a method or constructor of its enclosing class (e.g. if
    /// it is enclosed in an instance initializer, static initializer, instance variable initializer,
    /// or class variable initializer).
    pub outer_method_desc: Option<String>,

    pub annotations: Vec<AnnotationNode>,

    pub type_annotations: Vec<TypeAnnotationNode>,

    /// The non-standard attributes of this class.
    pub attrs: Vec<Attribute>,

    /// The inner classes of this class.
    pub inner_classes: Vec<InnerClassNode>,

    /// The internal name of the nest host class of this class (see [Type::get_internal_name]). 
    /// May be [None].
    pub nest_host_class: Option<String>,

    /// The internal names of the nest members of this class (see [Type::get_internal_name]). 
    pub nest_members: Vec<String>,

    /// The internal names of the permitted subclasses of this class (see [Type::get_internal_name]).
    pub permitted_subclasses: Vec<String>,

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
    pub name: String,

    /// The method's descriptor (see [Type::get_method_descriptor]).
    pub desc: String,

    /// The method's signature. May be [None].
    pub signature: Option<String>,

    /// The internal names of the method's exceptions (see [Type::get_internal_name]).
    pub exceptions: Vec<String>,

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
    pub attrs: Vec<Attribute>,

    /// The default value of this annotation interface method
    pub annotation_default: Option<AnnotationValue>,

    pub instructions: Vec<InsnNode>,

    pub try_catch_blocks: Vec<TryCatchBlockNode>,

    pub local_variables: Vec<LocalVariableNode>,

    pub local_variable_annotations: Vec<Vec<LocalVariableAnnotationNode>>,
}

#[derive(Clone, Debug)]
pub struct InnerClassNode {}

#[derive(Clone, Debug)]
pub struct RecordComponentNode {}

#[derive(Clone, Debug)]
pub struct ParameterNode {
    /// The parameter's name. May be [None].
    pub name: Option<String>,

    /// The parameter's access flags. Valid values are [Opcodes::ACC_FINAL], [Opcodes::ACC_SYNTHETIC]
    pub access: u32,
}

#[derive(Clone, Debug)]
pub struct FieldNode {}

#[derive(Clone, Debug)]
pub struct ModuleNode {}

#[derive(Clone, Debug)]
pub struct TypeAnnotationNode {
    pub target_info: TypeAnnotationTargetInfo,
    pub target_path: TypeAnnotationTargetPath,
    pub annotation_node: AnnotationNode,
}

#[derive(Clone, Debug)]
pub struct AnnotationNode {
    pub visible: bool,
    pub desc: String,
    // attribute -> value pairs
    pub values: Vec<(String, AnnotationValue)>,
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
    pub catch_type: Option<String>,
    /// type annotations on the exception handler type.
    pub type_annotations: Vec<TypeAnnotationNode>,
}

#[derive(Clone, Debug)]
pub struct LocalVariableNode {}

#[derive(Clone, Debug)]
pub struct LocalVariableAnnotationNode {
    /// The fist instructions corresponding to the continuous ranges 
    /// that make the scope of this local variable (inclusive)
    pub start: LabelNode,
    /// The last instructions corresponding to the continuous ranges 
    /// that make the scope of this local variable (exclusive).
    pub end: LabelNode,
    /// The local variable's index.
    pub index: u32,
}


#[derive(Clone, Debug)]
pub enum InsnNode {
    
}

#[derive(Clone, Debug)]
pub struct Attribute {}

// each label contains a unique id in the method scope.
#[derive(Clone, Debug)]
pub struct LabelNode(pub u32);
