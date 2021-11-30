use std::iter::FromIterator;

use quote::{format_ident, quote};
use syn::{FnArg, GenericArgument, ItemFn, Pat, Path, PathArguments, PatIdent, PatType, Token, Type, TypePath, TypePtr};
use syn::punctuated::Punctuated;

use crate::{extract, Function};
use crate::parse::path_to_str;

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

    syn::parse(quote!{
        #[no_mangle]
        extern "C" fn #ffi_name(#args) -> #ret_type {
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

fn c_to_rust_expr() {

}