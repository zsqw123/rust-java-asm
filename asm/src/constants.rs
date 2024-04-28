pub struct Constants;

impl Constants {
    // The ClassFile attribute names, in the order they are defined in
    // https://docs.oracle.com/javase/specs/jvms/se11/html/jvms-4.html#jvms-4.7-300.

    const CONSTANT_VALUE: &'static str = "ConstantValue";
    const CODE: &'static str = "Code";
    const STACK_MAP_TABLE: &'static str = "StackMapTable";
    const EXCEPTIONS: &'static str = "Exceptions";
    const INNER_CLASSES: &'static str = "InnerClasses";
    const ENCLOSING_METHOD: &'static str = "EnclosingMethod";
    const SYNTHETIC: &'static str = "Synthetic";
    const SIGNATURE: &'static str = "Signature";
    const SOURCE_FILE: &'static str = "SourceFile";
    const SOURCE_DEBUG_EXTENSION: &'static str = "SourceDebugExtension";
    const LINE_NUMBER_TABLE: &'static str = "LineNumberTable";
    const LOCAL_VARIABLE_TABLE: &'static str = "LocalVariableTable";
    const LOCAL_VARIABLE_TYPE_TABLE: &'static str = "LocalVariableTypeTable";
    const DEPRECATED: &'static str = "Deprecated";
    const RUNTIME_VISIBLE_ANNOTATIONS: &'static str = "RuntimeVisibleAnnotations";
    const RUNTIME_INVISIBLE_ANNOTATIONS: &'static str = "RuntimeInvisibleAnnotations";
    const RUNTIME_VISIBLE_PARAMETER_ANNOTATIONS: &'static str = "RuntimeVisibleParameterAnnotations";
    const RUNTIME_INVISIBLE_PARAMETER_ANNOTATIONS: &'static str ="RuntimeInvisibleParameterAnnotations";
    const RUNTIME_VISIBLE_TYPE_ANNOTATIONS: &'static str = "RuntimeVisibleTypeAnnotations";
    const RUNTIME_INVISIBLE_TYPE_ANNOTATIONS: &'static str = "RuntimeInvisibleTypeAnnotations";
    const ANNOTATION_DEFAULT: &'static str = "AnnotationDefault";
    const BOOTSTRAP_METHODS: &'static str = "BootstrapMethods";
    const METHOD_PARAMETERS: &'static str = "MethodParameters";
    const MODULE: &'static str = "Module";
    const MODULE_PACKAGES: &'static str = "ModulePackages";
    const MODULE_MAIN_CLASS: &'static str = "ModuleMainClass";
    const NEST_HOST: &'static str = "NestHost";
    const NEST_MEMBERS: &'static str = "NestMembers";
    const PERMITTED_SUBCLASSES: &'static str = "PermittedSubclasses";
    const RECORD: &'static str = "Record";

    const REF_getField: u8 = 1;
    const REF_getStatic: u8 = 2;
    const REF_putField: u8 = 3;
    const REF_putStatic: u8 = 4;
    const REF_invokeVirtual: u8 = 5;
    const REF_invokeStatic: u8 = 6;
    const REF_invokeSpecial: u8 = 7;
    const REF_newInvokeSpecial: u8 = 8;
    const REF_invokeInterface: u8 = 9;
}
