use proc_macro::TokenStream;

use quote::{format_ident, quote, quote_spanned};
use syn::{Data, DataEnum, DataStruct, DeriveInput, Fields, Ident, parse_macro_input};
use syn::__private::TokenStream2;
use syn::spanned::Spanned;

pub(crate) fn auto_write_bytes(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let data = input.data;

    let calls = match data {
        Data::Struct(data) => struct_write_bytes(data),
        Data::Enum(data) => enum_write_bytes(name, data),
        _ => {
            unimplemented!("{} is unsupported data type for auto write bytes, only struct and enum are supported.", name)
        }
    };

    let into_write_context_path = quote! { java_asm_internal::write::jvms::IntoWriteContext };
    let write_context_path = quote! { java_asm_internal::write::jvms::WriteContext };

    let generated = quote! {
        impl #into_write_context_path<#name> for #name {
            fn into_context(context: &mut #write_context_path, into: #name) {
                #calls
            }
        }
    };

    TokenStream::from(generated)
}

fn struct_write_bytes(data: DataStruct) -> TokenStream2 {
    let fields = &data.fields;
    match fields {
        Fields::Named(fields) => {
            let fields = &fields.named;
            let write_field = fields.iter().map(|field| {
                let ident = &field.ident;
                quote_spanned! { field.span() =>
                    context.push(into.#ident);
                }
            });
            quote! { #(#write_field)* }
        }
        Fields::Unnamed(fields) => {
            let fields = &fields.unnamed;
            let write_field = fields.iter().enumerate().map(|(index, field)| {
                quote_spanned! { field.span() =>
                    context.push(into.#index);
                }
            });
            quote! { #(#write_field)* }
        }
        Fields::Unit => quote!()
    }
}

fn enum_write_bytes(enum_name: &Ident, data: DataEnum) -> TokenStream2 {
    let variants = &data.variants;
    let write_variants = variants.iter().map(|variant| {
        let ident = &variant.ident;
        let fields = &variant.fields;
        let size = fields.len();
        let token_stream = match fields {
            Fields::Named(fields) => {
                let fields = &fields.named;
                let mut field_names = Vec::with_capacity(size);
                let mut push_calls = Vec::with_capacity(size);
                for field in fields {
                    let Some(field_ident) = &field.ident else {
                        panic!("named value without identifier")
                    };
                    field_names.push(field_ident);
                    push_calls.push(quote_spanned! { field.span() =>
                        context.push(#field_ident);
                    });
                }
                quote! {
                    #enum_name::#ident{#(#field_names),*} => {
                        #(#push_calls)*
                    },
                }
            }
            Fields::Unnamed(fields) => {
                let fields = &fields.unnamed;
                let field_names: Vec<Ident> = (0..size).map(|index| {
                    format_ident!("field_{}", index)
                }).collect();
                let mut push_calls = Vec::with_capacity(size);

                for index in 0..size {
                    let field = &fields[index];
                    let field_ident = &field_names[index];
                    push_calls.push(quote_spanned! { field.span() =>
                        context.push(#field_ident);
                    });
                }
                quote! {
                    #enum_name::#ident(#(#field_names),*) => {
                        #(#push_calls)*
                    },
                }
            }
            Fields::Unit => quote! {
                #enum_name::#ident => {},
            },
        };
        token_stream
    });
    quote! {
        match into {
            #(#write_variants)*
        }
    }
}
