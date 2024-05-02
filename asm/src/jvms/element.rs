// ClassFile {
//     u4             magic;
//     u2             minor_version;
//     u2             major_version;
//     u2             constant_pool_count;
//     cp_info        constant_pool[constant_pool_count-1];
//     u2             access_flags;
//     u2             this_class;
//     u2             super_class;
//     u2             interfaces_count;
//     u2             interfaces[interfaces_count];
//     u2             fields_count;
//     field_info     fields[fields_count];
//     u2             methods_count;
//     method_info    methods[methods_count];
//     u2             attributes_count;
//     attribute_info attributes[attributes_count];
// }

/// [JVMS4](https://docs.oracle.com/javase/specs/jvms/se9/html/jvms-4.html)
#[derive(Clone, Debug)]
pub struct ClassFile {
    pub magic: u32,
    pub minor_version: u16,
    pub major_version: u16,
    pub constant_pool_count: u16,
    pub constant_pool: Vec<CPInfo>,
    pub access_flags: u16,
    pub this_class: u16,
    pub super_class: u16,
    pub interfaces_count: u16,
    pub interfaces: Vec<u16>,
    pub fields_count: u16,
    pub fields: Vec<FieldInfo>,
    pub methods_count: u16,
    pub methods: Vec<MethodInfo>,
    pub attributes_count: u16,
    pub attributes: Vec<AttributeInfo>,
}

// cp_info {
//     u1 tag;
//     u1 info[];
// }
#[derive(Clone, Debug)]
pub struct CPInfo {
    pub tag: u8,
    pub info: Const,
}

#[derive(Clone, Debug)]
pub enum Const {
    // invalid const's tag is 0
    Invalid,
    // CONSTANT_Class_info {
    //     u1 tag;
    //     u2 name_index; // index of a CONSTANT_Utf8_info structure
    // }
    Class { name_index: u16 },
    // CONSTANT_Fieldref_info { (similar with Method/Interface)
    //     u1 tag;
    //     u2 class_index; // CONSTANT_Utf8_info
    //     u2 name_and_type_index; // CONSTANT_NameAndType_info
    // }
    Field { class_index: u16, name_and_type_index: u16 },
    Method { class_index: u16, name_and_type_index: u16 },
    InterfaceMethod { class_index: u16, name_and_type_index: u16 },
    // CONSTANT_String_info {
    //     u1 tag;
    //     u2 string_index; // CONSTANT_Utf8_info 
    // }
    String { string_index: u16 },
    // CONSTANT_Integer_info { (similar with Float)
    //     u1 tag;
    //     u4 bytes;
    // }
    // - Stored in big-endian (high byte first) order.
    Integer { bytes: u32 },
    Float { bytes: u32 }, // IEEE 754 floating-point single format
    // CONSTANT_Long_info { (similar with Double)
    //     u1 tag;
    //     u4 high_bytes;
    //     u4 low_bytes;
    // }
    // - All 8-byte constants take up two entries in the constant_pool table of the class file. 
    //   If a CONSTANT_Long_info or CONSTANT_Double_info structure is the item in the constant_pool table at index n, 
    //   then the next usable item in the pool is located at index n+2. 
    //   The constant_pool index n+1 must be valid but is considered unusable.
    // - Stored in big-endian (high byte first) order.
    Long { high_bytes: u32, low_bytes: u32 },
    Double { high_bytes: u32, low_bytes: u32 }, // IEEE 754 floating-point double format
    // CONSTANT_NameAndType_info {
    //     u1 tag;
    //     u2 name_index; // CONSTANT_Utf8_info 
    //     u2 descriptor_index; // CONSTANT_Utf8_info 
    // }
    NameAndType { name_index: u16, descriptor_index: u16 },
    // CONSTANT_Utf8_info {
    //     u1 tag;
    //     u2 length;
    //     u1 bytes[length];
    // }
    Utf8 { length: u16, bytes: Vec<u8> },
    // CONSTANT_MethodHandle_info {
    //     u1 tag;
    //     u1 reference_kind;
    //     u2 reference_index; // Field/Method/Interface
    // }
    MethodHandle { reference_kind: u8, reference_index: u16 },
    // CONSTANT_MethodType_info {
    //     u1 tag;
    //     u2 descriptor_index; // CONSTANT_Utf8_info 
    // }
    MethodType { descriptor_index: u16 },
    // CONSTANT_Dynamic_info { (similar with CONSTANT_InvokeDynamic_info)
    //     u1 tag;
    //     u2 bootstrap_method_attr_index; // BootstrapMethods_attribute 
    //     u2 name_and_type_index; // CONSTANT_NameAndType_info 
    // }
    Dynamic { bootstrap_method_attr_index: u16, name_and_type_index: u16 },
    InvokeDynamic { bootstrap_method_attr_index: u16, name_and_type_index: u16 },
    // CONSTANT_Module_info {
    //     u1 tag;
    //     u2 name_index; // CONSTANT_Utf8_info
    // }
    Module { name_index: u16 },
    // CONSTANT_Package_info {
    //     u1 tag;
    //     u2 name_index;
    // }
    Package { name_index: u16 },
}

// field_info {
//     u2             access_flags;
//     u2             name_index; // CONSTANT_Utf8_info
//     u2             descriptor_index; // CONSTANT_Utf8_info
//     u2             attributes_count;
//     attribute_info attributes[attributes_count];
// }
#[derive(Clone, Debug)]
pub struct FieldInfo {
    pub access_flags: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes_count: u16,
    pub attributes: Vec<AttributeInfo>,
}

// method_info {
//     u2             access_flags;
//     u2             name_index; // CONSTANT_Utf8_info
//     u2             descriptor_index; // CONSTANT_Utf8_info
//     u2             attributes_count;
//     attribute_info attributes[attributes_count];
// }
#[derive(Clone, Debug)]
pub struct MethodInfo {
    pub access_flags: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes_count: u16,
    pub attributes: Vec<AttributeInfo>,
}

// attribute_info {
//     u2 attribute_name_index; // CONSTANT_Utf8_info 
//     u4 attribute_length;
//     u1 info[attribute_length];
// }
#[derive(Clone, Debug)]
pub struct AttributeInfo {
    pub attribute_name_index: u16,
    pub attribute_length: u32,
    pub info: Attribute,
}

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
