use crate::err::{AsmErr, AsmResult};
use crate::impls::jvms::r::{ReadFrom, ReadContext};
use crate::impls::jvms::r::frame::Frame;
use crate::jvms::attr::{StackMapFrame, VerificationTypeInfo};

impl ReadFrom for VerificationTypeInfo {
    fn read_from(context: &mut ReadContext) -> AsmResult<VerificationTypeInfo> {
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
            _ => return Err(AsmErr::IllegalFormat(
                format!("unknown frame tag: {}", tag)
            ))
        };
        Ok(type_info)
    }
}

impl ReadFrom for StackMapFrame {
    fn read_from(context: &mut ReadContext) -> AsmResult<StackMapFrame> {
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
                    frame_type, offset_delta, number_of_locals, locals, number_of_stack_items, stack,
                }
            },
            _ => return Err(AsmErr::IllegalFormat(
                format!("unknown frame type: {}", frame_type)
            ))
        };
        Ok(frame)
    }
}
