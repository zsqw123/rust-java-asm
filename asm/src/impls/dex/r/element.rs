use crate::dex::element::{AsElement, ClassContentElement, FieldElement, MethodElement};
use crate::dex::{ClassDataItem, DUInt, DULeb128P1, DebugInfoItem, DexFileAccessor, EncodedField, EncodedMethod, FieldId, LocalVar, MethodId};
use crate::err::{AsmResultExt, AsmResultOkExt};
use crate::impls::dex::r::accessor::ProtoConst;
use crate::{AsmErr, AsmResult};

#[derive(Default)]
pub struct DebugInfoMap {
    // addr, source line, alternative source file name_idx
    pub records: LineTable<(DUInt, DULeb128P1)>,
    // addr, local var
    pub local_vars: LineTable<LocalVar>,
}


impl DebugInfoMap {
    pub(crate) fn from_raw(debug_info_item: Option<DebugInfoItem>) -> Self {
        let Some(debug_info_item) = debug_info_item else { return Default::default() };
        let DebugInfoItem { records: records_vec, local_vars: local_var_vec, .. } = debug_info_item;
        let mut records = Vec::with_capacity(records_vec.len());
        let mut local_vars = Vec::with_capacity(local_var_vec.len());
        for record in records_vec {
            let (addr, source_line, alternative_source_file) = record;
            records.push((addr, (source_line, alternative_source_file)));
        };
        for local_var in local_var_vec {
            let addr = local_var.start_addr;
            if let Some(addr) = addr {
                local_vars.push((addr, local_var.clone()));
            }
        }
        local_vars.shrink_to_fit();
        let records = LineTable::new(records);
        let local_vars = LineTable::new(local_vars);
        DebugInfoMap { records, local_vars }
    }
}

pub struct LineTable<T> {
    current_off: DUInt,
    current_idx: usize,
    pub lines: Vec<(DUInt, T)>,
}

impl<T> Default for LineTable<T> {
    fn default() -> Self {
        LineTable::new(vec![])
    }
}

impl<T> LineTable<T> {
    pub fn new(lines: Vec<(DUInt, T)>) -> Self {
        LineTable { current_off: 0, current_idx: 0, lines }
    }

    pub fn move_to(&mut self, offset: DUInt) -> Vec<&T> {
        let mut result = Vec::new();
        loop {
            if self.current_idx >= self.lines.len() { break }
            let Some((current_off, t)) = self.lines.get(self.current_idx) else { break };
            if *current_off > offset { break }
            result.push(t);
            self.current_off = *current_off;
            self.current_idx += 1;
        };
        result
    }
}

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
    let mut prev_index = 0;
    for c in vec {
        let (current_index, element) = c.to_element(accessor, prev_index)?;
        prev_index = current_index;
        result.push(element);
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
    let access_flags = encoded_field.access_flags.value();
    let descriptor = accessor.get_type(type_idx)?;
    let name = accessor.get_str(name_idx)?;
    FieldElement { access_flags, name, descriptor }.ok()
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

impl ClassDataItem {
    pub(crate) fn to_element(
        &self, accessor: &DexFileAccessor
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
        &self, accessor: &DexFileAccessor, previous_index: u32,
    ) -> AsmResult<(u32, FieldElement)> {
        let field_idx = self.field_idx_diff.value() + previous_index;
        Ok((field_idx, read_field(accessor, self, field_idx)?))
    }
}

impl AsElement<MethodElement> for EncodedMethod {
    fn to_element(
        &self, accessor: &DexFileAccessor, previous_index: u32,
    ) -> AsmResult<(u32, MethodElement)> {
        let method_idx = self.method_idx_diff.value() + previous_index;
        Ok((method_idx, read_method(accessor, self, method_idx)?))
    }
}
