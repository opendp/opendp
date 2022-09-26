use std::path::PathBuf;

use syn::{parse_macro_input, AttributeArgs, Item, ItemFn};

use proc_macro::TokenStream;

use opendp_tooling::{
    bootstrap::{arguments::Bootstrap, docstring::Docstring, reconcile_function},
    proof::{proven_get_proof_path, make_proof_link, bootstrap_get_proof_path},
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

// A procedural macro that gets run before the crate compiles and can read the source code.
// When bootstrap runs, it
// 1. simulates building bindings from foreign languages based on the docstring and function signature
// 2. inserts a link to a proof into the docstring, if there is one
#[proc_macro_attribute]
pub fn bootstrap(attr_args: TokenStream, input: TokenStream) -> TokenStream {
    let original_input = input.clone();

    // parse function
    let ItemFn { attrs, sig, .. } = parse_macro_input!(input as ItemFn);

    // attempt to parse docstring to detect formatting errors
    let docstrings = try_!(Docstring::from_attrs(attrs, &sig.output), original_input);

    // parse bootstrap arguments
    let attr_args = parse_macro_input!(attr_args as AttributeArgs);
    let mut bootstrap = try_!(Bootstrap::from_attribute_args(&attr_args), original_input);

    let func_name = sig.ident.to_string();
    if let None = &bootstrap.proof {
        bootstrap.proof = try_!(bootstrap_get_proof_path(&func_name), original_input);
    }
    
    try_!(
        reconcile_function(bootstrap.clone(), docstrings, sig),
        original_input
    );

    let mut output = TokenStream::new();

    let proof_link = try_!(
        (bootstrap.proof)
            .map(|rp| make_proof_link(PathBuf::from(rp)))
            .transpose(),
        original_input
    );

    // insert link to proof in documentation
    proof_link.map(|link| output.extend(TokenStream::from(quote::quote!(#[doc = #link]))));

    // insert cfg attributes
    (bootstrap.features.0.iter())
        .for_each(|feat| output.extend(TokenStream::from(quote::quote!(#[cfg(feature=#feat)]))));

    output.extend(original_input);
    output
}

// When proven runs, it inserts a link to a proof into the docstring, or throws an error if one cannot be found
#[proc_macro_attribute]
pub fn proven(attr_args: TokenStream, input: TokenStream) -> TokenStream {
    let original_input = input.clone();

    let attrs = parse_macro_input!(attr_args as AttributeArgs);
    let item = parse_macro_input!(input as Item);

    let proof_path = try_!(
        proven_get_proof_path(attrs, item),
        original_input
    );
    let proof_link = try_!(make_proof_link(PathBuf::from(proof_path)), original_input);

    let mut output = TokenStream::new();

    // insert link to proof in documentation
    output.extend(TokenStream::from(quote::quote!(#[doc = #proof_link])));
    output.extend(original_input);
    output
}
