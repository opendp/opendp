use std::path::PathBuf;

use syn::{parse_macro_input, AttributeArgs, Item, ItemFn};

use proc_macro::TokenStream;

use opendp_tooling::{
    proof::{load_proof_paths, make_proof_link, proven_get_proof_path},
    Function,
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

// The core of a procedural macro that gets run before the crate compiles and can read the source code.
// When bootstrap runs, it
// 1. simulates building bindings from foreign languages based on the docstring and function signature
// 2. inserts a link to a proof into the docstring, if there is one
pub(crate) fn bootstrap(attr_args: TokenStream, input: TokenStream) -> TokenStream {
    let original_input = input.clone();

    // prepare to build function
    let attr_args = parse_macro_input!(attr_args as AttributeArgs);
    let item_fn = parse_macro_input!(input as ItemFn);
    let proof_paths = load_proof_paths().unwrap_or_default();

    let function = try_!(
        Function::from_ast(attr_args, item_fn, &proof_paths),
        original_input
    );

    let mut output = TokenStream::new();

    let proof_link = try_!(
        (function.proof_path)
            .map(|rp| make_proof_link(PathBuf::from(rp)))
            .transpose(),
        original_input
    );

    // insert link to proof in documentation if it exists
    proof_link.map(|link| output.extend(TokenStream::from(quote::quote!(#[doc = #link]))));

    // insert cfg attributes
    (function.features.iter())
        .for_each(|feat| output.extend(TokenStream::from(quote::quote!(#[cfg(feature = #feat)]))));

    output.extend(original_input);
    output
}

// When proven runs, it inserts a link to a proof into the docstring, or throws an error if one cannot be found
pub(crate) fn proven(attr_args: TokenStream, input: TokenStream) -> TokenStream {
    let original_input = input.clone();

    let attrs = parse_macro_input!(attr_args as AttributeArgs);
    let item = parse_macro_input!(input as Item);

    let proof_path = try_!(proven_get_proof_path(attrs, item), original_input);
    let proof_link = try_!(make_proof_link(PathBuf::from(proof_path)), original_input);

    let mut output = TokenStream::new();

    // insert link to proof in documentation
    output.extend(TokenStream::from(quote::quote!(#[doc = #proof_link])));
    output.extend(original_input);
    output
}
