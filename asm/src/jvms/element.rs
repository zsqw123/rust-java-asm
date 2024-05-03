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
use crate::jvms::attr::Attribute;

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
    //     u2 class_index; // CONSTANT_Class_info
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


