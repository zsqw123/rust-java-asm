use proc_macro::TokenStream;

use crate::read::jvms::auto_read_bytes;
use crate::write::jvms::auto_write_bytes;

mod read;
mod write;
pub(crate) mod alignment;

#[proc_macro_derive(ReadFrom, attributes(index, align))]
pub fn from_read_context_impl(input: TokenStream) -> TokenStream {
    auto_read_bytes(input)
}

#[proc_macro_derive(WriteInto, attributes(index, align))]
pub fn to_write_context_impl(input: TokenStream) -> TokenStream {
    auto_write_bytes(input)
}

