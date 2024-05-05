use crate::asm_type::Type;

#[derive(Clone, Debug)]
pub enum AnnotationValue {
    Byte(u8),
    Boolean(bool),
    Char(char),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(String),
    Class(Type),
    Enum(String, String),
    Annotation(Box<AnnotationValue>),
    Array(Vec<AnnotationValue>),
}