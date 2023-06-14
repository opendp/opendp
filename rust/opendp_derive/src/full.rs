use quote::ToTokens;
use syn::{parse_macro_input, AttributeArgs, Item, ItemFn};

use proc_macro::TokenStream;

use opendp_tooling::{
    bootstrap::{
        docstring::{get_proof_path, insert_proof_attribute},
        partial::generate_partial,
    },
    proven::{filesystem::load_proof_paths, Proven},
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

    // parse the inputs
    let attr_args = parse_macro_input!(attr_args as AttributeArgs);
    let mut item_fn = parse_macro_input!(input as ItemFn);
    let proof_paths = load_proof_paths().unwrap_or_default();

    // mutate the docstring to add a proof path
    if let Some(proof_path) = try_!(
        get_proof_path(&attr_args, &item_fn, &proof_paths),
        original_input
    ) {
        try_!(
            insert_proof_attribute(&mut item_fn.attrs, proof_path),
            original_input
        );
    }

    let function = try_!(
        Function::from_ast(attr_args, item_fn.clone(), None),
        original_input
    );

    let mut output = TokenStream::new();

    // insert cfg attributes
    (function.features.iter())
        .for_each(|feat| output.extend(TokenStream::from(quote::quote!(#[cfg(feature = #feat)]))));

    output.extend(TokenStream::from(item_fn.to_token_stream()));

    if cfg!(feature = "partials") {
        // write a partial function if the constructor adheres to the partial function pattern
        if let Some(then_fn) = generate_partial(item_fn.clone()) {
            output.extend(TokenStream::from(then_fn.to_token_stream()));
        };
    }
    output
}

// When proven runs, it inserts a link to a proof into the docstring, or throws an error if one cannot be found
pub(crate) fn proven(attr_args: TokenStream, input: TokenStream) -> TokenStream {
    let original_input = input.clone();

    let attr_args = parse_macro_input!(attr_args as AttributeArgs);
    let mut item = parse_macro_input!(input as Item);

    let proven = try_!(Proven::from_ast(attr_args, item.clone()), original_input);

    let proof_path = proven.proof_path.expect("unreachable");

    let attr_docs = match &mut item {
        Item::Fn(item_fn) => &mut item_fn.attrs,
        Item::Impl(item_impl) => &mut item_impl.attrs,
        _ => unreachable!(),
    };
    // mutate the docstring to add a proof path
    try_!(
        insert_proof_attribute(attr_docs, proof_path),
        original_input
    );

    let mut output = TokenStream::new();

    // insert cfg attributes
    (proven.features.0.iter())
        .for_each(|feat| output.extend(TokenStream::from(quote::quote!(#[cfg(feature = #feat)]))));

    output.extend(TokenStream::from(item.to_token_stream()));
    output
}
