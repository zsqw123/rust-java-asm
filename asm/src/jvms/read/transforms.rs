use crate::constants::Constants;
use crate::err::AsmResult;
use crate::jvms::element::{Attribute, AttributeInfo, ClassFile, CPInfo, ExceptionTable, StackMapFrame};
use crate::jvms::read::bytes_reader::ReadContext;
use crate::jvms::read::util::read_utf8_from_cp;

pub(crate) fn transform_class_file(origin: ClassFile) -> AsmResult<ClassFile> {
    origin
}

fn transform_attr(attribute_info: AttributeInfo, cp: &Vec<CPInfo>) -> AsmResult<AttributeInfo> {
    let AttributeInfo { attribute_name_index, attribute_length, info } = attribute_info;
    let Attribute::Custom(bytes) = info;
    let mut context = ReadContext {
        bytes: &bytes,
        index: &mut 0,
    };
    let utf8 = read_utf8_from_cp(attribute_name_index as usize, cp)?;
    let attr = match utf8.as_str() {
        Constants::CONSTANT_VALUE => {
            Attribute::ConstantValue {
                attribute_name_index: context.read()?,
                attribute_length: context.read()?,
                constantvalue_index: context.read()?,
            }
        },
        Constants::CODE => {
            let attribute_name_index: u16 = context.read()?;
            let attribute_length: u32 = context.read()?;
            let max_stack: u16 = context.read()?;
            let max_locals: u16 = context.read()?;
            let code_length: u32 = context.read()?;
            let code: Vec<u8> = context.read_vec(code_length as usize)?;
            let exception_table_length: u16 = context.read()?;
            let exception_table: Vec<ExceptionTable> = context.read_vec(exception_table_length as usize)?;
            let attributes_count: u16 = context.read()?;
            let attributes: Vec<AttributeInfo> = context.read_vec(attributes_count as usize)?;
            for i in 0..attributes.len() {
                attributes[i] = transform_attr(copy_attribute(&attributes[i]), cp)?;
            }
            Attribute::Code {
                attribute_name_index, attribute_length,
                max_stack, max_locals,
                code_length, code,
                exception_table_length, exception_table,
                attributes_count, attributes,
            }
        },
        Constants::STACK_MAP_TABLE => {
            let attribute_name_index: u16 = context.read()?;
            let attribute_length: u32 = context.read()?;
            let number_of_entries: u16 = context.read()?;
            let entries: Vec<StackMapFrame> = context.read_vec(number_of_entries as usize)?;
            Attribute::StackMapTable {
                attribute_name_index, attribute_length,
                number_of_entries, entries,
            }
        }
        _ => Attribute::Custom(bytes)
    };
    let attribute_info = AttributeInfo {
        attribute_name_index, attribute_length, info: attr,
    };
    Ok(attribute_info)
}

fn copy_attribute(origin: &AttributeInfo) -> AttributeInfo {
    let &AttributeInfo { attribute_name_index, attribute_length, info } = origin;
    AttributeInfo { attribute_name_index, attribute_length, info }
}
