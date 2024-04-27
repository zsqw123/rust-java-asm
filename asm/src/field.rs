use crate::annotation::AnnotationVisitor;

pub struct FieldVisitor<'a> {
    pub delegated: Option<&'a FieldVisitor<'a>>,
}

/// todo pub fn visitTypeAnnotation
impl<'a> FieldVisitor<'a> {
    /// 
    /// Visits an annotation of the field.
    /// 
    /// 1. descriptor the class descriptor of the annotation class.
    /// 2. visible {@literal true} if the annotation is visible at runtime.
    /// 
    /// returns a visitor to visit the annotation values, or {@literal null} if this visitor is not
    ///     interested in visiting this annotation.
    /// 
    pub fn visit_annotation(&self, descriptor: &String, visible: bool) -> Option<&AnnotationVisitor> {
        self.delegated?.visit_annotation(descriptor, visible)
    }

    pub fn visit_attribute(&self) -> Option<()> {
        self.delegated?.visit_attribute()
    }

    pub fn visit_end(&self) -> Option<()> {
        self.delegated?.visit_end()
    }
}