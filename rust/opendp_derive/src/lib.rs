#[cfg(feature="ffi")]
mod ffi;

use proc_macro::TokenStream;

#[cfg(not(feature="ffi"))]
#[proc_macro_attribute]
pub fn ffi(attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[cfg(feature="ffi")]
#[proc_macro_attribute]
pub fn ffi(attr: TokenStream, item: TokenStream) -> TokenStream {
    ffi::ffi_impl(attr, item)
}