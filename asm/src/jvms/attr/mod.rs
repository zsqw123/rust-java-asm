use java_asm_internal::read::jvms::FromReadContext;
use java_asm_internal::write::jvms::IntoWriteContext;

use crate::jvms::attr::annotation::{AnnotationElementValueInfo, AnnotationInfo, ParameterAnnotationInfo};
use crate::jvms::attr::annotation::type_annotation::TypeAnnotation;
use crate::jvms::attr::module::{ModuleExports, ModuleOpens, ModuleProvides, ModuleRequires};
use crate::jvms::element::AttributeInfo;
use crate::node::element::LabelNode;

pub mod annotation;
pub mod module;

#[derive(Clone, Debug, IntoWriteContext)]
pub enum Attribute {
    Custom(Vec<u8>),
    // ConstantValue_attribute {
    //     u2 attribute_name_index; // CONSTANT_Utf8_info
    //     u4 attribute_length;
    //     u2 constantvalue_index; // index of const pool
    // }
    ConstantValue { constantvalue_index: u16 },
    // Code_attribute {
    //     u2 attribute_name_index;
    //     u4 attribute_length;
    //     u2 max_stack;
    //     u2 max_locals;
    //     u4 code_length;
    //     u1 code[code_length];
    //     u2 exception_table_length;
    //     {   u2 start_pc;
    //         u2 end_pc;
    //         u2 handler_pc;
    //         u2 catch_type;
    //     } exception_table[exception_table_length];
    //     u2 attributes_count;
    //     attribute_info attributes[attributes_count];
    // }
    Code {
        max_stack: u16,
        max_locals: u16,
        code_length: u32,
        code: Vec<u8>,
        exception_table_length: u16,
        exception_table: Vec<ExceptionTable>,
        attributes_count: u16,
        attributes: Vec<AttributeInfo>,
    },
    // StackMapTable_attribute {
    //     u2              attribute_name_index;
    //     u4              attribute_length;
    //     u2              number_of_entries;
    //     stack_map_frame entries[number_of_entries];
    // }
    StackMapTable {
        number_of_entries: u16,
        entries: Vec<StackMapFrame>,
    },
    // Exceptions_attribute {
    //     u2 attribute_name_index;
    //     u4 attribute_length;
    //     u2 number_of_exceptions;
    //     u2 exception_index_table[number_of_exceptions]; // CONSTANT_Class_info
    // }
    Exceptions {
        number_of_exceptions: u16,
        exception_index_table: Vec<u16>,
    },
    // InnerClasses_attribute {
    //     u2 attribute_name_index;
    //     u4 attribute_length;
    //     u2 number_of_classes;
    //     {   u2 inner_class_info_index;
    //         u2 outer_class_info_index;
    //         u2 inner_name_index;
    //         u2 inner_class_access_flags;
    //     } classes[number_of_classes];
    // }
    InnerClasses {
        number_of_classes: u16,
        classes: Vec<InnerClassInfo>,
    },
    // EnclosingMethod_attribute {
    //     u2 attribute_name_index;
    //     u4 attribute_length;
    //     u2 class_index; // CONSTANT_Class_info
    //     u2 method_index; // CONSTANT_NameAndType_info or zero
    // }
    EnclosingMethod {
        class_index: u16,
        method_index: u16,
    },
    // Synthetic_attribute {
    //     u2 attribute_name_index;
    //     u4 attribute_length;
    // }
    Synthetic,
    // Signature_attribute {
    //     u2 attribute_name_index;
    //     u4 attribute_length;
    //     u2 signature_index; // CONSTANT_Utf8_info
    // }
    Signature {
        signature_index: u16,
    },
    // SourceFile_attribute {
    //     u2 attribute_name_index;
    //     u4 attribute_length;
    //     u2 sourcefile_index; // CONSTANT_Utf8_info
    // }
    SourceFile {
        sourcefile_index: u16,
    },
    // SourceDebugExtension_attribute {
    //     u2 attribute_name_index;
    //     u4 attribute_length;
    //     u1 debug_extension[attribute_length];
    // }
    SourceDebugExtension {
        debug_extension: Vec<u8>,
    },
    // LineNumberTable_attribute {
    //     u2 attribute_name_index;
    //     u4 attribute_length;
    //     u2 line_number_table_length;
    //     {   u2 start_pc;
    //         u2 line_number;
    //     } line_number_table[line_number_table_length];
    // }
    LineNumberTable {
        line_number_table_length: u16,
        line_number_table: Vec<LineNumberTableInfo>,
    },
    // LocalVariableTable_attribute {
    //     u2 attribute_name_index;
    //     u4 attribute_length;
    //     u2 local_variable_table_length;
    //     {   u2 start_pc;
    //         u2 length;
    //         u2 name_index;
    //         u2 descriptor_index;
    //         u2 index;
    //     } local_variable_table[local_variable_table_length];
    // }
    LocalVariableTable {
        local_variable_table_length: u16,
        local_variable_table: Vec<LocalVariableTableInfo>,
    },
    // LocalVariableTypeTable_attribute {
    //     u2 attribute_name_index;
    //     u4 attribute_length;
    //     u2 local_variable_type_table_length;
    //     {   u2 start_pc;
    //         u2 length;
    //         u2 name_index;
    //         u2 signature_index;
    //         u2 index;
    //     } local_variable_type_table[local_variable_type_table_length];
    // }
    LocalVariableTypeTable {
        local_variable_type_table_length: u16,
        local_variable_table: Vec<LocalVariableTypeTableInfo>,
    },
    // Deprecated_attribute {
    //     u2 attribute_name_index;
    //     u4 attribute_length;
    // }
    Deprecated,
    // RuntimeVisibleAnnotations_attribute {
    //     u2         attribute_name_index;
    //     u4         attribute_length;
    //     u2         num_annotations;
    //     annotation annotations[num_annotations];
    // }
    RuntimeVisibleAnnotations {
        num_annotations: u16,
        annotations: Vec<AnnotationInfo>,
    },
    RuntimeInvisibleAnnotations {
        num_annotations: u16,
        annotations: Vec<AnnotationInfo>,
    },
    RuntimeVisibleParameterAnnotations {
        num_parameters: u8,
        parameter_annotations: Vec<ParameterAnnotationInfo>,
    },
    RuntimeInvisibleParameterAnnotations {
        num_parameters: u8,
        parameter_annotations: Vec<ParameterAnnotationInfo>,
    },
    RuntimeVisibleTypeAnnotations {
        num_parameters: u16,
        annotations: Vec<TypeAnnotation>,
    },
    RuntimeInvisibleTypeAnnotations {
        num_parameters: u16,
        annotations: Vec<TypeAnnotation>,
    },
    AnnotationDefault {
        default_value: AnnotationElementValueInfo,
    },
    // BootstrapMethods_attribute {
    //     u2 attribute_name_index;
    //     u4 attribute_length;
    //     u2 num_bootstrap_methods;
    //     {   u2 bootstrap_method_ref;
    //         u2 num_bootstrap_arguments;
    //         u2 bootstrap_arguments[num_bootstrap_arguments];
    //     } bootstrap_methods[num_bootstrap_methods];
    // }
    BootstrapMethods {
        num_bootstrap_methods: u16,
        bootstrap_methods: Vec<BootstrapMethod>,
    },
    // MethodParameters_attribute {
    //     u2 attribute_name_index;
    //     u4 attribute_length;
    //     u1 parameters_count;
    //     {   u2 name_index;
    //         u2 access_flags;
    //     } parameters[parameters_count];
    // }
    MethodParameters {
        parameters_count: u8,
        parameters: Vec<MethodParameter>,
    },
    // Module_attribute {
    //     u2 attribute_name_index;
    //     u4 attribute_length;
    //
    //     u2 module_name_index; // CONSTANT_Module_info
    //     u2 module_flags;
    //     u2 module_version_index; // CONSTANT_Utf8_info
    //
    //     u2 requires_count;
    //     {   u2 requires_index; // CONSTANT_Module_info
    //         u2 requires_flags;
    //         u2 requires_version_index; // CONSTANT_Utf8_info
    //     } requires[requires_count];
    //
    //     u2 exports_count;
    //     {   u2 exports_index; // CONSTANT_Package_info
    //         u2 exports_flags;
    //         u2 exports_to_count;
    //         u2 exports_to_index[exports_to_count]; // CONSTANT_Module_info
    //     } exports[exports_count];
    //
    //     u2 opens_count;
    //     {   u2 opens_index; // CONSTANT_Package_info
    //         u2 opens_flags;
    //         u2 opens_to_count;
    //         u2 opens_to_index[opens_to_count]; // CONSTANT_Module_info
    //     } opens[opens_count];
    //
    //     u2 uses_count;
    //     u2 uses_index[uses_count]; // CONSTANT_Class_info
    //
    //     u2 provides_count;
    //     {   u2 provides_index; // CONSTANT_Class_info
    //         u2 provides_with_count;
    //         u2 provides_with_index[provides_with_count]; // CONSTANT_Class_info
    //     } provides[provides_count];
    // }
    Module {
        module_name_index: u16,
        module_flags: u16,
        module_version_index: u16,
        requires_count: u16,
        requires: Vec<ModuleRequires>,
        exports_count: u16,
        exports: Vec<ModuleExports>,
        opens_count: u16,
        opens: Vec<ModuleOpens>,
        uses_count: u16,
        uses_index: Vec<u16>,
        provides_count: u16,
        provides: Vec<ModuleProvides>,
    },
    // ModulePackages_attribute {
    //     u2 attribute_name_index;
    //     u4 attribute_length;
    //     u2 package_count;
    //     u2 package_index[package_count]; // CONSTANT_Package_info
    // }
    ModulePackages {
        package_count: u16,
        package_index: Vec<u16>,
    },
    // ModuleMainClass_attribute {
    //     u2 attribute_name_index;
    //     u4 attribute_length;
    //     u2 main_class_index; // CONSTANT_Class_info
    // }
    ModuleMainClass {
        main_class_index: u16,
    },
    // NestHost_attribute {
    //     u2 attribute_name_index;
    //     u4 attribute_length;
    //     u2 host_class_index; // CONSTANT_Class_info
    // }
    NestHost {
        host_class_index: u16,
    },
    // NestMembers_attribute {
    //     u2 attribute_name_index;
    //     u4 attribute_length;
    //     u2 number_of_classes;
    //     u2 classes[number_of_classes]; // CONSTANT_Class_info
    // }
    NestMembers {
        number_of_classes: u16,
        classes: Vec<u16>,
    },
    // Record_attribute {
    //     u2                    attribute_name_index;
    //     u4                    attribute_length;
    //     u2                    components_count;
    //     record_component_info components[components_count];
    // }
    Record {
        components_count: u16,
        components: Vec<RecordComponentInfo>,
    },
    // PermittedSubclasses_attribute {
    //     u2 attribute_name_index;
    //     u4 attribute_length;
    //     u2 number_of_classes;
    //     u2 classes[number_of_classes]; // CONSTANT_Class_info
    // }
    PermittedSubclasses {
        number_of_classes: u16,
        classes: Vec<u16>,
    },
}

// {
//     u2 inner_class_info_index; // CONSTANT_Class_info
//     u2 outer_class_info_index; // CONSTANT_Class_info
//     u2 inner_name_index; // CONSTANT_Utf8_info
//     u2 inner_class_access_flags;
// }
#[derive(Clone, Copy, Debug, FromReadContext, IntoWriteContext)]
pub struct InnerClassInfo {
    pub inner_class_info_index: u16,
    pub outer_class_info_index: u16,
    pub inner_name_index: u16,
    pub inner_class_access_flags: u16,
}

// {
//     u2 start_pc;
//     u2 line_number;
// }
#[derive(Clone, Copy, Debug, FromReadContext, IntoWriteContext)]
pub struct LineNumberTableInfo {
    pub start_pc: LabelNode,
    pub line_number: u16,
}

// {
//     u2 start_pc;
//     u2 length;
//     u2 name_index; // CONSTANT_Utf8_info
//     u2 descriptor_index; // CONSTANT_Utf8_info
//     u2 index;
// }
#[derive(Clone, Copy, Debug, FromReadContext, IntoWriteContext)]
pub struct LocalVariableTableInfo {
    pub start_pc: LabelNode,
    pub length: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    // If the given local variable is of type double or long, it occupies both index and index + 1.
    pub index: u16,
}

// {
//     u2 start_pc;
//     u2 length;
//     u2 name_index; // CONSTANT_Utf8_info
//     u2 signature_index; // CONSTANT_Utf8_info
//     u2 index;
// }
#[derive(Clone, Copy, Debug, FromReadContext, IntoWriteContext)]
pub struct LocalVariableTypeTableInfo {
    pub start_pc: LabelNode,
    pub length: u16,
    pub name_index: u16,
    pub signature_index: u16,
    // If the given local variable is of type double or long, it occupies both index and index + 1.
    pub index: u16,
}

// union verification_type_info {
//     Top_variable_info; // ITEM_Top
//     Integer_variable_info; // ITEM_Integer
//     Float_variable_info; // ITEM_Float
//     Long_variable_info; // ITEM_Long
//     Double_variable_info; // ITEM_Double
//     Null_variable_info; // ITEM_Null
//     UninitializedThis_variable_info; // ITEM_UninitializedThis
//     Object_variable_info; // ITEM_Object CONSTANT_Class_info
//     Uninitialized_variable_info; // ITEM_Uninitialized
// }
#[derive(Clone, Copy, Debug, IntoWriteContext)]
pub enum VerificationTypeInfo {
    Top { tag: u8 },
    Integer { tag: u8 },
    Float { tag: u8 },
    Null { tag: u8 },
    UninitializedThis { tag: u8 },
    Object { tag: u8, cpool_index: u16 },
    Uninitialized { tag: u8, offset: u16 },
    Long { tag: u8 },
    Double { tag: u8 },
}

// union stack_map_frame {
//     same_frame; // SAME; /* 0-63 */
//     same_locals_1_stack_item_frame; // SAME_LOCALS_1_STACK_ITEM; /* 64-127 */
//     same_locals_1_stack_item_frame_extended; // SAME_LOCALS_1_STACK_ITEM_EXTENDED; /* 247 */
//     chop_frame; // CHOP; /* 248-250 */
//     same_frame_extended; // SAME_FRAME_EXTENDED; /* 251 */
//     append_frame; // APPEND; /* 252-254 */
//     full_frame; // FULL_FRAME; /* 255 */
// }
#[derive(Clone, Debug, IntoWriteContext)]
pub enum StackMapFrame {
    SameFrame { frame_type: u8 },
    SameFrameExtended { frame_type: u8, offset_delta: u16 },
    SameLocals1StackItemFrame {
        frame_type: u8,
        verification_type_info: VerificationTypeInfo,
    },
    SameLocals1StackItemFrameExtended {
        frame_type: u8,
        offset_delta: u16,
        verification_type_info: VerificationTypeInfo,
    },
    ChopFrame { frame_type: u8, offset_delta: u16 },
    AppendFrame {
        frame_type: u8,
        offset_delta: u16,
        locals: Vec<VerificationTypeInfo>,
    },
    FullFrame {
        frame_type: u8,
        offset_delta: u16,
        number_of_locals: u16,
        locals: Vec<VerificationTypeInfo>,
        number_of_stack_items: u16,
        stack: Vec<VerificationTypeInfo>,
    },
}

// ExceptionTable {
//     u2 start_pc;
//     u2 end_pc;
//     u2 handler_pc;
//     u2 catch_type; // CONSTANT_Class_info
// }
#[derive(Clone, Copy, Debug, FromReadContext, IntoWriteContext)]
pub struct ExceptionTable {
    pub start_pc: LabelNode,
    pub end_pc: LabelNode,
    pub handler_pc: LabelNode,
    pub catch_type: u16,
}

// {
//     u2 bootstrap_method_ref; // CONSTANT_MethodHandle_info
//     u2 num_bootstrap_arguments;
//     u2 bootstrap_arguments[num_bootstrap_arguments];  // valid index in const_pool
// }
#[derive(Clone, Debug, FromReadContext, IntoWriteContext)]
pub struct BootstrapMethod {
    pub bootstrap_method_ref: u16,
    pub num_bootstrap_arguments: u16,
    #[index(num_bootstrap_arguments)]
    pub bootstrap_arguments: Vec<u16>,
}

// {
//     u2 name_index;
//     u2 access_flags;
// }
#[derive(Clone, Copy, Debug, FromReadContext, IntoWriteContext)]
pub struct MethodParameter {
    pub name_index: u16,
    pub access_flags: u16,
}

// record_component_info {
//     u2             name_index;
//     u2             descriptor_index;
//     u2             attributes_count;
//     attribute_info attributes[attributes_count];
// }
#[derive(Clone, Debug, FromReadContext, IntoWriteContext)]
pub struct RecordComponentInfo {
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes_count: u16,
    #[index(attributes_count)]
    pub attributes: Vec<AttributeInfo>,
}
