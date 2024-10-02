use proc_macro::TokenStream;

use quote::{format_ident, quote, quote_spanned};
use syn::{Attribute, Data, DataStruct, DeriveInput, Field, Fields, Ident, parse_macro_input};
use syn::__private::TokenStream2;
use syn::spanned::Spanned;

pub(crate) fn auto_read_bytes(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let name = derive_input.ident;

    let Data::Struct(data) = derive_input.data else {
        unimplemented!("unsupported data type for auto read bytes, only struct is supported.")
    };
    let read_all_data = read_fields(&data.fields);
    let all_field_names = all_field_names(&data);
    let build_item = match data.fields {
        Fields::Named(_) => quote! { Ok(Self { #all_field_names }) },
        Fields::Unnamed(_) => quote! { Ok(Self(#all_field_names)) },
        Fields::Unit => quote! { Ok(Self) }
    };

    let from_read_context_path = quote! { crate::impls::jvms::r::ReadFrom };
    let read_context_path = quote! { crate::impls::jvms::r::ReadContext };
    let asm_result_path = quote! { crate::err::AsmResult };

    let generated = quote! {
        impl #from_read_context_path for #name {
            #[inline]
            fn read_from(context: &mut #read_context_path) -> #asm_result_path<#name> {
                #read_all_data
                #build_item
            }
        }
    };
    TokenStream::from(generated)
}

fn read_fields(fields: &Fields) -> TokenStream2 {
    match fields {
        Fields::Named(fields) => {
            let fields = &fields.named;
            let read_field = fields.iter().map(|field| {
                let Some(ident) = &field.ident else {
                    panic!("field must have a name for decode struct")
                };
                build_read_bytes_for_field(field, ident)
            });
            quote! { #(#read_field)* }
        }
        Fields::Unnamed(fields) => {
            let fields = &fields.unnamed;
            let read_field = fields.iter().enumerate().map(|(index, field)| {
                let ident = format_ident!("field_{}", span = field.span(), index);
                build_read_bytes_for_field(field, &ident)
            });
            quote! { #(#read_field)* }
        }
        Fields::Unit => quote!()
    }
}

fn build_read_bytes_for_field(field: &Field, ident: &Ident) -> TokenStream2 {
    let ty = &field.ty;
    let field_name = find_index_field_name(&field.attrs);
    match field_name {
        Some(field_name) => quote_spanned! { field.span() =>
            let #ident = context.read_vec(#field_name as usize)?;
        },
        None => quote_spanned! { field.span() =>
            let #ident = context.read::<#ty>()?;
        },
    }
}

fn all_field_names(data: &DataStruct) -> TokenStream2 {
    match &data.fields {
        Fields::Named(fields) => {
            let fields = &fields.named;
            let field_names = fields.iter().map(|field| {
                let ident = &field.ident;
                quote_spanned! { field.span() => #ident, }
            });
            quote! { #(#field_names)* }
        }
        Fields::Unnamed(fields) => {
            let fields = &fields.unnamed;
            let field_names = fields.iter().enumerate().map(|(index, field)| {
                let ident = format_ident!("field_{}", span = field.span(), index);
                quote_spanned! { field.span() => #ident, }
            });
            quote! { #(#field_names)* }
        }
        Fields::Unit => quote!()
    }
}

/// find the index field name in `attrs`
fn find_index_field_name(attrs: &Vec<Attribute>) -> Option<Ident> {
    for attr in attrs {
        if !attr.path.is_ident("index") { continue }
        let Ok(ident) = attr.parse_args::<Ident>() else {
            panic!("`index_for` attribute must have a field name as argument. Current attribute tokens: {}", attr.tokens)
        };
        return Some(ident);
    }
    None
}
