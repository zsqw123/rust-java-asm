use crate::err::AsmResult;
use crate::jvms::element::{AttributeInfo, ClassFile, Const, CPInfo, FieldInfo, MethodInfo};
use crate::jvms::write::bytes::{FromWriteContext, WriteContext};
use crate::jvms::write::push_items;

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

