use crate::asm_type::Type;
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

    /// The runtime visible annotations of this class.
    pub visible_annotations: Vec<AnnotationNode>,

    /// The runtime invisible annotations of this class.
    pub invisible_annotations: Vec<AnnotationNode>,

    /// The runtime visible type annotations of this class.
    pub visible_type_annotations: Vec<TypeAnnotationNode>,

    /// The runtime invisible type annotations of this class.
    pub invisible_type_annotations: Vec<TypeAnnotationNode>,

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
pub struct ModuleNode {
    
}

#[derive(Clone, Debug)]
pub struct TypeAnnotationNode {}

#[derive(Clone, Debug)]
pub struct AnnotationNode {}

#[derive(Clone, Debug)]
pub struct InnerClassNode {}

#[derive(Clone, Debug)]
pub struct MethodNode {}

#[derive(Clone, Debug)]
pub struct FieldNode {}

#[derive(Clone, Debug)]
pub struct RecordComponentNode {}

#[derive(Clone, Debug)]
pub enum InsnNode {
    
}

#[derive(Clone, Debug)]
pub struct Attribute {}


