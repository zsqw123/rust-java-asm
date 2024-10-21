use proc_macro::TokenStream;
use quote::quote;
use syn::__private::TokenStream2;
use syn::{parse_macro_input, AttributeArgs, Expr, Ident, ImplItem, ItemImpl, Meta, NestedMeta, Path, Type};

pub fn const_container_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    // read generic argument C from attr
    // #[const_container(DUShort)]
    let attr_input = parse_macro_input!(attr as AttributeArgs);
    let const_type_path_attr = get_type_path_from_attr(&attr_input);

    let item_impl = parse_macro_input!(item as ItemImpl);
    let impl_ident: &Type = &item_impl.self_ty;
    let const_items = read_const_items_from_impl_body(&item_impl, const_type_path_attr);
    let const_container_impl = generate_const_container_impl(
        impl_ident, const_type_path_attr, const_items,
    );

    let result = quote! {
        #item_impl
        #const_container_impl
    };
    result.into()
}

// #[const_container(DUShort)] -> DUShort
fn get_type_path_from_attr(attr: &AttributeArgs) -> &Path {
    let const_type_path_attr = attr.get(0)
        .expect("const_container attribute must have a type argument");
    if let NestedMeta::Meta(Meta::Path(path)) = const_type_path_attr {
        path
    } else {
        panic!("const_container attribute must have a type argument");
    }
}

/// read const items from impl body
///
/// ```
/// impl MapListTypeConst {
///     pub const TYPE_HEADER_ITEM: DUShort = 0x0000;
///     pub const TYPE_STRING_ID_ITEM: DUShort = 0x0001;
///     pub const TYPE_TYPE_ID_ITEM: DUShort = 0x0002;
///     pub const TYPE_PROTO_ID_ITEM: DUShort = 0x0003;
/// }
/// ```
///
/// returns list of pairs of (Const, Name)
fn read_const_items_from_impl_body(
    impl_body: &ItemImpl,
    expected_type: &Path,
) -> Vec<(Expr, Ident)> {
    impl_body.items.iter().filter_map(|item| {
        if let ImplItem::Const(const_item) = item {
            if let Type::Path(type_path) = &const_item.ty {
                if is_strict_same_type(&type_path.path, expected_type) {
                    return Some((const_item.expr.clone(), const_item.ident.clone()));
                }
            }
        }
        None
    }).collect()
}

fn generate_const_container_impl(
    struct_ident: &Type,
    const_type_path: &Path,
    const_items: Vec<(Expr, Ident)>,
) -> TokenStream2 {
    let const_name_match_arms = const_items.iter().map(|(expr, ident)| {
        let ident_str = ident.to_string().to_ascii_lowercase();
        quote! { #struct_ident::#ident => Some(#ident_str), }
    });

    let const_container_path = quote! { crate::ConstContainer };

    let const_container_impl = quote! {
        impl #const_container_path for #struct_ident {
            type ConstType = #const_type_path;
            fn const_name(c: Self::ConstType) -> Option<&'static str> {
                match c {
                    #(#const_name_match_arms)*
                    _ => None,
                }
            }
        }
    };

    const_container_impl
}

fn is_strict_same_type(
    current: &Path, another: &Path,
) -> bool {
    let current_segments = &current.segments;
    let another_segments = &another.segments;
    if current_segments.len() != another_segments.len() {
        return false;
    }
    let zip_iter = current_segments.iter().zip(another_segments.iter());
    for (current_segment, another_segment) in zip_iter {
        if current_segment.ident != another_segment.ident {
            return false;
        }
    }
    true
}

