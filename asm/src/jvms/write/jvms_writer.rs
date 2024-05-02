use crate::err::AsmResult;
use crate::jvms::element::{Attribute, AttributeInfo, ClassFile, Const, CPInfo, ExceptionTable, FieldInfo, MethodInfo, StackMapFrame, VerificationTypeInfo};
use crate::jvms::write::bytes::{FromWriteContext, WriteContext};

macro_rules! push_items {
    (
        $contextExpr:expr, $fromExpr:expr;
        $($fieldIdent:ident $(,)?)*
    ) => {
        $(
            $contextExpr.push($fromExpr.$fieldIdent)?;
        )*
    };
}

macro_rules! push_enum {
    (
        $contextExpr:expr, $fromExpr:expr;
        $(@$enumPath1:path {
            $( $fieldIdent1:ident $(,)? )*
        };)*
        $($enumPath2:path {
            $( $fieldIdent2:ident $(,)? )*
        };)*
    ) => {
        match $fromExpr {
            $($enumPath1($($fieldIdent1,)*) => {
                $(
                    $contextExpr.push($fieldIdent1)?;
                )*
            })*
            $($enumPath2{$($fieldIdent2,)*} => {
                $(
                    $contextExpr.push($fieldIdent2)?;
                )*
            })* 
        }
    };
}

impl FromWriteContext<ClassFile> for ClassFile {
    fn from_context(context: &mut WriteContext, from: ClassFile) -> AsmResult<()> {
        push_items!(
            context, from;
            magic, minor_version, major_version,
            constant_pool_count, constant_pool, access_flags,
            this_class, super_class,
            interfaces_count, interfaces,
            fields_count, fields,
            methods_count, methods,
            attributes_count, attributes
        );
        Ok(())
    }
}

impl FromWriteContext<CPInfo> for CPInfo {
    fn from_context(context: &mut WriteContext, from: CPInfo) -> AsmResult<()> {
        push_items!(
            context, from;
            tag, info,
        );
        Ok(())
    }
}

impl FromWriteContext<Const> for Const {
    fn from_context(context: &mut WriteContext, from: Const) -> AsmResult<()> {
        todo!()
    }
}

impl FromWriteContext<FieldInfo> for FieldInfo {
    fn from_context(context: &mut WriteContext, from: FieldInfo) -> AsmResult<()> {
        push_items!(
            context, from;
            access_flags, name_index, descriptor_index,
            attributes_count, attributes,
        );
        Ok(())
    }
}

impl FromWriteContext<MethodInfo> for MethodInfo {
    fn from_context(context: &mut WriteContext, from: MethodInfo) -> AsmResult<()> {
        push_items!(
            context, from;
            access_flags, name_index, descriptor_index,
            attributes_count, attributes,
        );
        Ok(())
    }
}

impl FromWriteContext<AttributeInfo> for AttributeInfo {
    fn from_context(context: &mut WriteContext, from: AttributeInfo) -> AsmResult<()> {
        push_items!(
            context, from;
            attribute_name_index, attribute_length, info,
        );
        Ok(())
    }
}

impl FromWriteContext<Attribute> for Attribute {
    fn from_context(context: &mut WriteContext, from: Attribute) -> AsmResult<()> {
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
        );
        Ok(())
    }
}

impl FromWriteContext<ExceptionTable> for ExceptionTable {
    fn from_context(context: &mut WriteContext, from: ExceptionTable) -> AsmResult<()> {
        push_items!(
            context, from;
            start_pc, end_pc, handler_pc, catch_type,
        );
        Ok(())
    }
}

impl FromWriteContext<StackMapFrame> for StackMapFrame {
    fn from_context(context: &mut WriteContext, from: StackMapFrame) -> AsmResult<()> {
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

impl FromWriteContext<VerificationTypeInfo> for VerificationTypeInfo {
    fn from_context(context: &mut WriteContext, from: VerificationTypeInfo) -> AsmResult<()> {
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
