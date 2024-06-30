pub struct Constants;

#[allow(non_upper_case_globals)]
impl Constants {
    // The ClassFile attribute names, in the order they are defined in
    // https://docs.oracle.com/javase/specs/jvms/se11/html/jvms-4.html#jvms-4.7-300.

    pub const CONSTANT_VALUE: &'static str = "ConstantValue";
    pub const CODE: &'static str = "Code";
    pub const STACK_MAP_TABLE: &'static str = "StackMapTable";
    pub const EXCEPTIONS: &'static str = "Exceptions";
    pub const INNER_CLASSES: &'static str = "InnerClasses";
    pub const ENCLOSING_METHOD: &'static str = "EnclosingMethod";
    pub const SYNTHETIC: &'static str = "Synthetic";
    pub const SIGNATURE: &'static str = "Signature";
    pub const SOURCE_FILE: &'static str = "SourceFile";
    pub const SOURCE_DEBUG_EXTENSION: &'static str = "SourceDebugExtension";
    pub const LINE_NUMBER_TABLE: &'static str = "LineNumberTable";
    pub const LOCAL_VARIABLE_TABLE: &'static str = "LocalVariableTable";
    pub const LOCAL_VARIABLE_TYPE_TABLE: &'static str = "LocalVariableTypeTable";
    pub const DEPRECATED: &'static str = "Deprecated";
    pub const RUNTIME_VISIBLE_ANNOTATIONS: &'static str = "RuntimeVisibleAnnotations";
    pub const RUNTIME_INVISIBLE_ANNOTATIONS: &'static str = "RuntimeInvisibleAnnotations";
    pub const RUNTIME_VISIBLE_PARAMETER_ANNOTATIONS: &'static str = "RuntimeVisibleParameterAnnotations";
    pub const RUNTIME_INVISIBLE_PARAMETER_ANNOTATIONS: &'static str = "RuntimeInvisibleParameterAnnotations";
    pub const RUNTIME_VISIBLE_TYPE_ANNOTATIONS: &'static str = "RuntimeVisibleTypeAnnotations";
    pub const RUNTIME_INVISIBLE_TYPE_ANNOTATIONS: &'static str = "RuntimeInvisibleTypeAnnotations";
    pub const ANNOTATION_DEFAULT: &'static str = "AnnotationDefault";
    pub const BOOTSTRAP_METHODS: &'static str = "BootstrapMethods";
    pub const METHOD_PARAMETERS: &'static str = "MethodParameters";
    pub const MODULE: &'static str = "Module";
    pub const MODULE_PACKAGES: &'static str = "ModulePackages";
    pub const MODULE_MAIN_CLASS: &'static str = "ModuleMainClass";
    pub const NEST_HOST: &'static str = "NestHost";
    pub const NEST_MEMBERS: &'static str = "NestMembers";
    pub const PERMITTED_SUBCLASSES: &'static str = "PermittedSubclasses";
    pub const RECORD: &'static str = "Record";

    // CP const
    pub const CONSTANT_Invalid: u8 = 0;
    pub const CONSTANT_Utf8: u8 = 1;
    pub const CONSTANT_Integer: u8 = 3;
    pub const CONSTANT_Float: u8 = 4;
    pub const CONSTANT_Long: u8 = 5;
    pub const CONSTANT_Double: u8 = 6;
    pub const CONSTANT_Class: u8 = 7;
    pub const CONSTANT_String: u8 = 8;
    pub const CONSTANT_Fieldref: u8 = 9;
    pub const CONSTANT_Methodref: u8 = 10;
    pub const CONSTANT_InterfaceMethodref: u8 = 11;
    pub const CONSTANT_NameAndType: u8 = 12;
    pub const CONSTANT_MethodHandle: u8 = 15;
    pub const CONSTANT_MethodType: u8 = 16;
    pub const CONSTANT_Dynamic: u8 = 17;
    pub const CONSTANT_InvokeDynamic: u8 = 18;
    pub const CONSTANT_Module: u8 = 19;
    pub const CONSTANT_Package: u8 = 20;
    
    pub const OBJECT_INTERNAL_NAME: &'static str = "java/lang/Object";


    pub const REF_getField: u8 = 1;
    pub const REF_getStatic: u8 = 2;
    pub const REF_putField: u8 = 3;
    pub const REF_putStatic: u8 = 4;
    pub const REF_invokeVirtual: u8 = 5;
    pub const REF_invokeStatic: u8 = 6;
    pub const REF_invokeSpecial: u8 = 7;
    pub const REF_newInvokeSpecial: u8 = 8;
    pub const REF_invokeInterface: u8 = 9;
}
