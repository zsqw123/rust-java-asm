pub struct Attribute<'a> {
    attr_type: &'a str,
    content: &'a [i8],
    next_attr: Option<&'a Attribute<'a>>
}

