use java_asm_macro::ReadFrom;

#[derive(Clone, Debug)]
pub struct DexFile {
    pub header: Header,
    pub string_ids: Vec<StringId>,
    pub type_ids: Vec<TypeId>,
    pub proto_ids: Vec<ProtoId>,
    pub field_ids: Vec<FieldId>,
    pub method_ids: Vec<MethodId>,
    pub class_defs: Vec<ClassDef>,
    pub call_site_ids: Vec<CallSiteId>,
    pub method_handles: Vec<MethodHandle>,
    pub data: Vec<u8>,
    pub link_data: Vec<u8>,
}

#[derive(Copy, Clone, Debug, ReadFrom)]
#[align(4)]
pub struct Header {
    pub magic: [u8; 8],  // should be "dex\n039\0", and 039 is the dex version number
    pub checksum: DUInt, // adler32 checksum of the rest of the file (everything except magic and this field)
    pub signature: [u8; 20], // SHA-1 hash of the rest of the file (everything except magic, checksum, and this field)
    pub file_size: DUInt,
    pub header_size: DUInt, // length of this section
    pub endian_tag: DUInt,  // 0x12345678 for little-endian, 0x78563412 for big-endian
    pub link_size: DUInt,   // size of the link section, or 0 if this file isn't statically linked
    pub link_off: DUInt, // offset from the start of the file to the link section, or 0 if link_size == 0
    pub map_off: DUInt,
    pub string_ids_size: DUInt, // count of StringId items
    pub string_ids_off: DUInt,  // offset from the start of the file to the StringId items
    pub type_ids_size: DUInt,   // count of TypeId items, at most 65535
    pub type_ids_off: DUInt,
    pub proto_ids_size: DUInt, // count of ProtoId items, at most 65535
    pub proto_ids_off: DUInt,
    pub field_ids_size: DUInt, // count of FieldId items
    pub field_ids_off: DUInt,
    pub method_ids_size: DUInt,
    pub method_ids_off: DUInt,
    pub class_defs_size: DUInt,
    pub class_defs_off: DUInt,
    pub data_size: DUInt, // size of the data section, must be an even multiple of sizeof(uint)
    pub data_off: DUInt,
}

#[derive(Clone, Debug, ReadFrom)]
#[align(4)]
pub struct MapList {
    pub size: DUInt,
    #[index(size)]
    pub items: Vec<MapItem>,
}

#[derive(Copy, Clone, Debug, ReadFrom)]
pub struct MapItem {
    pub type_value: DUShort,
    pub unused: DUShort, // reserved
    pub size: DUInt, // count of items to be found at the specified offset
    pub offset: DUInt, // offset from the start of the file
}

#[derive(Copy, Clone, Debug, ReadFrom)]
#[align(4)]
pub struct StringId {
    pub string_data_off: DUInt, // StringData, offset from the start of the file
}

#[derive(Clone, Debug)]
pub struct StringData {
    /// The size of the string, in UTF-16 code units, this is the decoded length of the string,
    /// and the encoded length is implied by position of `\0` in the data. (because the MUTF-8
    /// format will not include a `\0` in the encoded data)
    pub utf16_size: DULeb128,
    /// A series of MUTF-8 bytes followed by a single '\0' byte.
    pub data: Vec<u8>,
}

#[derive(Copy, Clone, Debug, ReadFrom)]
#[align(4)]
pub struct TypeId {
    pub descriptor_idx: DUInt, // index into `string_ids` for the descriptor string
}

#[derive(Copy, Clone, Debug, ReadFrom)]
#[align(4)]
pub struct ProtoId {
    pub shorty_idx: DUInt,      // index into `string_ids` for shorty descriptor
    pub return_type_idx: DUInt, // index into `type_ids` for return type
    pub parameters_off: DUInt, // offset from the start of the file to the `type_list` for the parameters
}

#[derive(Copy, Clone, Debug, ReadFrom)]
#[align(4)]
pub struct FieldId {
    pub class_idx: DUShort, // index into `type_ids` for the definer
    pub type_idx: DUShort,  // index into `type_ids` for the type
    pub name_idx: DUInt,    // index into `string_ids` for the name
}

#[derive(Copy, Clone, Debug, ReadFrom)]
#[align(4)]
pub struct MethodId {
    pub class_idx: DUShort, // index into `type_ids` for the definer
    pub proto_idx: DUShort, // index into `proto_ids` for the prototype
    pub name_idx: DUInt,    // index into `string_ids` for the name
}

#[derive(Copy, Clone, Debug, ReadFrom)]
#[align(4)]
pub struct ClassDef {
    /// index into `type_ids`
    pub class_idx: DUInt,
    pub access_flags: DUInt,
    /// index into `type_ids`, or NO_INDEX if this class has no superclass
    pub superclass_idx: DUInt,
    /// offset from the start of the file to the list of interfaces, or 0 if there are no interfaces
    /// for this class. Must be an offset of a `type_list` structure.
    pub interfaces_off: DUInt,
    pub source_file_idx: DUInt, // index into `string_ids` for the source file name, or NO_INDEX
    /// offset from the start of the file to the `annotations_directory_item` or 0 if not present.
    pub annotations_off: DUInt,
    /// offset from the start of the file to the `class_data_item` or 0 if not present.
    pub class_data_off: DUInt,
    /// offset from the start of the file to the list of `encoded_array_item`, or 0 if not present.
    /// Same order of the static fields in the `field_list`.
    pub static_values_off: DUInt,
}

#[derive(Clone, Debug, ReadFrom)]
#[align(4)]
pub struct TypeList {
    pub size: DUInt,
    #[index(size)]
    pub type_id_indices: Vec<DUInt>, // index into `type_ids`
}

#[derive(Copy, Clone, Debug, ReadFrom)]
#[align(4)]
pub struct CallSiteId {
    /// offset from the start of the file to the `call_site_item`
    pub call_site_off: DUInt,
}

/// The call site item is an encoded array of the following form:
/// 1. A method handle representing the bootstrap linker method (VALUE_METHOD_HANDLE).
/// 2. A method name that the bootstrap linker should resolve (VALUE_STRING).
/// 3. A method type corresponding to the type of the method name to be resolved (VALUE_METHOD_TYPE).
/// ... Any additional arguments are constant values passed to the bootstrap linker method.
pub type CallSiteItem = EncodedArray;

#[derive(Clone, Debug)]
pub enum EncodedValue {
    Byte(DByte),
    Short(DShort),
    Char(DUShort),
    Int(DInt),
    Long(DLong),
    Float([DUByte; 4]),                 // IEEE754 32-bit
    Double([DUByte; 8]),                // IEEE754 64-bit
    MethodType(DUInt),                  // index into `proto_ids`
    MethodHandle(DUInt),                // index into `method_handles`
    String(DUInt),                      // index into `string_ids`
    Type(DUInt),                        // index into `type_ids`
    Field(DUInt),                       // index into `field_ids`
    Method(DUInt),                      // index into `method_ids`
    Enum(DUInt),                        // index into `field_ids`
    Array(Vec<Self>),                   // `encoded_array`
    Annotation(Vec<EncodedAnnotation>), // `encoded_annotation`
    Null,
    Boolean(bool),
}

#[derive(Clone, Debug)]
pub struct EncodedArray {
    pub size: DULeb128,
    pub values: Vec<EncodedValue>,
}

#[derive(Clone, Debug)]
pub struct EncodedAnnotation {
    pub type_idx: DULeb128, // index into `type_ids`
    pub size: DULeb128,     // size of name-value mappings
    pub elements: Vec<EncodedAnnotationAttribute>,
}

#[derive(Clone, Debug)]
pub struct EncodedAnnotationAttribute {
    pub name_idx: DULeb128, // index into `string_ids`
    pub value: EncodedValue,
}

#[derive(Copy, Clone, Debug, ReadFrom)]
#[align(4)]
pub struct MethodHandle {
    pub method_handle_type: DUShort,
    pub unused_stub_0: DUShort, // android reserved, don't know why.
    pub field_or_method_id: DUShort,
    pub unused_stub_1: DUShort, // android reserved, don't know why.
}

// dex types
pub type DByte = i8;
pub type DUByte = u8;
pub type DShort = i16;
pub type DUShort = u16;
pub type DInt = i32;
pub type DUInt = u32;
pub type DLong = i64;
pub type DULong = u64;

#[derive(Copy, Clone, Debug)]
pub struct DSleb128(pub(crate) u32);
#[derive(Copy, Clone, Debug)]
pub struct DULeb128(pub(crate) u32);
#[derive(Copy, Clone, Debug)]
pub struct DULeb128P1(pub(crate) u32);

pub const NO_INDEX: u32 = 0xFFFFFFFF;

impl DSleb128 {
    #[inline]
    pub const fn value(&self) -> i32 {
        self.0 as i32
    }
}

impl DULeb128 {
    #[inline]
    pub const fn value(&self) -> u32 {
        self.0
    }
}

impl Into<usize> for DULeb128 {
    fn into(self) -> usize {
        self.value() as usize
    }
}

impl DULeb128P1 {
    // -1 usually used for representing null (NO_INDEX in dex format)
    #[inline]
    pub const fn value(&self) -> Option<u32> {
        let internal = self.0;
        if internal == 0 {
            None
        } else {
            Some(internal - 1)
        }
    }
}
