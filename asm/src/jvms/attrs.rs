use crate::jvms::element::AttributeInfo;

#[derive(Clone, Debug)]
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
        
    },
    RuntimeInvisibleTypeAnnotations,
    AnnotationDefault,
    BootstrapMethods,
    MethodParameters,
    Module,
    ModulePackages,
    ModuleMainClass,
    NestHost,
    NestMembers,
    Record,
    PermittedSubclasses,
}

// {
//     u2 inner_class_info_index;
//     u2 outer_class_info_index;
//     u2 inner_name_index;
//     u2 inner_class_access_flags;
// }
#[derive(Clone, Copy, Debug)]
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
#[derive(Clone, Copy, Debug)]
pub struct LineNumberTableInfo {
    pub start_pc: u16,
    pub line_number: u16,
}

// {
//     u2 start_pc;
//     u2 length;
//     u2 name_index;
//     u2 descriptor_index;
//     u2 index;
// }
#[derive(Clone, Copy, Debug)]
pub struct LocalVariableTableInfo {
    start_pc: u16,
    length: u16,
    name_index: u16,
    descriptor_index: u16,
    // If the given local variable is of type double or long, it occupies both index and index + 1.
    index: u16,
}

// {
//     u2 start_pc;
//     u2 length;
//     u2 name_index;
//     u2 signature_index;
//     u2 index;
// }
#[derive(Clone, Copy, Debug)]
pub struct LocalVariableTypeTableInfo {
    start_pc: u16,
    length: u16,
    name_index: u16,
    signature_index: u16,
    // If the given local variable is of type double or long, it occupies both index and index + 1.
    index: u16,
}

// union verification_type_info {
//     Top_variable_info; // ITEM_Top
//     Integer_variable_info; // ITEM_Integer
//     Float_variable_info; // ITEM_Float
//     Long_variable_info; // ITEM_Long
//     Double_variable_info; // ITEM_Double
//     Null_variable_info; // ITEM_Null
//     UninitializedThis_variable_info; // ITEM_UninitializedThis
//     Object_variable_info; // ITEM_Object
//     Uninitialized_variable_info; // ITEM_Uninitialized
// }
#[derive(Clone, Copy, Debug)]
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
#[derive(Clone, Debug)]
pub enum StackMapFrame {
    SameFrame { frame_type: u8 },
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
    SameFrameExtended { frame_type: u8, offset_delta: u16 },
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
//     u2 catch_type;
// }
#[derive(Clone, Copy, Debug)]
pub struct ExceptionTable {
    pub start_pc: u16,
    pub end_pc: u16,
    pub handler_pc: u16,
    pub catch_type: u16,
}

// annotation {
//     u2 type_index;
//     u2 num_element_value_pairs;
//     {   u2            element_name_index;
//         element_value value;
//     } element_value_pairs[num_element_value_pairs];
// }
#[derive(Clone, Debug)]
pub struct AnnotationInfo {
    type_index: u16,
    num_element_value_pairs: u16,
    element_value_pairs: Vec<AnnotationElement>,
}

// {   
//     u2            element_name_index;
//     element_value value;
// } 
#[derive(Clone, Debug)]
pub struct AnnotationElement {
    element_name_index: u16,
    value: AnnotationElementValueInfo,
}

// element_value {
//     u1 tag;
//     union {
//         u2 const_value_index;
// 
//         {   u2 type_name_index;
//             u2 const_name_index;
//         } enum_const_value;
// 
//         u2 class_info_index;
// 
//         annotation annotation_value;
// 
//         {   u2            num_values;
//             element_value values[num_values];
//         } array_value;
//     } value;
// }
#[derive(Clone, Debug)]
pub struct AnnotationElementValueInfo {
    tag: u8,
    value: AnnotationElementValue,
}

// union {
//     u2 const_value_index;
// 
//     {   u2 type_name_index; // CONSTANT_Utf8_info 
//         u2 const_name_index; // CONSTANT_Utf8_info 
//     } enum_const_value;
// 
//     u2 class_info_index; // CONSTANT_Utf8_info 
// 
//     annotation annotation_value;
// 
//     {   u2            num_values;
//         element_value values[num_values];
//     } array_value;
// } value;
#[derive(Clone, Debug)]
pub enum AnnotationElementValue {
    Const { const_value_index: u16 },
    EnumConst { type_name_index: u16, const_name_index: u16 },
    Annotation { annotation_value: AnnotationInfo },
    Array { num_values: u16, values: Vec<AnnotationElementValueInfo> },
}

// {   
//     u2         num_annotations;
//     annotation annotations[num_annotations];
// }
#[derive(Clone, Debug)]
pub struct ParameterAnnotationInfo {
    num_annotations: u16,
    annotations: Vec<AnnotationInfo>,
}

