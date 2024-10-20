use crate::dex::element::{AsElement, ClassContentElement, FieldElement, MethodElement};
use crate::dex::{ClassDataItem, DUInt, DUShort, DexFileAccessor, EncodedField, EncodedMethod, FieldId, MethodId, ProtoId, StringData, TypeList};
use crate::err::{AsmResultExt, AsmResultOkExt};
use crate::impls::VecEx;
use crate::{AsmErr, AsmResult, StrRef};

// C: origin element which located in vec
// E: returned element
fn to_element_list_impl<C, E>(
    accessor: &DexFileAccessor, vec: &Vec<C>,
) -> AsmResult<Vec<E>>
where
    C: AsElement<E>,
{
    let vec_size = vec.len();
    let mut result = Vec::with_capacity(vec_size);
    let mut index = 0;
    while index < vec_size {
        let previous = if index != 0 {
            Some(&vec[index - 1])
        } else {
            None // cannot using `index - 1` because it may overflow when index is 0u32
        };
        // SAFETY: index is always less than vec_size
        let c = vec.get(index).unwrap();
        let element = c.to_element(accessor, previous)?;
        result.push(element);
        index += 1;
    }
    Ok(result)
}

fn read_field(
    accessor: &DexFileAccessor, encoded_field: &EncodedField, field_idx: u32,
) -> AsmResult<FieldElement> {
    let dex_file = &accessor.file;
    let field_id = dex_file.field_ids.get(field_idx as usize)
        .ok_or_error(|| AsmErr::OutOfRange(field_idx as usize).e())?;
    let FieldId { type_idx, name_idx, .. } = *field_id;
    let descriptor = read_type(accessor, type_idx)?;
    let name = read_string(accessor, name_idx)?;
    FieldElement {
        access_flags: encoded_field.access_flags.value(),
        name,
        descriptor,
    }.ok()
}

fn read_method(
    accessor: &DexFileAccessor, encoded_method: &EncodedMethod, method_idx: u32,
) -> AsmResult<MethodElement> {
    let dex_file = &accessor.file;
    let method_id = dex_file.method_ids.get(method_idx as usize)
        .ok_or_error(|| AsmErr::OutOfRange(method_idx as usize).e())?;
    let MethodId { proto_idx, name_idx, .. } = *method_id;
    let access_flags = encoded_method.access_flags.value();
    let ProtoElement { shorty: shorty_descriptor, return_type, parameters } = read_proto(accessor, proto_idx)?;
    let name = read_string(accessor, name_idx)?;
    let code_off = encoded_method.code_off.value();
    MethodElement { access_flags, name, shorty_descriptor, return_type, parameters, code_off }.ok()
}

struct ProtoElement {
    shorty: StrRef,
    return_type: StrRef,
    parameters: Vec<StrRef>,
}

fn read_proto(
    accessor: &DexFileAccessor, proto_idx: DUShort,
) -> AsmResult<ProtoElement> {
    let dex_file = &accessor.file;
    let proto_id = dex_file.proto_ids.get(proto_idx as usize)
        .ok_or_error(|| AsmErr::OutOfRange(proto_idx as usize).e())?;
    let ProtoId { shorty_idx, return_type_idx, parameters_off } = *proto_id;
    let shorty = read_string(accessor, shorty_idx)?;
    let return_type = read_type(accessor, return_type_idx as DUShort)?;
    let parameters = read_type_list(accessor, parameters_off)?;
    ProtoElement { shorty, return_type, parameters }.ok()
}

fn read_type_list(
    accessor: &DexFileAccessor, type_list_off: DUInt,
) -> AsmResult<Vec<StrRef>> {
    if type_list_off == 0 { return Ok(vec![]); }
    let TypeList { type_id_indices, .. } = accessor.get_data_impl::<TypeList>(type_list_off)?;
    type_id_indices.map_res(|type_idx| read_type(accessor, *type_idx))
}

fn read_type(
    accessor: &DexFileAccessor, type_idx: DUShort,
) -> AsmResult<StrRef> {
    let dex_file = &accessor.file;
    let type_id = dex_file.type_ids.get(type_idx as usize)
        .ok_or_error(|| AsmErr::OutOfRange(type_idx as usize).e())?;
    read_string(accessor, type_id.descriptor_idx)
}

fn read_string(
    accessor: &DexFileAccessor, string_idx: u32,
) -> AsmResult<StrRef> {
    let dex_file = &accessor.file;
    let string_data_off = dex_file.string_ids.get(string_idx as usize)
        .ok_or_error(|| AsmErr::OutOfRange(string_idx as usize).e())?.string_data_off;
    Ok(accessor.get_data_impl::<StringData>(string_data_off)?.str_ref)
}

impl AsElement<ClassContentElement> for ClassDataItem {
    fn to_element(
        &self, accessor: &DexFileAccessor, _previous: Option<&ClassDataItem>,
    ) -> AsmResult<ClassContentElement> {
        let static_fields = to_element_list_impl(
            accessor, &self.static_fields,
        )?;
        let instance_fields = to_element_list_impl(
            accessor, &self.instance_fields,
        )?;
        let direct_methods = to_element_list_impl(
            accessor, &self.direct_methods,
        )?;
        let virtual_methods = to_element_list_impl(
            accessor, &self.virtual_methods,
        )?;
        ClassContentElement {
            static_fields, instance_fields,
            direct_methods, virtual_methods,
        }.ok()
    }
}

impl AsElement<FieldElement> for EncodedField {
    fn to_element(
        &self, accessor: &DexFileAccessor, previous: Option<&EncodedField>,
    ) -> AsmResult<FieldElement> {
        let previous_field_idx = previous.map_or(0, |f| f.field_idx_diff.value());
        let field_idx = self.field_idx_diff.value() + previous_field_idx;
        read_field(accessor, self, field_idx)
    }
}

impl AsElement<MethodElement> for EncodedMethod {
    fn to_element(
        &self, accessor: &DexFileAccessor, previous: Option<&EncodedMethod>,
    ) -> AsmResult<MethodElement> {
        let previous_method_idx = previous.map_or(0, |m| m.method_idx_diff.value());
        let method_idx = self.method_idx_diff.value() + previous_method_idx;
        read_method(accessor, self, method_idx)
    }
}
