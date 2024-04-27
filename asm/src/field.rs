use crate::annotation::AnnotationVisitor;

#[derive(Default)]
pub struct FieldVisitor<'a> {
    pub delegated: Option<&'a FieldVisitor<'a>>,
}

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
    pub fn visit_annotation(&self, descriptor: &str, visible: bool) -> Option<&AnnotationVisitor> {
        self.delegated?.visit_annotation(descriptor, visible)
    }

    pub fn visit_attribute(&self) -> Option<()> {
        self.delegated?.visit_attribute()
    }

    pub fn visit_end(&self) -> Option<()> {
        self.delegated?.visit_end()
    }
}

impl<'a> FieldVisitor<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn from(origin: &'a FieldVisitor) -> Self {
        Self {
            delegated: Some(origin)
        }
    }
}
