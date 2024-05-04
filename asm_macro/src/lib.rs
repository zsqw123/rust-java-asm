use proc_macro::TokenStream;

use quote::{format_ident, quote, quote_spanned};
use syn::{Data, DataStruct, DeriveInput, Fields, parse_macro_input};
use syn::__private::TokenStream2;
use syn::spanned::Spanned;

#[proc_macro_derive(FromReadContext)]
pub fn auto_read_bytes(input: TokenStream) -> TokenStream {
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
    
    let from_read_context_path = quote! { java_asm_internal::read::jvms::FromReadContext };
    let read_context_path = quote! { java_asm_internal::read::jvms::ReadContext };
    let asm_result_path = quote! { java_asm_internal::err::AsmResult };
    
    let generated = quote! {
        impl #from_read_context_path<#name> for #name {
            fn from_context(context: &mut #read_context_path) -> #asm_result_path<#name> {
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
                let ident = &field.ident;
                let ty = &field.ty;
                quote_spanned! { field.span() =>
                    let #ident = context.read::<#ty>()?;
                }
            });
            quote! { #(#read_field)* }
        }
        Fields::Unnamed(fields) => {
            let fields = &fields.unnamed;
            let read_field = fields.iter().enumerate().map(|(index, field)| {
                let ident = format_ident!("field_{}", span = field.span(), index);
                let ty = &field.ty;
                quote_spanned! { field.span() =>
                    let #ident = context.read::<#ty>()?;
                }
            });
            quote! { #(#read_field)* }
        }
        Fields::Unit => quote!()
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

