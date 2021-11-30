
use std::iter::FromIterator;

use quote::{format_ident, quote};
use syn::{FnArg, GenericArgument, ItemFn, Pat, Path, PathArguments, PatIdent, PatType, Token, Type, TypePath, TypePtr, TypeTuple};
use syn::punctuated::Punctuated;

use crate::ffi::{extract, Function, Argument};
use crate::ffi::parse::path_to_str;
use std::collections::HashSet;
use proc_macro2::TokenStream;

pub(crate) fn gen_function(function: Function) -> ItemFn {
    let ffi_name = format_ident!("opendp_{}__{}_AUTO",
            function.module.join("_"),
            function.name);

    let args = Punctuated::<FnArg, Token![,]>::from_iter(
        function.arguments.iter().map(|a| FnArg::Typed(PatType {
            attrs: vec![],
            pat: Box::new(Pat::Ident(PatIdent {
                attrs: vec![],
                by_ref: None,
                mutability: None,
                ident: format_ident!("{}", a.name.as_ref().expect("arguments must have names")),
                subpat: None
            })),
            colon_token: Default::default(),
            ty: Box::new(rust_to_c_type(a.rust_type.clone()))
        })).chain(function.generics.iter().map(|g| FnArg::Typed(PatType {
            attrs: vec![],
            pat: Box::new(Pat::Ident(PatIdent {
                attrs: vec![],
                by_ref: None,
                mutability: None,
                ident: format_ident!("{}", g.name),
                subpat: None
            })),
            colon_token: Default::default(),
            ty: Box::new(syn::parse_str("*const std::os::raw::c_char").unwrap())
        }))));

    let ret_type = rust_to_c_type(function.ret.rust_type);

    let parse_types = function.generics.iter()
        .map(|g| format_ident!("{}", g.name))
        .fold(TokenStream::new(), |mut stream, name| {
            stream.extend(quote!(let #name = try_!(crate::ffi::util::Type::try_from(#name));));
            stream
        });

    // let generic_names = function.generics.iter()
    //     .map(|generic| generic.name).collect();

    // let parse_args = function.arguments.iter()
    //     .filter(|arg| )
    //     .cloned()
    //     .map(|arg| c_to_rust_expr(arg, &generic_names))
    //     .fold()

    syn::parse(quote!{
        #[no_mangle]
        extern "C" fn #ffi_name(#args) -> #ret_type {
            use std::convert::TryFrom;
            #parse_types
            unimplemented!()
        }
    }.into()).expect("failed to parse generated function")
}

fn rust_to_c_type(ty: Type) -> Type {
    match ty {
        Type::Path(TypePath {path, ..}) => {

            // new path
            match path_to_str(path.clone()) {
                i if i == "String" => syn::parse_str("crate::ffi::any::AnyObject"),
                i if i == "bool" => syn::parse_str("std::os::raw::c_bool"),
                i if i == "Transformation" => syn::parse_str("crate::ffi::any::AnyTransformation"),
                i if i == "Measurement" => syn::parse_str("crate::ffi::any::AnyMeasurement"),
                i if i == "Fallible" => {
                    let mut new_path: Path = syn::parse_str("crate::core::FfiResult").unwrap();
                    let mut last = extract!(path.segments.last().unwrap().arguments.clone(), PathArguments::AngleBracketed(v) => v);
                    last.args = Punctuated::from_iter(
                        last.args.iter().cloned()
                            .map(|arg| extract!(arg, GenericArgument::Type(v) => v))
                            .map(rust_to_c_type)
                            .map(|ty| Type::Ptr(TypePtr {
                                star_token: Default::default(),
                                const_token: None,
                                mutability: Some(Default::default()),
                                elem: Box::new(ty)
                            }))
                            .map(GenericArgument::Type));
                    new_path.segments.last_mut().unwrap().arguments = PathArguments::AngleBracketed(last);
                    Ok(Type::Path(TypePath { qself: None, path: new_path }))
                },
                i => panic!("unrecognized type: {}", i)
            }
        }
        Type::Tuple(_) => syn::parse_str("crate::ffi::any::AnyObject"),
        _ => panic!("unrecognized type structure")
    }.unwrap()
}

fn c_to_rust_expr(arg: Argument, generics: &HashSet<String>) -> Option<TokenStream> {
    if is_generic(arg.rust_type.clone(), &generics) {
        return None;
    }

    unimplemented!()

}

fn is_generic(ty: Type, generics: &HashSet<String>) -> bool {
    match ty {
        Type::Path(TypePath { path, .. }) => path.segments.into_iter()
            .any(|segment| generics.contains(&segment.ident.to_string())),
        Type::Tuple(TypeTuple { elems, .. }) => elems.into_iter()
            .any(|elem| is_generic(elem.clone(), generics)),
        _ => panic!("testing for genericity is only implemented for path and tuple types")
    }
}