use crate::err::{AsmErr, AsmResult};

use crate::constants::Constants;
use crate::impls::jvms::r::{ReadFrom, ReadContext};
use crate::jvms::attr::Attribute;
use crate::jvms::element::{AttributeInfo, ClassFile, Const, CPInfo, FieldInfo, MethodInfo};

impl ReadFrom for ClassFile {
    fn read_from(context: &mut ReadContext) -> AsmResult<ClassFile> {
        let magic: u32 = context.read()?;
        let minor_version: u16 = context.read()?;
        let major_version: u16 = context.read()?;
        let constant_pool_count: u16 = context.read()?;
        let constant_pool: Vec<CPInfo> = cp_infos_from_context(context, constant_pool_count as usize)?;
        let access_flags: u16 = context.read()?;
        let this_class: u16 = context.read()?;
        let super_class: u16 = context.read()?;
        let interfaces_count: u16 = context.read()?;
        let interfaces: Vec<u16> = context.read_vec::<u16>(interfaces_count)?;
        let fields_count: u16 = context.read()?;
        let fields: Vec<FieldInfo> = context.read_vec(fields_count)?;
        let methods_count: u16 = context.read()?;
        let methods: Vec<MethodInfo> = context.read_vec(methods_count)?;
        let attributes_count: u16 = context.read()?;
        let attributes: Vec<AttributeInfo> = context.read_vec(attributes_count)?;
        let cf = ClassFile {
            magic, minor_version, major_version,
            constant_pool_count, constant_pool,
            access_flags, this_class, super_class,
            interfaces_count, interfaces,
            fields_count, fields,
            methods_count, methods,
            attributes_count, attributes,
        };
        Ok(cf)
    }
}

/// due to [Const::Double] & [Const::Long] occupy 2 slots to store.
/// special treated for reading[Vec<CPInfo>]
fn cp_infos_from_context(context: &mut ReadContext, max_len: usize) -> AsmResult<Vec<CPInfo>> {
    let mut max_len = max_len;
    let mut result = Vec::with_capacity(max_len);
    result.push(CPInfo { tag: 0, info: Const::Invalid });
    max_len -= 1;
    while max_len > 0 {
        let tag: u8 = context.read()?;
        let info: Const = Const::from_context(context, tag)?;
        match tag {
            Constants::CONSTANT_Long | Constants::CONSTANT_Double => {
                max_len -= 2;
            },
            _ => { max_len -= 1; }
        }
        result.push(CPInfo { tag, info });
    };
    Ok(result)
}

impl ReadFrom for AttributeInfo {
    /// Returns raw attributes in this section,
    /// All attributes will be treated at [Attribute::Custom]
    fn read_from(context: &mut ReadContext) -> AsmResult<AttributeInfo> {
        let attribute_name_index: u16 = context.read()?;
        let attribute_length: u32 = context.read()?;
        let attribute_vec: Vec<u8> = context.read_vec(attribute_length as usize)?;
        let info = Attribute::Custom(attribute_vec);
        let attribute_info = AttributeInfo {
            attribute_name_index, attribute_length, info,
        };
        Ok(attribute_info)
    }
}

impl Const {
    #[allow(unused_variables)] // rust can not analyze the usages which captured by macro.
    fn from_context(context: &mut ReadContext, tag: u8) -> AsmResult<Const> {
        macro_rules! match_context {
            {$($constName:ident => $constType:ident {
                $($fieldIdent:ident $(,)?)*
            },)*} => {
                match tag {
                    $(Constants::$constName => Const::$constType {
                        $($fieldIdent: context.read()?,)*
                    },)*
                    Constants::CONSTANT_Utf8 => {
                        let length: u16 = context.read()?;
                        Const::Utf8 { length, bytes: context.read_vec(length as usize)? }
                    }
                    _ => return Err(AsmErr::IllegalFormat(
                        format!("unknown const tag in const pool: {}", tag),
                    )),
                }
            };
        }

        let info = match_context! {
            CONSTANT_Invalid => Invalid {},
            CONSTANT_Class => Class { name_index },
            // refs
            CONSTANT_Fieldref => Field { class_index, name_and_type_index },
            CONSTANT_Methodref => Method { class_index,name_and_type_index },
            CONSTANT_InterfaceMethodref => InterfaceMethod { class_index, name_and_type_index },
            // numbers
            CONSTANT_Integer => Integer { bytes },
            CONSTANT_Float => Float { bytes },
            CONSTANT_Long => Long { high_bytes, low_bytes },
            CONSTANT_Double => Double { high_bytes, low_bytes },
            // others
            CONSTANT_NameAndType => NameAndType { name_index, descriptor_index },
            CONSTANT_MethodHandle => MethodHandle { reference_kind, reference_index},
            CONSTANT_MethodType => MethodType { descriptor_index },
            CONSTANT_Dynamic => Dynamic { bootstrap_method_attr_index, name_and_type_index },
            CONSTANT_InvokeDynamic => InvokeDynamic { bootstrap_method_attr_index, name_and_type_index },
            CONSTANT_Module => Module { name_index },
            CONSTANT_Package => Package { name_index },
            // string
            CONSTANT_String => String { string_index },
        };
        Ok(info)
    }
}
