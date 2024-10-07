use crate::constant::const_container_impl;
use crate::read::jvms::auto_read_bytes;
use crate::write::jvms::auto_write_bytes;
use proc_macro::TokenStream;

mod read;
mod write;
pub(crate) mod alignment;
mod constant;

#[proc_macro_derive(ReadFrom, attributes(index, align))]
pub fn from_read_context_impl(input: TokenStream) -> TokenStream {
    auto_read_bytes(input)
}

#[proc_macro_derive(WriteInto, attributes(index, align))]
pub fn to_write_context_impl(input: TokenStream) -> TokenStream {
    auto_write_bytes(input)
}

#[proc_macro_attribute]
pub fn const_container(attr: TokenStream, item: TokenStream) -> TokenStream {
    const_container_impl(attr, item)
}
