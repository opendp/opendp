use proc_macro::TokenStream;

#[cfg(feature = "full")]
mod full;

#[cfg(feature = "full")]
#[proc_macro_attribute]
pub fn bootstrap(attr_args: TokenStream, input: TokenStream) -> TokenStream {
    full::bootstrap(attr_args, input)
}

#[cfg(not(feature = "full"))]
#[proc_macro_attribute]
pub fn bootstrap(_attr_args: TokenStream, input: TokenStream) -> TokenStream {
    input
}

#[cfg(feature = "full")]
#[proc_macro_attribute]
pub fn proven(attr_args: TokenStream, input: TokenStream) -> TokenStream {
    full::proven(attr_args, input)
}

#[cfg(not(feature = "full"))]
#[proc_macro_attribute]
pub fn proven(_attr_args: TokenStream, input: TokenStream) -> TokenStream {
    input
}