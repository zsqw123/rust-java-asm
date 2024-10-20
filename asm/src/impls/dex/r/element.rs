use crate::dex::element::{AsElement, ClassContentElement, FieldElement, MethodElement};
use crate::dex::{ClassDataItem, DexFileAccessor, EncodedField, EncodedMethod};
use crate::err::AsmResultOkExt;
use crate::AsmResult;

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
        let previous = vec.get(index - 1);
        // SAFETY: index is always less than vec_size
        let c = vec.get(index).unwrap();
        let element = c.to_element(accessor, previous)?;
        result.push(element);
        index += 1;
    }
    Ok(result)
}

impl AsElement<ClassContentElement> for ClassDataItem {
    fn to_element(
        &self, accessor: &DexFileAccessor, previous: Option<&ClassDataItem>,
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
        todo!()
    }
}

impl AsElement<MethodElement> for EncodedMethod {
    fn to_element(
        &self, accessor: &DexFileAccessor, previous: Option<&EncodedMethod>,
    ) -> AsmResult<MethodElement> {
        todo!()
    }
}
