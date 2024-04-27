use crate::{asm_visitor, asm_visitor_impl};
use crate::annotation::AnnotationVisitor;

asm_visitor! {
    pub struct FieldVisitor<'a>
}

asm_visitor_impl! {
    /// todo pub fn visitTypeAnnotation
    impl FieldVisitor<'_> {
       /// Visits an annotation of the field.
       ///
       /// - descriptor the class descriptor of the annotation class.
       /// - visible `true` if the annotation is visible at runtime.
       ///
       /// returns a visitor to visit the annotation values, or `null` if this visitor is not
       /// interested in visiting this annotation.
       ///
        pub fn visit_annotation(&self, descriptor: &str, visible: bool) -> Option<&AnnotationVisitor>;
        pub fn visit_attribute(&self) -> Option<()>;
        pub fn visit_end(&self) -> Option<()>;
    }
}
