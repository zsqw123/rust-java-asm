use crate::dex::raw::{DUByte, DUInt, DUShort};
use java_asm_macro::const_container;

pub struct MapListTypeConst;

#[const_container(DUShort)]
impl MapListTypeConst {
    pub const TYPE_HEADER_ITEM: DUShort = 0x0000;
    pub const TYPE_STRING_ID_ITEM: DUShort = 0x0001;
    pub const TYPE_TYPE_ID_ITEM: DUShort = 0x0002;
    pub const TYPE_PROTO_ID_ITEM: DUShort = 0x0003;
    pub const TYPE_FIELD_ID_ITEM: DUShort = 0x0004;
    pub const TYPE_METHOD_ID_ITEM: DUShort = 0x0005;
    pub const TYPE_CLASS_DEF_ITEM: DUShort = 0x0006;
    pub const TYPE_CALL_SITE_ID_ITEM: DUShort = 0x0007;
    pub const TYPE_METHOD_HANDLE_ITEM: DUShort = 0x0008;
    pub const TYPE_MAP_LIST: DUShort = 0x1000;
    pub const TYPE_TYPE_LIST: DUShort = 0x1001;
    pub const TYPE_ANNOTATION_SET_REF_LIST: DUShort = 0x1002;
    pub const TYPE_ANNOTATION_SET_ITEM: DUShort = 0x1003;
    pub const TYPE_CLASS_DATA_ITEM: DUShort = 0x2000;
    pub const TYPE_CODE_ITEM: DUShort = 0x2001;
    pub const TYPE_STRING_DATA_ITEM: DUShort = 0x2002;
    pub const TYPE_DEBUG_INFO_ITEM: DUShort = 0x2003;
    pub const TYPE_ANNOTATION_ITEM: DUShort = 0x2004;
    pub const TYPE_ENCODED_ARRAY_ITEM: DUShort = 0x2005;
    pub const TYPE_ANNOTATIONS_DIRECTORY_ITEM: DUShort = 0x2006;
    pub const TYPE_HIDDENAPI_CLASS_DATA_ITEM: DUShort = 0xF000;
}

pub struct AccessFlags;
pub struct ClassAccessFlags; // class specific access flags
pub struct MethodAccessFlags; // method specific access flags
pub struct FieldAccessFlags; // field specific access flags

#[const_container(DUInt)]
impl AccessFlags {
    pub const ACC_PUBLIC: DUInt = 0x0001;
    pub const ACC_PRIVATE: DUInt = 0x0002;
    pub const ACC_PROTECTED: DUInt = 0x0004;
    pub const ACC_STATIC: DUInt = 0x0008;
    pub const ACC_FINAL: DUInt = 0x0010;
    pub const ACC_ABSTRACT: DUInt = 0x0400;
    pub const UNUSED: DUInt = 0x8000;
    pub const ACC_SYNTHETIC: DUInt = 0x1000;
    pub const ACC_ENUM: DUInt = 0x4000;
}

#[const_container(DUInt)]
impl ClassAccessFlags {
    pub const ACC_PUBLIC: DUInt = AccessFlags::ACC_PUBLIC;
    pub const ACC_PRIVATE: DUInt = AccessFlags::ACC_PRIVATE;
    pub const ACC_PROTECTED: DUInt = AccessFlags::ACC_PROTECTED;
    pub const ACC_STATIC: DUInt = AccessFlags::ACC_STATIC;
    pub const ACC_FINAL: DUInt = AccessFlags::ACC_FINAL;
    pub const ACC_INTERFACE: DUInt = 0x0200;
    pub const ACC_ABSTRACT: DUInt = AccessFlags::ACC_ABSTRACT;
    pub const ACC_SYNTHETIC: DUInt = AccessFlags::ACC_SYNTHETIC;
    pub const ACC_ANNOTATION: DUInt = 0x2000;
    pub const ACC_ENUM: DUInt = AccessFlags::ACC_ENUM;
}

#[const_container(DUInt)]
impl MethodAccessFlags {
    pub const ACC_PUBLIC: DUInt = AccessFlags::ACC_PUBLIC;
    pub const ACC_PRIVATE: DUInt = AccessFlags::ACC_PRIVATE;
    pub const ACC_PROTECTED: DUInt = AccessFlags::ACC_PROTECTED;
    pub const ACC_STATIC: DUInt = AccessFlags::ACC_STATIC;
    pub const ACC_FINAL: DUInt = AccessFlags::ACC_FINAL;
    pub const ACC_SYNCHRONIZED: DUInt = 0x0020;
    pub const ACC_BRIDGE: DUInt = 0x0040;
    pub const ACC_VARARGS: DUInt = 0x0080;
    pub const ACC_NATIVE: DUInt = 0x0100;
    pub const ACC_ABSTRACT: DUInt = AccessFlags::ACC_ABSTRACT;
    pub const ACC_STRICT: DUInt = 0x0800;
    pub const ACC_SYNTHETIC: DUInt = AccessFlags::ACC_SYNTHETIC;
    pub const ACC_CONSTRUCTOR: DUInt = 0x10000;
    pub const ACC_DECLARED_SYNCHRONIZED: DUInt = 0x20000;
}

#[const_container(DUInt)]
impl FieldAccessFlags {
    pub const ACC_PUBLIC: DUInt = AccessFlags::ACC_PUBLIC;
    pub const ACC_PRIVATE: DUInt = AccessFlags::ACC_PRIVATE;
    pub const ACC_PROTECTED: DUInt = AccessFlags::ACC_PROTECTED;
    pub const ACC_STATIC: DUInt = AccessFlags::ACC_STATIC;
    pub const ACC_FINAL: DUInt = AccessFlags::ACC_FINAL;
    pub const ACC_VOLATILE: DUInt = 0x0040;
    pub const ACC_TRANSIENT: DUInt = 0x0080;
    pub const ACC_SYNTHETIC: DUInt = AccessFlags::ACC_SYNTHETIC;
    pub const ACC_ENUM: DUInt = 0x4000;
}

struct EncodedValueType;
#[const_container(DUByte)]
impl EncodedValueType {
    pub const VALUE_BYTE: DUByte = 0x00;
    pub const VALUE_SHORT: DUByte = 0x02;
    pub const VALUE_CHAR: DUByte = 0x03;
    pub const VALUE_INT: DUByte = 0x04;
    pub const VALUE_LONG: DUByte = 0x06;
    pub const VALUE_FLOAT: DUByte = 0x10;
    pub const VALUE_DOUBLE: DUByte = 0x11;
    pub const VALUE_METHOD_TYPE: DUByte = 0x15;
    pub const VALUE_METHOD_HANDLE: DUByte = 0x16;
    pub const VALUE_STRING: DUByte = 0x17;
    pub const VALUE_TYPE: DUByte = 0x18;
    pub const VALUE_FIELD: DUByte = 0x19;
    pub const VALUE_METHOD: DUByte = 0x1a;
    pub const VALUE_ENUM: DUByte = 0x1b;
    pub const VALUE_ARRAY: DUByte = 0x1c;
    pub const VALUE_ANNOTATION: DUByte = 0x1d;
    pub const VALUE_NULL: DUByte = 0x1e;
    pub const VALUE_BOOLEAN: DUByte = 0x1f;
}

struct MethodHandleType;

#[const_container(DUByte)]
impl MethodHandleType {
    pub const METHOD_HANDLE_TYPE_STATIC_PUT: DUByte = 0x00;
    pub const METHOD_HANDLE_TYPE_STATIC_GET: DUByte = 0x01;
    pub const METHOD_HANDLE_TYPE_INSTANCE_PUT: DUByte = 0x02;
    pub const METHOD_HANDLE_TYPE_INSTANCE_GET: DUByte = 0x03;
    pub const METHOD_HANDLE_TYPE_INVOKE_STATIC: DUByte = 0x04;
    pub const METHOD_HANDLE_TYPE_INVOKE_INSTANCE: DUByte = 0x05;
    pub const METHOD_HANDLE_TYPE_INVOKE_CONSTRUCTOR: DUByte = 0x06;
    pub const METHOD_HANDLE_TYPE_INVOKE_DIRECT: DUByte = 0x07;
    pub const METHOD_HANDLE_TYPE_INVOKE_INTERFACE: DUByte = 0x08;
}
