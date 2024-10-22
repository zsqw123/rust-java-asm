use crate::dex::{CallSiteId, CallSiteItem, DUInt, DUShort, FieldId, Header, MapList, MethodId, ProtoId, StringData, TypeList};
use crate::dex::{DexFileAccessor, MethodHandle};
use crate::err::{AsmResultExt, AsmResultOkExt};
use crate::impls::dex::r::element::read_type_list;
use crate::impls::jvms::r::{ReadContext, ReadFrom, U32BasedSize};
use crate::impls::VecEx;
use crate::{AsmErr, AsmResult, DescriptorRef, StrRef};

impl DexFileAccessor {
    #[inline]
    pub fn get_data_impl<T: ReadFrom>(
        &self, data_off: DUInt,
    ) -> AsmResult<T> {
        Self::get_data_in_bytes(&self.bytes, data_off, self.endian)
    }

    #[inline]
    pub fn get_map_list(
        bytes: &[u8], header: &Header, endian: bool,
    ) -> AsmResult<MapList> {
        Self::get_data_in_bytes(bytes, header.map_off, endian)
    }

    #[inline]
    pub fn get_call_site_ids(
        bytes: &[u8], call_site_offset: DUInt, size: U32BasedSize, endian: bool,
    ) -> Vec<CallSiteId> {
        Self::get_vec_in_bytes(bytes, call_site_offset, size, endian)
            .unwrap_or_default()
    }

    #[inline]
    pub fn get_method_handles(
        bytes: &[u8], method_handle_offset: DUInt, size: U32BasedSize, endian: bool,
    ) -> Vec<MethodHandle> {
        Self::get_vec_in_bytes(bytes, method_handle_offset, size, endian)
            .unwrap_or_default()
    }

    #[inline]
    pub fn get_data_in_bytes<T: ReadFrom>(
        bytes: &[u8], data_off: DUInt, endian: bool,
    ) -> AsmResult<T> {
        let mut read_context = if endian {
            ReadContext::big_endian(bytes)
        } else {
            ReadContext::little_endian(bytes)
        };
        read_context.index = data_off as usize;
        read_context.read()
    }

    #[inline]
    pub fn get_vec_in_bytes<T: ReadFrom>(
        bytes: &[u8], data_off: DUInt, size: impl Into<usize>, endian: bool,
    ) -> AsmResult<Vec<T>> {
        let size = Into::into(size);
        if size == 0 {
            return Ok(Vec::new());
        }
        let mut read_context = if endian {
            ReadContext::big_endian(bytes)
        } else {
            ReadContext::little_endian(bytes)
        };
        read_context.index = data_off as usize;
        read_context.read_vec(size)
    }

    #[inline]
    pub fn get_str(&self, str_idx: DUInt) -> AsmResult<StrRef> {
        let dex_file = &self.file;
        let string_data_off = dex_file.string_ids.get(str_idx as usize)
            .ok_or_error(|| AsmErr::OutOfRange(str_idx as usize).e())?.string_data_off;
        Ok(self.get_data_impl::<StringData>(string_data_off)?.str_ref)
    }

    #[inline]
    pub fn get_type(&self, type_idx: DUShort) -> AsmResult<DescriptorRef> {
        let dex_file = &self.file;
        let type_id = dex_file.type_ids.get(type_idx as usize)
            .ok_or_error(|| AsmErr::OutOfRange(type_idx as usize).e())?;
        self.get_str(type_id.descriptor_idx)
    }

    #[inline]
    pub fn get_proto(&self, proto_idx: DUShort) -> AsmResult<ProtoConst> {
        let dex_file = &self.file;
        let proto_id = dex_file.proto_ids.get(proto_idx as usize)
            .ok_or_error(|| AsmErr::OutOfRange(proto_idx as usize).e())?;
        let ProtoId { shorty_idx, return_type_idx, parameters_off } = *proto_id;
        let shorty = self.get_str(shorty_idx)?;
        let return_type = self.get_type(return_type_idx as DUShort)?;
        let parameters = self.get_type_list(parameters_off)?;
        ProtoConst { shorty, return_type, parameters }.ok()
    }

    #[inline]
    pub fn get_field(&self, field_idx: DUShort) -> AsmResult<FieldConst> {
        let dex_file = &self.file;
        let field_id = dex_file.field_ids.get(field_idx as usize)
            .ok_or_error(|| AsmErr::OutOfRange(field_idx as usize).e())?;
        let FieldId { class_idx, type_idx, name_idx, .. } = *field_id;
        let class_type = self.get_type(class_idx)?;
        let field_type = self.get_type(type_idx)?;
        let field_name = self.get_str(name_idx)?;
        FieldConst { class_type, field_type, field_name }.ok()
    }

    #[inline]
    pub fn get_method(&self, method_idx: DUShort) -> AsmResult<MethodConst> {
        let dex_file = &self.file;
        let method_id = dex_file.method_ids.get(method_idx as usize)
            .ok_or_error(|| AsmErr::OutOfRange(method_idx as usize).e())?;
        let MethodId { class_idx, proto_idx, name_idx } = *method_id;
        let class_type = self.get_type(class_idx)?;
        let proto_const = self.get_proto(proto_idx)?;
        let method_name = self.get_str(name_idx)?;
        MethodConst { class_type, proto_const, method_name }.ok()
    }
    #[inline]
    pub fn get_call_site(&self, call_site_index: DUShort) -> AsmResult<CallSiteItem> {
        let call_site_off = self.call_site_ids.get(call_site_index as usize)
            .ok_or_error(|| AsmErr::OutOfRange(call_site_index as usize).e())?;
        self.get_data_impl(call_site_off.call_site_off)
    }

    #[inline]
    pub fn get_type_list(&self, type_list_off: DUInt) -> AsmResult<Vec<DescriptorRef>> {
        if type_list_off == 0 { return Ok(vec![]); }
        let TypeList { type_id_indices, .. } = self.get_data_impl::<TypeList>(type_list_off)?;
        type_id_indices.map_res(|type_idx| self.get_type(*type_idx))
    }
}

pub struct ProtoConst {
    pub shorty: DescriptorRef,
    pub return_type: DescriptorRef,
    pub parameters: Vec<DescriptorRef>,
}

pub struct FieldConst {
    pub class_type: DescriptorRef,
    pub field_type: DescriptorRef,
    pub field_name: StrRef,
}

pub struct MethodConst {
    pub class_type: DescriptorRef,
    pub proto_const: ProtoConst,
    pub method_name: StrRef,
}
