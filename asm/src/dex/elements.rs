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

pub struct Header;

pub struct StringId {
    pub string_data_off: DUInt,
}

pub struct TypeId {
    pub descriptor_idx: DUInt,
}

pub struct ProtoId {
    pub shorty_idx: DUInt,
    pub return_type_idx: DUInt,
    pub parameters_off: DUInt,
}

pub struct FieldId {
    pub class_idx: DUShort,
    pub type_idx: DUShort,
    pub name_idx: DUInt,
}

pub struct MethodId {
    pub class_idx: DUShort,
    pub proto_idx: DUShort,
    pub name_idx: DUInt,
}

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

pub struct CallSiteId {
    pub call_site_off: DUInt,
}

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

pub struct DSleb128(pub(crate) u32);

pub struct DULeb128(pub(crate) u32);

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

impl DULeb128P1 {
    // -1 usually used for representing null
    #[inline]
    pub const fn value(&self) -> Option<u32> {
        let internal = self.0;
        if internal == 0 { None } else { Some(internal - 1) }
    }
}
