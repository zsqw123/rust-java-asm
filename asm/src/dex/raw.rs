use crate::dex::insn::DexInsn;
use crate::impls::jvms::r::U32BasedSize;
use java_asm_macro::ReadFrom;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DexFile {
    pub header: Header,
    pub string_ids: Vec<StringId>,
    pub type_ids: Vec<TypeId>,
    pub proto_ids: Vec<ProtoId>,
    pub field_ids: Vec<FieldId>,
    pub method_ids: Vec<MethodId>,
    pub class_defs: Vec<ClassDef>,
    // we don't need to save the actual data chunk, every data item is indexed by
    // using the offset from the start of the file.
    // 
    // pub call_site_ids: Vec<CallSiteId>,
    // pub method_handles: Vec<MethodHandle>,
    // pub data: Vec<u8>,
    // pub link_data: Vec<u8>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ReadFrom)]
#[align(4)]
pub struct Header {
    /// should be "dex\n039\0", and 039 is the dex version number
    pub magic: [u8; 8],
    /// adler32 checksum of the rest of the file (everything except magic and this field)
    pub checksum: DUInt,
    /// SHA-1 hash of the rest of the file (everything except magic, checksum, and this field)
    pub signature: [u8; 20], 
    /// size of the entire file (including the header), in bytes
    pub file_size: DUInt,
    /// length of this section
    pub header_size: DUInt,
    /// [Header::LITTLE_ENDIAN_TAG] or [Header::BIG_ENDIAN_TAG]
    pub endian_tag: DUInt,
    /// size of the link section, or 0 if this file isn't statically linked
    pub link_size: DUInt,
    /// offset from the start of the file to the link section, or 0 if link_size == 0
    pub link_off: DUInt, 
    /// offset from the start of the file to the data chunk with `map_item` format
    pub map_off: DUInt,
    /// count of StringId items
    pub string_ids_size: U32BasedSize,
    /// offset from the start of the file to the StringId items
    pub string_ids_off: DUInt,
    /// count of TypeId items, at most 65535
    pub type_ids_size: U32BasedSize,   
    pub type_ids_off: DUInt,
    /// count of ProtoId items, at most 65535
    pub proto_ids_size: U32BasedSize, 
    pub proto_ids_off: DUInt,
    /// count of FieldId items
    pub field_ids_size: U32BasedSize,
    pub field_ids_off: DUInt,
    pub method_ids_size: U32BasedSize,
    pub method_ids_off: DUInt,
    pub class_defs_size: U32BasedSize,
    pub class_defs_off: DUInt,
    /// size of the data section, must be an even multiple of sizeof(uint)
    pub data_size: U32BasedSize, 
    pub data_off: DUInt,
}

impl Header {
    pub const LITTLE_ENDIAN_TAG: u32 = 0x12345678;
    pub const BIG_ENDIAN_TAG: u32 = 0x78563412;
}

#[derive(Clone, Debug, Eq, PartialEq, ReadFrom)]
#[align(4)]
pub struct MapList {
    pub size: U32BasedSize,
    #[index(size)]
    pub items: Vec<MapItem>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ReadFrom)]
pub struct MapItem {
    /// defined in [crate::dex::constant::MapListTypeConst]
    pub type_value: DUShort,
    pub unused: DUShort, // reserved
    /// count of items to be found at the specified offset
    pub size: DUInt,
    pub offset: DUInt, // offset from the start of the file
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ReadFrom)]
#[align(4)]
pub struct StringId {
    pub string_data_off: DUInt, // StringData, offset from the start of the file
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StringData {
    /// The size of the string, in UTF-16 code units, this is the decoded length of the string,
    /// and the encoded length is implied by position of `\0` in the data. (because the MUTF-8
    /// format will not include a `\0` in the encoded data)
    pub utf16_size: DULeb128,
    /// A series of MUTF-8 bytes followed by a single '\0' byte.
    pub data: Vec<u8>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ReadFrom)]
#[align(4)]
pub struct TypeId {
    pub descriptor_idx: DUInt, // index into `string_ids` for the descriptor string
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ReadFrom)]
#[align(4)]
pub struct ProtoId {
    pub shorty_idx: DUInt,      // index into `string_ids` for shorty descriptor
    pub return_type_idx: DUInt, // index into `type_ids` for return type
    pub parameters_off: DUInt, // offset from the start of the file to the `type_list` for the parameters
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ReadFrom)]
#[align(4)]
pub struct FieldId {
    pub class_idx: DUShort, // index into `type_ids` for the definer
    pub type_idx: DUShort,  // index into `type_ids` for the type
    pub name_idx: DUInt,    // index into `string_ids` for the name
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ReadFrom)]
#[align(4)]
pub struct MethodId {
    pub class_idx: DUShort, // index into `type_ids` for the definer
    pub proto_idx: DUShort, // index into `proto_ids` for the prototype
    pub name_idx: DUInt,    // index into `string_ids` for the name
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ReadFrom)]
#[align(4)]
pub struct ClassDef {
    /// index into `type_ids`
    pub class_idx: DUInt,
    pub access_flags: DUInt,
    /// index into `type_ids`, or NO_INDEX if this class has no superclass
    pub superclass_idx: DUInt,
    /// offset from the start of the file to the list of interfaces, or 0 if there are no interfaces
    /// for this class. Must be an offset of a [TypeList] structure.
    pub interfaces_off: DUInt,
    pub source_file_idx: DUInt, // index into `string_ids` for the source file name, or NO_INDEX
    /// offset from the start of the file to the `annotations_directory_item` or 0 if not present.
    pub annotations_off: DUInt,
    /// offset from the start of the file to the [ClassDataItem] or 0 if not present.
    pub class_data_off: DUInt,
    /// offset from the start of the file to the list of [EncodedArray], or 0 if not present.
    /// Same order of the static fields in the `field_list`.
    pub static_values_off: DUInt,
}

#[derive(Clone, Debug, Eq, PartialEq, ReadFrom)]
pub struct ClassDataItem {
    pub static_fields_size: DULeb128,
    pub instance_fields_size: DULeb128,
    pub direct_methods_size: DULeb128,
    pub virtual_methods_size: DULeb128,
    #[index(static_fields_size)]
    pub static_fields: Vec<EncodedField>,
    #[index(instance_fields_size)]
    pub instance_fields: Vec<EncodedField>,
    #[index(direct_methods_size)]
    pub direct_methods: Vec<EncodedMethod>,
    #[index(virtual_methods_size)]
    pub virtual_methods: Vec<EncodedMethod>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ReadFrom)]
pub struct EncodedField {
    /// index into `field_ids`[FieldId], diff with previous encoded field
    pub field_idx_diff: DULeb128,
    /// see [crate::dex::FieldAccessFlags]
    pub access_flags: DULeb128,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ReadFrom)]
pub struct EncodedMethod {
    /// index into `method_ids`[MethodId], diff with previous encoded method
    pub method_idx_diff: DULeb128,
    /// see [crate::dex::MethodAccessFlags]
    pub access_flags: DULeb128,
    /// offset from the start of the file to the [CodeItem],
    /// or 0 if this method is abstract or native
    pub code_off: DULeb128,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodeItem {
    pub registers_size: DUShort,
    pub ins_size: DUShort,
    pub outs_size: DUShort,
    pub tries_size: DUShort,
    pub debug_info_off: DUInt,
    pub insn_container: InsnContainer,
    pub tries: Vec<TryItem>,
    pub handlers: EncodedCatchHandlerList,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InsnContainer {
    pub insns_size: DUInt,
    pub insns: Vec<DexInsn>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ReadFrom)]
pub struct TryItem {
    pub start_addr: DUInt,
    /// The last code unit covered (inclusive) is `start_addr + insn_count - 1`.
    pub insn_count: DUShort,
    /// offset in bytes from the start of the associated `encoded_catch_hander_list` 
    /// to the `encoded_catch_handler`
    pub handler_off: DUShort,
}

#[derive(Clone, Debug, Eq, PartialEq, ReadFrom, Default)]
pub struct EncodedCatchHandlerList {
    pub size: DULeb128,
    #[index(size)]
    pub list: Vec<EncodedCatchHandler>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncodedCatchHandler {
    pub size: DSleb128,
    pub handlers: Vec<EncodedTypeAddrPair>,
    pub catch_all_addr: Option<DULeb128>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ReadFrom)]
pub struct EncodedTypeAddrPair {
    pub type_idx: DULeb128,
    pub addr: DULeb128,
}

#[derive(Clone, Debug, Eq, PartialEq, ReadFrom)]
#[align(4)]
pub struct TypeList {
    pub size: U32BasedSize,
    #[index(size)]
    pub type_id_indices: Vec<DUShort>, // index into `type_ids`
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ReadFrom)]
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

#[derive(Clone, Debug, Eq, PartialEq)]
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncodedArray {
    pub size: DULeb128,
    pub values: Vec<EncodedValue>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncodedAnnotation {
    pub type_idx: DULeb128, // index into `type_ids`
    pub size: DULeb128,     // size of name-value mappings
    pub elements: Vec<EncodedAnnotationAttribute>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncodedAnnotationAttribute {
    pub name_idx: DULeb128, // index into `string_ids`
    pub value: EncodedValue,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ReadFrom)]
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

#[derive(Copy, Clone, Debug, Eq, PartialEq, Default)]
pub struct DSleb128(pub(crate) u32);
#[derive(Copy, Clone, Debug, Eq, PartialEq, Default)]
pub struct DULeb128(pub(crate) u32);
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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
