use java_asm_internal::err::{AsmErr, AsmResult};
use java_asm_internal::read::jvms::{FromReadContext, ReadContext};

use crate::jvms::attr::annotation::{AnnotationElementValue, AnnotationElementValueInfo};
use crate::jvms::attr::type_annotation::{TypeAnnotation, TypeAnnotationTargetInfo};

impl FromReadContext<AnnotationElementValueInfo> for AnnotationElementValueInfo {
    fn from_context(context: &mut ReadContext) -> AsmResult<AnnotationElementValueInfo> {
        let tag = context.read()?;
        let value = match tag {
            // byte, char, double, float, int, long, short, boolean, String
            // will be stored in the constant pool
            b'B' | b'C' | b'D' | b'F' | b'I' | b'J' | b'S' | b'Z' | b's' =>
                AnnotationElementValue::Const { const_value_index: context.read()? },
            // enum_const_value
            b'e' => AnnotationElementValue::EnumConst {
                type_name_index: context.read()?,
                const_name_index: context.read()?,
            },
            // class_info_index
            b'c' => AnnotationElementValue::Class { class_info_index: context.read()? },
            // annotation_value
            b'@' => AnnotationElementValue::Annotation { annotation_value: context.read()? },
            // array_value
            b'[' => {
                let num_values = context.read()?;
                let values = context.read_vec(num_values as usize)?;
                AnnotationElementValue::Array { num_values, values }
            },
            _ => return Err(AsmErr::IllegalArgument(
                format!("unknown tag `{}` when reading an annotation element value.", tag))
            ),
        };
        Ok(AnnotationElementValueInfo { tag, value })
    }
}

// ---------------------------
// type annotations impls
// ---------------------------

impl FromReadContext<TypeAnnotation> for TypeAnnotation {
    fn from_context(context: &mut ReadContext) -> AsmResult<TypeAnnotation> {
        let target_type = context.read()?;
        let target_info = match target_type {
            0x00 | 0x01 => TypeAnnotationTargetInfo::TypeParameter { type_parameter_index: context.read()? },
            0x10 => TypeAnnotationTargetInfo::SuperType { supertype_index: context.read()? },
            0x11 | 0x12 => TypeAnnotationTargetInfo::TypeParameterBound {
                type_parameter_index: context.read()?,
                bound_index: context.read()?,
            },
            0x13 | 0x14 | 0x15 => TypeAnnotationTargetInfo::Empty,
            0x16 => TypeAnnotationTargetInfo::FormalParameter { formal_parameter_index: context.read()? },
            0x17 => TypeAnnotationTargetInfo::Throws { throws_type_index: context.read()? },
            0x40 | 0x41 => {
                let table_length = context.read()?;
                let table = context.read_vec(table_length as usize)?;
                TypeAnnotationTargetInfo::LocalVar { table_length, table }
            },
            0x42 => TypeAnnotationTargetInfo::Catch { exception_table_index: context.read()? },
            0x43 | 0x44 | 0x45 | 0x46 => TypeAnnotationTargetInfo::Offset { offset: context.read()? },
            0x47 | 0x48 | 0x49 | 0x4A | 0x4B => TypeAnnotationTargetInfo::TypeArgument {
                offset: context.read()?,
                type_argument_index: context.read()?,
            },
            _ => return Err(AsmErr::IllegalArgument(
                format!("unknown target type {} when reading a type annotation.", target_type))
            ),
        };
        let target_path = context.read()?;
        let type_index = context.read()?;
        let num_element_value_pairs = context.read()?;
        let element_value_pairs = context.read_vec(num_element_value_pairs as usize)?;
        Ok(TypeAnnotation { 
            target_type, target_info, target_path,
            type_index, num_element_value_pairs, element_value_pairs
        })
    }
}
