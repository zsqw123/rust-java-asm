use java_asm_macro::const_container;

pub trait ConstContainer<C> {
    fn const_name(c: C) -> Option<&'static str>;
}

pub struct Constants;

pub struct JavaVersions;

pub struct JavaAccessFlags;
pub struct JavaClassAccessFlags;
pub struct JavaMethodAccessFlags;
pub struct JavaFieldAccessFlags;
pub struct JavaParameterAccessFlags;
pub struct JavaModuleAccessFlags;
pub struct JavaModuleRequireAccessFlags;

#[const_container(u32)]
impl JavaVersions {
    // Java ClassFile versions (the minor version is stored in the 16 most significant bits, and the
    // major version in the 16 least significant bits).
    pub const V1_1: u32 = 3 << 16 | 45;
    pub const V1_2: u32 = 0 << 16 | 46;
    pub const V1_3: u32 = 0 << 16 | 47;
    pub const V1_4: u32 = 0 << 16 | 48;
    pub const V1_5: u32 = 0 << 16 | 49;
    pub const V1_6: u32 = 0 << 16 | 50;
    pub const V1_7: u32 = 0 << 16 | 51;
    pub const V1_8: u32 = 0 << 16 | 52;
    pub const V9: u32 = 0 << 16 | 53;
    pub const V10: u32 = 0 << 16 | 54;
    pub const V11: u32 = 0 << 16 | 55;
    pub const V12: u32 = 0 << 16 | 56;
    pub const V13: u32 = 0 << 16 | 57;
    pub const V14: u32 = 0 << 16 | 58;
    pub const V15: u32 = 0 << 16 | 59;
    pub const V16: u32 = 0 << 16 | 60;
    pub const V17: u32 = 0 << 16 | 61;
    pub const V18: u32 = 0 << 16 | 62;
    pub const V19: u32 = 0 << 16 | 63;
    pub const V20: u32 = 0 << 16 | 64;
    pub const V21: u32 = 0 << 16 | 65;
    pub const V22: u32 = 0 << 16 | 66;
    pub const V23: u32 = 0 << 16 | 67;

    /**
     * Version flag indicating that the class is using 'preview' features.
     *
     * <p>{@code version & V_PREVIEW == V_PREVIEW} tests if a version is flagged with {@code
     * V_PREVIEW}.
    */
    pub const V_PREVIEW: u32 = 0xFFFF0000;
}

// Access flags values, defined in
// - https://docs.oracle.com/javase/specs/jvms/se9/html/jvms-4.html#jvms-4.1-200-E.1
// - https://docs.oracle.com/javase/specs/jvms/se9/html/jvms-4.html#jvms-4.5-200-A.1
// - https://docs.oracle.com/javase/specs/jvms/se9/html/jvms-4.html#jvms-4.6-200-A.1
// - https://docs.oracle.com/javase/specs/jvms/se9/html/jvms-4.html#jvms-4.7.25
#[const_container(u16)]
impl JavaAccessFlags {
    pub const ACC_PUBLIC: u16 = 0x0001;
    pub const ACC_PRIVATE: u16 = 0x0002;
    pub const ACC_PROTECTED: u16 = 0x0004;
    pub const ACC_STATIC: u16 = 0x0008;
    pub const ACC_FINAL: u16 = 0x0010;
    pub const ACC_ABSTRACT: u16 = 0x0400;
    pub const ACC_SYNTHETIC: u16 = 0x1000;
    pub const ACC_ENUM: u16 = 0x4000;
}

#[const_container(u16)]
impl JavaClassAccessFlags {
    pub const ACC_PUBLIC: u16 = JavaAccessFlags::ACC_PUBLIC;
    pub const ACC_PRIVATE: u16 = JavaAccessFlags::ACC_PRIVATE;
    pub const ACC_PROTECTED: u16 = JavaAccessFlags::ACC_PROTECTED;
    pub const ACC_FINAL: u16 = JavaAccessFlags::ACC_FINAL;
    pub const ACC_SUPER: u16 = 0x0020;
    pub const ACC_INTERFACE: u16 = 0x0200;
    pub const ACC_ABSTRACT: u16 = JavaAccessFlags::ACC_ABSTRACT;
    pub const ACC_SYNTHETIC: u16 = JavaAccessFlags::ACC_SYNTHETIC;
    pub const ACC_ANNOTATION: u16 = 0x2000;
    pub const ACC_ENUM: u16 = JavaAccessFlags::ACC_ENUM;
    pub const ACC_MODULE: u16 = 0x8000;
}

#[const_container(u16)]
impl JavaMethodAccessFlags {
    pub const ACC_PUBLIC: u16 = JavaAccessFlags::ACC_PUBLIC;
    pub const ACC_PRIVATE: u16 = JavaAccessFlags::ACC_PRIVATE;
    pub const ACC_PROTECTED: u16 = JavaAccessFlags::ACC_PROTECTED;
    pub const ACC_STATIC: u16 = JavaAccessFlags::ACC_STATIC;
    pub const ACC_FINAL: u16 = JavaAccessFlags::ACC_FINAL;
    pub const ACC_SYNCHRONIZED: u16 = 0x0020;
    pub const ACC_BRIDGE: u16 = 0x0040;
    pub const ACC_VARARGS: u16 = 0x0080;
    pub const ACC_NATIVE: u16 = 0x0100;
    pub const ACC_ABSTRACT: u16 = JavaAccessFlags::ACC_ABSTRACT;
    pub const ACC_STRICT: u16 = 0x0800;
    pub const ACC_SYNTHETIC: u16 = JavaAccessFlags::ACC_SYNTHETIC;
    pub const ACC_MANDATED: u16 = 0x8000;
}

#[const_container(u16)]
impl JavaFieldAccessFlags {
    pub const ACC_PUBLIC: u16 = JavaAccessFlags::ACC_PUBLIC;
    pub const ACC_PRIVATE: u16 = JavaAccessFlags::ACC_PRIVATE;
    pub const ACC_PROTECTED: u16 = JavaAccessFlags::ACC_PROTECTED;
    pub const ACC_STATIC: u16 = JavaAccessFlags::ACC_STATIC;
    pub const ACC_FINAL: u16 = JavaAccessFlags::ACC_FINAL;
    pub const ACC_VOLATILE: u16 = 0x0040;
    pub const ACC_TRANSIENT: u16 = 0x0080;
    pub const ACC_SYNTHETIC: u16 = JavaAccessFlags::ACC_SYNTHETIC;
    pub const ACC_ENUM: u16 = JavaAccessFlags::ACC_ENUM;
    pub const ACC_MANDATED: u16 = 0x8000;
}

#[const_container(u16)]
impl JavaParameterAccessFlags {
    pub const ACC_FINAL: u16 = JavaAccessFlags::ACC_FINAL;
    pub const ACC_SYNTHETIC: u16 = JavaAccessFlags::ACC_SYNTHETIC;
    pub const ACC_MANDATED: u16 = 0x8000;
}

#[const_container(u16)]
impl JavaModuleAccessFlags {
    pub const ACC_OPEN: u16 = 0x0020;
    pub const ACC_SYNTHETIC: u16 = JavaAccessFlags::ACC_SYNTHETIC;
    pub const ACC_MANDATED: u16 = 0x8000;
}

#[const_container(u16)]
impl JavaModuleRequireAccessFlags {
    pub const ACC_TRANSITIVE: u16 = 0x0020;
    pub const ACC_STATIC_PHASE: u16 = 0x0040;
    pub const ACC_SYNTHETIC: u16 = JavaAccessFlags::ACC_SYNTHETIC;
    pub const ACC_MANDATED: u16 = 0x8000;
}

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
