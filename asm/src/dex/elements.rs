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

#[derive(Copy, Clone, Debug)]
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

#[derive(Copy, Clone, Debug)]
pub struct StringId {
    pub string_data_off: DUInt,
}

pub struct StringData {
    pub utf16_size: DULeb128,
    pub data: Vec<u8>,
}

#[derive(Copy, Clone, Debug)]
pub struct TypeId {
    pub descriptor_idx: DUInt,
}

#[derive(Copy, Clone, Debug)]
pub struct ProtoId {
    pub shorty_idx: DUInt,
    pub return_type_idx: DUInt,
    pub parameters_off: DUInt,
}

#[derive(Copy, Clone, Debug)]
pub struct FieldId {
    pub class_idx: DUShort,
    pub type_idx: DUShort,
    pub name_idx: DUInt,
}

#[derive(Copy, Clone, Debug)]
pub struct MethodId {
    pub class_idx: DUShort,
    pub proto_idx: DUShort,
    pub name_idx: DUInt,
}

#[derive(Copy, Clone, Debug)]
pub struct ClassDef {
    pub class_idx: DUInt,
    pub access_flags: DUInt,
    pub superclass_idx: DUInt,
    pub interfaces_off: DUInt,
    pub source_file_idx: DUInt,
    pub annotations_off: DUInt,
    pub class_data_off: DUInt,
    pub static_values_off: DUInt,
}

#[derive(Copy, Clone, Debug)]
pub struct CallSiteId {
    pub call_site_off: DUInt,
}

#[derive(Copy, Clone, Debug)]
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
