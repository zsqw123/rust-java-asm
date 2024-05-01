use crate::constants::Constants;
use crate::err::{AsmErr, AsmResult};
use crate::jvms::element::{Attribute, AttributeInfo, ClassFile, CPInfo};
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
            Attribute::Code {
                
            }
        },
        Constants::STACK_MAP_TABLE => {
            Attribute::StackMapTable {
                
            }
        }
        _ => return Err(AsmErr::ReadUTF8(""))
    }


    Ok(attribute_info)
}

