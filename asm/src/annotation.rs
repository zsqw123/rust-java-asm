use crate::{asm_visitor, asm_visitor_impl};

asm_visitor! {
    pub struct AnnotationVisitor<'a>
}

pub enum AnnotationValue {}

asm_visitor_impl! {
    impl AnnotationVisitor<'_> {
        /// Visits a primitive value of the annotation.
        ///
        /// - name, the value name.
        /// - value, the actual value, whose type must be {@link Byte}, {@link Boolean}, {@link
        ///     Character}, {@link Short}, {@link Integer} , {@link Long}, {@link Float}, {@link Double},
        ///     {@link String} or {@link Type} of {@link Type#OBJECT} or {@link Type#ARRAY} sort. This
        ///     value can also be an array of byte, boolean, short, char, int, long, float or double values
        ///     (this is equivalent to using {@link #visit_array} and visiting each array element in turn,
        ///     but is more convenient).
        fn visit(&self, name: &str, value: &AnnotationValue) -> Option<()> ;

        /// Visits an enumeration value of the annotation.
        ///
        /// - name, the value name.
        /// - descriptor, the class descriptor of the enumeration class.
        /// - value, the actual enumeration value.
        fn visit_enum(&self, name: &str, descriptor: &str, value: &AnnotationValue) -> Option<()>;

        /// Visits a nested annotation value of the annotation.
        ///
        /// - name, the value name.
        /// - descriptor, the class descriptor of the nested annotation class.
        ///
        /// returns a visitor to visit the actual nested annotation value, or [None] if this
        ///     visitor is not interested in visiting this nested annotation. <i>The nested annotation
        ///     value must be fully visited before calling other methods on this annotation visitor</i>.
        fn visit_annotation(&self, name: &str, descriptor: &str) -> Option<&Self> ;

        /// Visits an array value of the annotation. Note that arrays of primitive values (such as byte,
        /// boolean, short, char, int, long, float or double) can be passed as value to {@link #visit
        /// visit}. This is what {@link ClassReader} does for non empty arrays of primitive values.
        ///
        /// - name, the value name.
        /// returns a visitor to visit the actual array value elements, or [None] if this visitor
        ///     is not interested in visiting these values. The 'name' parameters passed to the methods of
        ///     this visitor are ignored. <i>All the array values must be visited before calling other
        ///     methods on this annotation visitor</i>.
        fn visit_array(&self, name: &str) -> Option<&Self> ;

        pub fn visit_end(&self) -> Option<()> ;
    }
}
