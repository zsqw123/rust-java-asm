use proc_macro::TokenStream;

#[proc_macro_derive(AutoReadBytes)]
pub fn auto_read_bytes(input: TokenStream) -> TokenStream {
    input
}
