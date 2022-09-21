use std::path::PathBuf;

use syn::{parse_macro_input, AttributeArgs, ItemFn};

use proc_macro::TokenStream;

use opendp_bootstrap::parse::{
    bootstrap::Bootstrap,
    docstring::{find_relative_proof_path, make_proof_link, Docstring},
};

macro_rules! try_ {
    ($expr:expr, $original:expr) => {
        match $expr {
            Ok(v) => v,
            Err(e) => {
                return TokenStream::from_iter(
                    TokenStream::from(e.write_errors())
                        .into_iter()
                        .chain($original),
                )
            }
        }
    };
}

#[proc_macro_attribute]
pub fn bootstrap(attr_args: TokenStream, input: TokenStream) -> TokenStream {
    let original_input = input.clone();

    // parse function
    let ItemFn { attrs, sig, .. } = parse_macro_input!(input as ItemFn);

    // attempt to parse docstring to detect formatting errors
    try_!(Docstring::from_attrs(attrs), original_input);

    // parse bootstrap arguments
    let attr_args = parse_macro_input!(attr_args as AttributeArgs);
    let mut bootstrap = try_!(Bootstrap::from_attribute_args(&attr_args), original_input);

    if let None = bootstrap.proof {
        bootstrap.proof = find_relative_proof_path(&sig.ident.to_string());
    }

    let mut output = TokenStream::new();

    // insert cfg attributes
    (bootstrap.features.0.iter())
        .for_each(|feat| output.extend(TokenStream::from(quote::quote!(#[cfg(feature=#feat)]))));

    // insert link to proof in documentation
    (bootstrap.proof)
        .map(|rp| make_proof_link(PathBuf::from(rp)))
        .map(|link| output.extend(TokenStream::from(quote::quote!(#[doc = #link]))));

    output.extend(original_input);
    output
}
