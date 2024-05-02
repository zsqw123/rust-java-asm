use crate::err::AsmResult;
use crate::jvms::element::{AttributeInfo, ClassFile, CPInfo, FieldInfo, MethodInfo};
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
        todo!()
    }
}

impl FromWriteContext<FieldInfo> for FieldInfo {
    fn from_context(context: &mut WriteContext, from: FieldInfo) -> AsmResult<()> {
        todo!()
    }
}

impl FromWriteContext<MethodInfo> for MethodInfo {
    fn from_context(context: &mut WriteContext, from: MethodInfo) -> AsmResult<()> {
        todo!()
    }
}

impl FromWriteContext<AttributeInfo> for AttributeInfo {
    fn from_context(context: &mut WriteContext, from: AttributeInfo) -> AsmResult<()> {
        todo!()
    }
}
