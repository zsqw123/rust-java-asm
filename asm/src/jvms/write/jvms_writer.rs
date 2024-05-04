use java_asm_internal::err::AsmResult;
use java_asm_internal::write::jvms::{IntoWriteContext, WriteContext};

use crate::jvms::element::{AttributeInfo, ClassFile, Const, CPInfo, FieldInfo, MethodInfo};
use crate::jvms::write::push_items;

impl IntoWriteContext<ClassFile> for ClassFile {
    fn into_context(context: &mut WriteContext, from: ClassFile) -> AsmResult<()> {
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

impl IntoWriteContext<CPInfo> for CPInfo {
    fn into_context(context: &mut WriteContext, from: CPInfo) -> AsmResult<()> {
        push_items!(
            context, from;
            tag, info,
        );
        Ok(())
    }
}

impl IntoWriteContext<Const> for Const {
    fn into_context(context: &mut WriteContext, from: Const) -> AsmResult<()> {
        todo!()
    }
}

impl IntoWriteContext<FieldInfo> for FieldInfo {
    fn into_context(context: &mut WriteContext, from: FieldInfo) -> AsmResult<()> {
        push_items!(
            context, from;
            access_flags, name_index, descriptor_index,
            attributes_count, attributes,
        );
        Ok(())
    }
}

impl IntoWriteContext<MethodInfo> for MethodInfo {
    fn into_context(context: &mut WriteContext, from: MethodInfo) -> AsmResult<()> {
        push_items!(
            context, from;
            access_flags, name_index, descriptor_index,
            attributes_count, attributes,
        );
        Ok(())
    }
}

impl IntoWriteContext<AttributeInfo> for AttributeInfo {
    fn into_context(context: &mut WriteContext, from: AttributeInfo) -> AsmResult<()> {
        push_items!(
            context, from;
            attribute_name_index, attribute_length, info,
        );
        Ok(())
    }
}

