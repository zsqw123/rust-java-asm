use crate::constants::Constants;
use crate::err::{AsmErr, AsmResult};
use crate::jvms::element::{Attribute, AttributeInfo, ClassFile, Const, CPInfo, ExceptionTable, FieldInfo, MethodInfo, StackMapFrame, VerificationTypeInfo};
use crate::jvms::frame::Frame;
use crate::jvms::read::bytes::{FromReadContext, ReadContext};

impl FromReadContext<ClassFile> for ClassFile {
    fn from_context(context: &mut ReadContext) -> AsmResult<ClassFile> {
        let magic: u32 = context.read()?;
        let minor_version: u16 = context.read()?;
        let major_version: u16 = context.read()?;
        let constant_pool_count: u16 = context.read()?;
        let constant_pool: Vec<CPInfo> = cp_infos_from_context(context, constant_pool_count as usize)?;
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
    /// Returns raw attributes in this section,
    /// All attributes will be treated at [Attribute::Custom]
    fn from_context(context: &mut ReadContext) -> AsmResult<AttributeInfo> {
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
                        let length: u16 = context.read()?;
                        Const::Utf8 { length, bytes: context.read_vec(length as usize)? }
                    }
                    _ => return Err(AsmErr::IllegalArgument(
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

impl FromReadContext<ExceptionTable> for ExceptionTable {
    fn from_context(context: &mut ReadContext) -> AsmResult<ExceptionTable> {
        Ok(ExceptionTable {
            start_pc: context.read()?,
            end_pc: context.read()?,
            handler_pc: context.read()?,
            catch_type: context.read()?,
        })
    }
}

impl FromReadContext<StackMapFrame> for StackMapFrame {
    fn from_context(context: &mut ReadContext) -> AsmResult<StackMapFrame> {
        let frame_type: u8 = context.read()?;
        let frame = match frame_type {
            0..=63 => StackMapFrame::SameFrame { frame_type },
            64..=127 => StackMapFrame::SameLocals1StackItemFrame {
                frame_type,
                verification_type_info: context.read()?,

            },
            247 => StackMapFrame::SameLocals1StackItemFrameExtended {
                frame_type,
                offset_delta: context.read()?,
                verification_type_info: context.read()?,
            },
            248..=250 => StackMapFrame::ChopFrame {
                frame_type,
                offset_delta: context.read()?,
            },
            251 => StackMapFrame::SameFrameExtended {
                frame_type,
                offset_delta: context.read()?,
            },
            252..=254 => StackMapFrame::AppendFrame {
                frame_type,
                offset_delta: context.read()?,
                locals: context.read_vec((frame_type - 251) as usize)?,
            },
            255 => {
                let offset_delta: u16 = context.read()?;
                let number_of_locals: u16 = context.read()?;
                let locals: Vec<VerificationTypeInfo> = context.read_vec(number_of_locals as usize)?;
                let number_of_stack_items: u16 = context.read()?;
                let stack: Vec<VerificationTypeInfo> = context.read_vec(number_of_stack_items as usize)?;
                StackMapFrame::FullFrame {
                    frame_type,
                    offset_delta,
                    number_of_locals,
                    locals,
                    number_of_stack_items,
                    stack,
                }
            },
            _ => return Err(AsmErr::IllegalArgument(
                format!("unknown frame type: {}", frame_type)
            ))
        };
        Ok(frame)
    }
}

impl FromReadContext<VerificationTypeInfo> for VerificationTypeInfo {
    fn from_context(context: &mut ReadContext) -> AsmResult<VerificationTypeInfo> {
        let tag: u8 = context.read()?;
        let type_info = match tag {
            Frame::ITEM_Top => VerificationTypeInfo::Top { tag },
            Frame::ITEM_Integer => VerificationTypeInfo::Integer { tag },
            Frame::ITEM_Float => VerificationTypeInfo::Float { tag },
            Frame::ITEM_Null => VerificationTypeInfo::Null { tag },
            Frame::ITEM_UninitializedThis => VerificationTypeInfo::UninitializedThis { tag },
            Frame::ITEM_Object => VerificationTypeInfo::Object { tag, cpool_index: context.read()? },
            Frame::ITEM_Uninitialized => VerificationTypeInfo::Uninitialized { tag, offset: context.read()? },
            Frame::ITEM_Long => VerificationTypeInfo::Long { tag },
            Frame::ITEM_Double => VerificationTypeInfo::Double { tag },
            _ => return Err(AsmErr::IllegalArgument(
                format!("unknown frame tag: {}", tag)
            ))
        };
        Ok(type_info)
    }
}
