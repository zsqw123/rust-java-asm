use syn::Attribute;

/// return: bytes count for alignment
pub fn find_alignment_in_attributes(attrs: &Vec<Attribute>) -> u16 {
    for attr in attrs {
        if !attr.path.is_ident("align") { continue }
        let Ok(alignment) = attr.parse_args::<syn::LitInt>()
            .and_then(|lit_int| lit_int.base10_parse::<u16>())
        else {
            panic!("`align` attribute must be a int. e.g. `#[align(4)]`")
        };
        return alignment;
    }
    0
}
