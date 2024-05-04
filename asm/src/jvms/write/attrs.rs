use java_asm_internal::err::AsmResult;
use crate::jvms::attr::Attribute;
use crate::jvms::attr::{ExceptionTable, StackMapFrame, VerificationTypeInfo};
use java_asm_internal::write::jvms::{IntoWriteContext, WriteContext};
use crate::jvms::write::{push_enum, push_items};

impl IntoWriteContext<Attribute> for Attribute {
    fn into_context(context: &mut WriteContext, from: Attribute) -> AsmResult<()> {
        push_enum!(context, from;
            @Attribute::Custom { vec };
            Attribute::ConstantValue { constantvalue_index };
            Attribute::Code {
                max_stack, max_locals,
                code_length, code,
                exception_table_length, exception_table,
                attributes_count, attributes,
            };
            Attribute::StackMapTable {
                number_of_entries,
                entries,
            };
            Attribute::Exceptions {
                number_of_exceptions,
                exception_index_table,
            };
            
        );
        Ok(())
    }
}

impl IntoWriteContext<ExceptionTable> for ExceptionTable {
    fn into_context(context: &mut WriteContext, from: ExceptionTable) -> AsmResult<()> {
        push_items!(
            context, from;
            start_pc, end_pc, handler_pc, catch_type,
        );
        Ok(())
    }
}

impl IntoWriteContext<StackMapFrame> for StackMapFrame {
    fn into_context(context: &mut WriteContext, from: StackMapFrame) -> AsmResult<()> {
        push_enum!(
            context, from;
            StackMapFrame::SameFrame { frame_type };
            StackMapFrame::SameLocals1StackItemFrame { frame_type, verification_type_info };
            StackMapFrame::SameLocals1StackItemFrameExtended { 
                frame_type, offset_delta, verification_type_info,
            };
            StackMapFrame::ChopFrame { frame_type, offset_delta };
            StackMapFrame::SameFrameExtended { frame_type, offset_delta };
            StackMapFrame::AppendFrame { frame_type, offset_delta, locals };
            StackMapFrame::FullFrame { 
                frame_type, offset_delta, 
                number_of_locals, locals,
                number_of_stack_items, stack, 
            };
        );
        Ok(())
    }
}

impl IntoWriteContext<VerificationTypeInfo> for VerificationTypeInfo {
    fn into_context(context: &mut WriteContext, from: VerificationTypeInfo) -> AsmResult<()> {
        push_enum!(
            context, from;
            VerificationTypeInfo::Top { tag };
            VerificationTypeInfo::Integer { tag };
            VerificationTypeInfo::Float { tag };
            VerificationTypeInfo::Null { tag };
            VerificationTypeInfo::UninitializedThis { tag };
            VerificationTypeInfo::Object { tag, cpool_index };
            VerificationTypeInfo::Uninitialized { tag, offset };
            VerificationTypeInfo::Long { tag };
            VerificationTypeInfo::Double { tag };
        );
        Ok(())
    }
}