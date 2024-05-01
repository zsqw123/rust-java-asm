use crate::{asm_visitor, asm_visitor_impl};
use crate::annotation::AnnotationVisitor;
use crate::attribute::Attribute;
use crate::field::{FieldValue, FieldVisitor};
use crate::method::MethodVisitor;
use crate::type_path::TypePath;

asm_visitor! {
    pub struct ClassVisitor<'a>
}

asm_visitor_impl! {
    impl ClassVisitor<'_> {
        /// Visits the header of the class.
        ///
        /// @param version the class version. The minor version is stored in the 16 most significant bits,
        ///     and the major version in the 16 least significant bits.
        /// @param access the class's access flags (see {@link Opcodes}). This parameter also indicates if
        ///     the class is deprecated {@link Opcodes#ACC_DEPRECATED} or a record {@link
        ///     Opcodes#ACC_RECORD}.
        /// @param name the internal name of the class (see {@link Type#get_internal_name()}).
        /// @param signature the signature of this class. May be [None] if the class is not a
        ///     generic one, and does not extend or implement generic classes or interfaces.
        /// @param super_name the internal of name of the super class (see {@link Type#get_internal_name()}).
        ///     For interfaces, the super class is {@link Object}. May be [None], but only for the
        ///     {@link Object} class.
        /// - interfaces the internal names of the class's interfaces (see {@link
        ///     Type#get_internal_name()}). May be [None].
        pub fn visit(
            &self, version: u32, access: u16, name: &str, signature: &str,
            super_name: &str, interfaces: &[&str],
        ) -> Option<()>;
        
        /// Visits the source of the class.
        ///
        /// @param source the name of the source file from which the class was compiled. May be {@literal
        ///     null}.
        /// @param debug additional debug information to compute the correspondence between source and
        ///     compiled elements of the class. May be [None].
        pub fn visit_source(
            &self, source: &str, debug: &str,
        ) -> Option<()>;
        
        /// Visit the module corresponding to the class.
        ///
        /// @param name the fully qualified name (using dots) of the module.
        /// @param access the module access flags, among {@code ACC_OPEN}, {@code ACC_SYNTHETIC} and {@code
        ///     ACC_MANDATED}.
        /// @param version the module version, or [None].
        ///
        /// returns a visitor to visit the module values, or [None] if this visitor is not
        ///     interested in visiting this module.
        pub fn visit_module(
            &self, access: u16, version: &str,
        ) -> Option<&ModuleVisitor>;
        
        /// Visits the nest host class of the class. A nest is a set of classes of the same package that
        /// share access to their private members. One of these classes, called the host, lists the other
        /// members of the nest, which in turn should link to the host of their nest. This method must be
        /// called only once and only if the visited class is a non-host member of a nest. A class is
        /// implicitly its own nest, so it's invalid to call this method with the visited class name as
        /// argument.
        ///
        /// @param nest_host the internal name of the host class of the nest (see {@link
        ///     Type#get_internal_name()}).
        pub fn visit_nest_host(&self, nest_host: &str) -> Option<()>;
        
        /// Visits the enclosing class of the class. This method must be called only if this class is a
        /// local or anonymous class. See the JVMS 4.7.7 section for more details.
        ///
        /// @param owner internal name of the enclosing class of the class (see {@link
        ///     Type#get_internal_name()}).
        /// @param name the name of the method that contains the class, or [None] if the class is
        ///     not enclosed in a method or constructor of its enclosing class (e.g. if it is enclosed in
        ///     an instance initializer, static initializer, instance variable initializer, or class
        ///     variable initializer).
        /// @param descriptor the descriptor of the method that contains the class, or [None] if
        ///     the class is not enclosed in a method or constructor of its enclosing class (e.g. if it is
        ///     enclosed in an instance initializer, static initializer, instance variable initializer, or
        ///     class variable initializer).
        pub fn visit_outer_class(&self, owner: &str, name: &str, descriptor: &str) -> Option<()>;
        
        /// Visits an annotation of the class.
        ///
        /// @param descriptor the class descriptor of the annotation class.
        /// @param visible {@literal true} if the annotation is visible at runtime.
        ///
        /// returns a visitor to visit the annotation values, or [None] if this visitor is not
        ///     interested in visiting this annotation.
        pub fn visit_annotation(&self, descriptor: &str, visible: bool) -> Option<&AnnotationVisitor>;
        
        /// Visits an annotation on a type in the class signature.
        ///
        /// @param type_ref a reference to the annotated type. The sort of this type reference must be
        ///     {@link TypeReference#CLASS_TYPE_PARAMETER}, {@link
        ///     TypeReference#CLASS_TYPE_PARAMETER_BOUND} or {@link TypeReference#CLASS_EXTENDS}. See
        ///     {@link TypeReference}.
        /// @param type_path the path to the annotated type argument, wildcard bound, array element type, or
        ///     static inner type within 'typeRef'. May be [None] if the annotation targets
        ///     'typeRef' as a whole.
        /// @param descriptor the class descriptor of the annotation class.
        /// @param visible {@literal true} if the annotation is visible at runtime.
        ///
        /// returns a visitor to visit the annotation values, or [None] if this visitor is not
        ///     interested in visiting this annotation.
        pub fn visit_type_annotation(&self, type_ref: i32, type_path: &TypePath, descriptor: &str, visible: bool) -> Option<&AnnotationVisitor>;
        
        /// Visits a non standard attribute of the class.
        ///
        /// @param attribute an attribute.
        pub fn visit_attribute(&self, attribute: &Attribute) -> Option<()>;
        
        /// Visits a member of the nest. A nest is a set of classes of the same package that share access
        /// to their private members. One of these classes, called the host, lists the other members of the
        /// nest, which in turn should link to the host of their nest. This method must be called only if
        /// the visited class is the host of a nest. A nest host is implicitly a member of its own nest, so
        /// it's invalid to call this method with the visited class name as argument.
        ///
        /// @param nest_member the internal name of a nest member (see {@link Type#get_internal_name()}).
        pub fn visit_nest_member(&self, nest_member: &str) -> Option<()>;
        
        /// Visits a permitted subclasses. A permitted subclass is one of the allowed subclasses of the
        /// current class.
        ///
        /// @param permitted_subclass the internal name of a permitted subclass (see {@link
        ///     Type#get_internal_name()}).
        pub fn visit_permitted_subclass(&self, permitted_subclass: &str) -> Option<()>;
        
        /// Visits information about an inner class. This inner class is not necessarily a member of the
        /// class being visited. More precisely, every class or interface C which is referenced by this
        /// class and which is not a package member must be visited with this method. This class must
        /// reference its nested class or interface members, and its enclosing class, if any. See the JVMS
        /// 4.7.6 section for more details.
        ///
        /// @param name the internal name of C (see {@link Type#get_internal_name()}).
        /// @param outerName the internal name of the class or interface C is a member of (see {@link
        ///     Type#get_internal_name()}). Must be [None] if C is not the member of a class or
        ///     interface (e.g. for local or anonymous classes).
        /// @param innerName the (simple) name of C. Must be [None] for anonymous inner classes.
        /// @param access the access flags of C originally declared in the source code from which this
        ///     class was compiled.
        pub fn visit_inner_class(&self, name: &str, outer_name: &str, inner_name: &str, access: u16) -> Option<()>;
        
        /// Visits a record component of the class.
        ///
        /// @param name the record component name.
        /// @param descriptor the record component descriptor (see {@link Type}).
        /// @param signature the record component signature. May be [None] if the record component
        ///     type does not use generic types.
        ///
        /// returns a visitor to visit this record component annotations and attributes, or [None]
        ///     if this class visitor is not interested in visiting these annotations and attributes.
        pub fn visit_record_component(&self, name: &str, descriptor: &str, signature: &str) -> Option<&RecordComponentVisitor>;
        
        /// Visits a field of the class.
        ///
        /// @param access the field's access flags (see {@link Opcodes}). This parameter also indicates if
        ///     the field is synthetic and/or deprecated.
        /// @param name the field's name.
        /// @param descriptor the field's descriptor (see {@link Type}).
        /// @param signature the field's signature. May be [None] if the field's type does not use
        ///     generic types.
        /// @param value the field's initial value. This parameter, which may be [None] if the
        ///     field does not have an initial value, must be an {@link Integer}, a {@link Float}, a {@link
        ///     Long}, a {@link Double} or a {@link String} (for {@code int}, {@code float}, {@code long}
        ///     or {@code String} fields respectively). <i>This parameter is only used for static
        ///     fields</i>. Its value is ignored for non static fields, which must be initialized through
        ///     bytecode instructions in constructors or methods.
        /// 
        /// returns a visitor to visit field annotations and attributes, or [None] if this class
        ///     visitor is not interested in visiting these annotations and attributes.
        pub fn visit_field(&self, name: &str, descriptor: &str, signature: &str, value: &FieldValue) -> Option<&FieldVisitor>;
        
        /// Visits a method of the class. This method <i>must</i> return a new {@link MethodVisitor}
        /// instance (or [None]) each time it is called, i.e., it should not return a previously
        /// returned visitor.
        /// 
        /// @param access the method's access flags (see {@link Opcodes}). This parameter also indicates if
        ///     the method is synthetic and/or deprecated.
        /// @param name the method's name.
        /// @param descriptor the method's descriptor (see {@link Type}).
        /// @param signature the method's signature. May be [None] if the method parameters,
        ///     return type and exceptions do not use generic types.
        /// @param exceptions the internal names of the method's exception classes (see {@link
        ///     Type#get_internal_name()}). May be [None].
        /// 
        /// returns an object to visit the byte code of the method, or [None] if this class
        ///     visitor is not interested in visiting the code of this method.
        pub fn visit_method(&self, name: &str, descriptor: &str, signature: &str, exceptions: &[&str]) -> Option<&MethodVisitor>;
        
        /// Visits the end of the class. This method, which is the last one to be called, is used to inform
        /// the visitor that all the fields and methods of the class have been visited.
        pub fn visit_end(&self) -> Option<()>;
    }
}

asm_visitor! {
    pub struct ModuleVisitor<'a>
}

asm_visitor! {
    pub struct RecordComponentVisitor<'a>
}
