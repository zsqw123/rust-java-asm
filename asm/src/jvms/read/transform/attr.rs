use crate::constants::Constants;
use crate::err::AsmResult;
use crate::jvms::attr::{Attribute, ExceptionTable, StackMapFrame};
use crate::jvms::element::{AttributeInfo, CPInfo};
use crate::jvms::read::bytes::ReadContext;
use crate::jvms::read::transform::transform_attrs;
use crate::jvms::read::util::read_utf8_from_cp;

pub(crate) fn transform_attr(attribute_info: &AttributeInfo, cp: &Vec<CPInfo>) -> AsmResult<AttributeInfo> {
    let attribute_name_index = attribute_info.attribute_name_index;
    let attribute_length = attribute_info.attribute_length;
    let info = attribute_info.info.clone();
    let Attribute::Custom(bytes) = info else { return Ok(attribute_info.clone()); };
    let mut context = ReadContext {
        bytes: &bytes,
        index: &mut 0,
    };
    let utf8 = read_utf8_from_cp(attribute_name_index as usize, cp)?;
    let attr = match utf8.as_str() {
        Constants::CONSTANT_VALUE => {
            Attribute::ConstantValue {
                constantvalue_index: context.read()?,
            }
        }
        Constants::CODE => {
            let max_stack: u16 = context.read()?;
            let max_locals: u16 = context.read()?;
            let code_length: u32 = context.read()?;
            let code: Vec<u8> = context.read_vec(code_length as usize)?;
            let exception_table_length: u16 = context.read()?;
            let exception_table: Vec<ExceptionTable> = context.read_vec(exception_table_length as usize)?;
            let attributes_count: u16 = context.read()?;
            let mut attributes: Vec<AttributeInfo> = context.read_vec(attributes_count as usize)?;
            transform_attrs(&mut attributes, cp)?;
            Attribute::Code {
                max_stack, max_locals,
                code_length, code,
                exception_table_length, exception_table,
                attributes_count, attributes,
            }
        }
        Constants::STACK_MAP_TABLE => {
            let number_of_entries: u16 = context.read()?;
            let entries: Vec<StackMapFrame> = context.read_vec(number_of_entries as usize)?;
            Attribute::StackMapTable { number_of_entries, entries }
        }
        Constants::EXCEPTIONS => {
            let number_of_exceptions = context.read()?;
        }
        Constants::INNER_CLASSES => {}
        Constants::ENCLOSING_METHOD => {}
        Constants::SYNTHETIC => {}
        Constants::SIGNATURE => {}
        Constants::SOURCE_FILE => {}
        Constants::SOURCE_DEBUG_EXTENSION => {}
        Constants::LINE_NUMBER_TABLE => {}
        Constants::LOCAL_VARIABLE_TABLE => {}
        Constants::LOCAL_VARIABLE_TYPE_TABLE => {}
        Constants::DEPRECATED => {}
        Constants::RUNTIME_VISIBLE_ANNOTATIONS => {}
        Constants::RUNTIME_INVISIBLE_ANNOTATIONS => {}
        Constants::RUNTIME_VISIBLE_PARAMETER_ANNOTATIONS => {}
        Constants::RUNTIME_INVISIBLE_PARAMETER_ANNOTATIONS => {}
        Constants::RUNTIME_VISIBLE_TYPE_ANNOTATIONS => {}
        Constants::RUNTIME_INVISIBLE_TYPE_ANNOTATIONS => {}
        Constants::ANNOTATION_DEFAULT => {}
        Constants::BOOTSTRAP_METHODS => {}
        Constants::METHOD_PARAMETERS => {}
        Constants::MODULE => {}
        Constants::MODULE_PACKAGES => {}
        Constants::MODULE_MAIN_CLASS => {}
        Constants::NEST_HOST => {}
        Constants::NEST_MEMBERS => {}
        Constants::PERMITTED_SUBCLASSES => {}
        Constants::RECORD => {}
        _ => Attribute::Custom(bytes)
    };
    let attribute_info = AttributeInfo {
        attribute_name_index,
        attribute_length,
        info: attr,
    };
    Ok(attribute_info)
}
