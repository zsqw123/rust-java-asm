use crate::asm_visitor;

asm_visitor! {
    pub struct MethodVisitor<'a>
}

impl MethodVisitor<'_> {
    // -----------------------------------------------------------------------------------------------
    // Parameters, annotations and non standard attributes
    // -----------------------------------------------------------------------------------------------
    fn visit_parameter(name: &str, access: u8) {}
}
