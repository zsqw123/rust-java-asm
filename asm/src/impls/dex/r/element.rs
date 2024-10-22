use crate::dex::element::{AsElement, ClassContentElement, FieldElement, MethodElement};
use crate::dex::{ClassDataItem, DexFileAccessor, EncodedField, EncodedMethod, FieldId, MethodId};
use crate::err::{AsmResultExt, AsmResultOkExt};
use crate::impls::dex::r::accessor::ProtoConst;
use crate::{AsmErr, AsmResult};

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

pub fn read_field(
    accessor: &DexFileAccessor, encoded_field: &EncodedField, field_idx: u32,
) -> AsmResult<FieldElement> {
    let dex_file = &accessor.file;
    let field_id = dex_file.field_ids.get(field_idx as usize)
        .ok_or_error(|| AsmErr::OutOfRange(field_idx as usize).e())?;
    let FieldId { type_idx, name_idx, .. } = *field_id;
    let descriptor = accessor.get_type(type_idx)?;
    let name = accessor.get_str(name_idx)?;
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
    let ProtoConst { shorty: shorty_descriptor, return_type, parameters } = accessor.get_proto(proto_idx)?;
    let name = accessor.get_str(name_idx)?;
    let code_off = encoded_method.code_off.value();
    MethodElement { access_flags, name, shorty_descriptor, return_type, parameters, code_off }.ok()
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
