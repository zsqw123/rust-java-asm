use crate::err::AsmResult;
use crate::jvms::element::{AttributeInfo, ClassFile, CPInfo, FieldInfo, MethodInfo};
use crate::jvms::read::transform::attr::transform_attr;

mod attr;

pub(crate) fn transform_class_file(origin: ClassFile) -> AsmResult<ClassFile> {
    let mut new_file = origin.clone();
    let cp_info = &new_file.constant_pool;
    transform_attrs(&mut new_file.attributes, cp_info)?;
    transform_fields(&mut new_file.fields, cp_info)?;
    transform_methods(&mut new_file.methods, cp_info)?;
    Ok(new_file)
}

fn transform_fields(fields: &mut Vec<FieldInfo>, cp: &Vec<CPInfo>) -> AsmResult<()> {
    for i in 0..fields.len() {
        fields[i] = transform_field(&fields[i], cp)?;
    };
    Ok(())
}

fn transform_field(field_info: &FieldInfo, cp: &Vec<CPInfo>) -> AsmResult<FieldInfo> {
    let mut field_info = field_info.clone();
    let attributes = &mut field_info.attributes;
    transform_attrs(attributes, cp)?;
    Ok(field_info)
}

fn transform_methods(methods: &mut Vec<MethodInfo>, cp: &Vec<CPInfo>) -> AsmResult<()> {
    for i in 0..methods.len() {
        methods[i] = transform_method(&methods[i], cp)?;
    };
    Ok(())
}

fn transform_method(method_info: &MethodInfo, cp: &Vec<CPInfo>) -> AsmResult<MethodInfo> {
    let mut method_info = method_info.clone();
    let attributes = &mut method_info.attributes;
    transform_attrs(attributes, cp)?;
    Ok(method_info)
}

fn transform_attrs(attributes: &mut Vec<AttributeInfo>, cp: &Vec<CPInfo>) -> AsmResult<()> {
    for i in 0..attributes.len() {
        attributes[i] = transform_attr(&attributes[i], cp)?;
    };
    Ok(())
}

