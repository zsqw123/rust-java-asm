#[derive(Debug)]
pub struct Attribute {
    attr_type: String,
    content: Vec<u8>,
    next_attr: Option<Box<Attribute>>,
}

