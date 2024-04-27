use crate::{asm_visitor, asm_visitor_impl};
use crate::annotation::AnnotationVisitor;
use crate::attribute::Attribute;

asm_visitor! {
    pub struct FieldVisitor<'a>
}

pub enum FieldValue {
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    String(String),
}

// todo pub fn visitTypeAnnotation
asm_visitor_impl! {
    impl FieldVisitor<'_> {
        /// Visits an annotation of the field.
        ///
        /// - descriptor the class descriptor of the annotation class.
        /// - visible `true` if the annotation is visible at runtime.
        ///
        /// returns a visitor to visit the annotation values, or `null` if this visitor is not
        /// interested in visiting this annotation.
        pub fn visit_annotation(&self, descriptor: &str, visible: bool) -> Option<&AnnotationVisitor>;
        pub fn visit_attribute(&self, attr: &Attribute) -> Option<()>;
        pub fn visit_end(&self) -> Option<()>;
    }
}
