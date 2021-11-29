use proc_macro::TokenStream;

use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{AttributeArgs, ItemFn, parse_macro_input, Type, Visibility, WhereClause, TypeParamBound, Token, NestedMeta};


mod parsing;
use crate::parsing::{parse_macro_config, parse_doc_comments, normalize_function};


macro_rules! extract {
    ($value:expr, $pattern:pat => $extracted_value:expr) => (match $value {
        $pattern => $extracted_value,
        _ => panic!(concat!(stringify!($value), " doesn't match ", stringify!($pattern))),
    })
}
pub(crate) use extract;
use syn::punctuated::Punctuated;
use std::collections::HashMap;

// metadata for each function in a module
struct Function {
    name: String,
    // location of this function in the module tree
    module: Vec<String>,
    // required feature flags to execute function
    features: Vec<String>,
    // plaintext description of the function used to generate documentation
    description: Vec<String>,

    // standard arguments
    arguments: Vec<Argument>,
    // type arguments
    generics: Vec<Generic>,
    // metadata for return type
    ret: Argument,

    // metadata for constructing new types based on existing types or introspection
    derived_types: Vec<Argument>,
    // syn tree for the function's where clause
    where_clause: Option<WhereClause>,
    // TODO: parse into representation
    dispatch: HashMap<String, String>
}

// Metadata for function arguments, derived types and returns.
pub(crate) struct Argument {
    // argument name. Optional for return types
    name: Option<String>,
    // // c type to translate to/from for FFI. Optional for derived types
    // c_type: Option<String>,
    // // RuntimeType expressed in terms of rust types with generics.
    // // Includes various RuntimeType constructors
    // old_rust_type: Option<RuntimeType>,
    // // a list of names in the rust_type that should be considered generics
    // old_generics: Vec<String>,
    rust_type: Type,
    // // type hint- a more abstract type that all potential arguments inherit from
    // hint: Option<String>,
    // plaintext description of the argument used to generate documentation
    description: Vec<String>,
    // // default value for the argument
    // default: Option<()>,
    // // set to true if the argument represents a type
    // is_type: bool,
    // most functions convert c_to_py or py_to_c. Set to true to leave the value as-is
    // an example usage is slice_as_object,
    // //  to prevent the returned AnyObject from getting converted back to python
    // do_not_convert: bool,
    // // when is_type, use this as an example to infer the type
    // example: Option<RuntimeType>
    meta: Vec<NestedMeta>
}

pub(crate) struct Generic {
    name: String,
    description: Vec<String>,
    bounds: Punctuated<TypeParamBound, Token![+]>,
    meta: Vec<NestedMeta>
}

// RuntimeType contains the metadata to generate code that evaluates to a rust type name
#[derive(Debug, PartialEq, Clone)]
enum RuntimeType {
    // reference an existing RuntimeType
    Name(String),
    // get the ith subtype of an existing RuntimeType
    Lower { root: Box<RuntimeType>, index: i32 },
    // build a higher level RuntimeType
    Raise { origin: String, args: Vec<Box<RuntimeType>> },
    // construct the RuntimeType via function call
    Function { function: String, params: Vec<Box<RuntimeType>> },
}

#[proc_macro_attribute]
pub fn generate_ffi(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_ = item.clone();

    let config = parse_macro_config(parse_macro_input!(attr as AttributeArgs));

    // Parse the function signature
    let ItemFn {
        attrs, vis, sig, ..
    } = parse_macro_input!(item as ItemFn);

    // assert that visibility must be public
    extract!(vis, Visibility::Public(_) => ());

    let doc_comments = parse_doc_comments(attrs);

    let function = normalize_function(sig, doc_comments, config);

    // Retrieve the base function name and construct the extern fn name
    let ffi_name = Ident::new(
        &*format!("opendp_{}__{}",
                  function.module.join("_"),
                  function.name),
        Span::call_site());

    // Construct the extern fn (in-progress)
    let ffi_func = quote!{
        extern "C" fn #ffi_name() {}
    };

    // current state of the generated function:
    println!("{}", ffi_func);

    // for now, just return the base function as-is, without adding the extern fn
    item_
}
