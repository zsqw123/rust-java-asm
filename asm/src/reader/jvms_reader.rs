use std::io::{BufReader, Read};

use crate::constants::Constants;
use crate::err::{AsmErr, AsmResult};
use crate::jvms::{AttributeInfo, ClassFile, Const, CPInfo, FieldInfo, MethodInfo};
use crate::jvms::Const::Utf8;
use crate::reader::bytes_reader::{FromReadContext, ReadContext};

struct JvmsClassReader {}

impl JvmsClassReader {
    fn read_class_file<T: Read>(read: T) -> AsmResult<ClassFile> {
        let mut reader = BufReader::new(read);
        let mut str = String::new();
        let read_result = reader.read_to_string(&mut str);
        if let Err(e) = read_result {
            return Err(AsmErr::ContentReadErr { io_error: e });
        };
        let bytes = str.as_bytes();
        let index = &mut 0;
        ClassFile::from_context(&mut ReadContext { bytes, index })
    }
}


impl FromReadContext<ClassFile> for ClassFile {
    fn from_context(context: &mut ReadContext) -> AsmResult<ClassFile> {
        let magic: u32 = context.read()?;
        let minor_version: u16 = context.read()?;
        let major_version: u16 = context.read()?;
        let constant_pool_count: u16 = context.read()?;
        let constant_pool: Vec<CPInfo> = context.read_vec(constant_pool_count as usize)?;
        let access_flags: u16 = context.read()?;
        let this_class: u16 = context.read()?;
        let super_class: u16 = context.read()?;
        let interfaces_count: u16 = context.read()?;
        let interfaces: Vec<u16> = context.read_vec::<u16>(interfaces_count as usize)?;
        let fields_count: u16 = context.read()?;
        let fields: Vec<FieldInfo> = context.read_vec(fields_count as usize)?;
        let methods_count: u16 = context.read()?;
        let methods: Vec<MethodInfo> = context.read_vec(methods_count as usize)?;
        let attributes_count: u16 = context.read()?;
        let attributes: Vec<AttributeInfo> = context.read_vec(attributes_count as usize)?;
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

impl FromReadContext<CPInfo> for CPInfo {
    fn from_context(context: &mut ReadContext) -> AsmResult<CPInfo> {
        let tag: u8 = context.read()?;
        let info: Const = Const::from_context(context, tag)?;
        Ok(CPInfo { tag, info })
    }
}

struct MemberInfo {
    access_flags: u16,
    name_index: u16,
    descriptor_index: u16,
    attributes_count: u16,
    attributes: Vec<AttributeInfo>,
}

fn member_from_context(context: &mut ReadContext) -> AsmResult<MemberInfo> {
    let access_flags: u16 = context.read()?;
    let name_index: u16 = context.read()?;
    let descriptor_index: u16 = context.read()?;
    let attributes_count: u16 = context.read()?;
    let attributes: Vec<AttributeInfo> = context.read_vec(attributes_count as usize)?;
    let member = MemberInfo {
        access_flags, name_index, descriptor_index,
        attributes_count, attributes,
    };
    Ok(member)
}

impl FromReadContext<FieldInfo> for FieldInfo {
    fn from_context(context: &mut ReadContext) -> AsmResult<FieldInfo> {
        let MemberInfo {
            access_flags, name_index, descriptor_index,
            attributes_count, attributes,
        } = member_from_context(context)?;
        let field = FieldInfo {
            access_flags, name_index, descriptor_index,
            attributes_count, attributes,
        };
        Ok(field)
    }
}

impl FromReadContext<MethodInfo> for MethodInfo {
    fn from_context(context: &mut ReadContext) -> AsmResult<MethodInfo> {
        let MemberInfo {
            access_flags, name_index, descriptor_index,
            attributes_count, attributes,
        } = member_from_context(context)?;
        let method = MethodInfo {
            access_flags, name_index, descriptor_index,
            attributes_count, attributes,
        };
        Ok(method)
    }
}

impl FromReadContext<AttributeInfo> for AttributeInfo {
    fn from_context(context: &mut ReadContext) -> AsmResult<AttributeInfo> {
        todo!()
    }
}

impl Const {
    #[allow(unused_variables)]
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
                        let length = context.read()?;
                        Utf8 { length, bytes: context.read_vec(length as usize)? }
                    }
                    _ => return Err(AsmErr::IllegalArgument {
                        info: format!("unknown const tag in const pool: {}", tag),
                    }),
                }
            };
        }

        let info = match_context! {
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
        };
        Ok(info)
    }
}

