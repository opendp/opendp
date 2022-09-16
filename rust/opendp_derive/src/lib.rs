use darling::FromMeta;
use docstring::{path_to_str, DocComments};
use json::{Argument, Function, RuntimeType};
use parse::DerivedTypes;
use proc_macro::TokenStream;
use std::{
    collections::{HashMap, HashSet},
    env,
    path::PathBuf,
};
use syn::{
    parse_macro_input, AttributeArgs, FnArg, GenericParam, ItemFn, Pat, Signature,
    Type, TypeParam, Visibility,
};

use crate::{docstring::parse_doc_comments, parse::Bootstrap};

mod docstring;
mod json;
mod parse;

macro_rules! extract {
    ($value:expr, $pattern:pat => $extracted_value:expr) => {
        match $value {
            $pattern => $extracted_value,
            _ => panic!(concat!(
                stringify!($value),
                " doesn't match ",
                stringify!($pattern)
            )),
        }
    };
}
pub(crate) use extract;

#[proc_macro_attribute]
pub fn bootstrap(attr: TokenStream, input: TokenStream) -> TokenStream {
    let attr_args = parse_macro_input!(attr as AttributeArgs);
    let original_input = input.clone();

    let bootstrap = match Bootstrap::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }
    };

    // Parse the function signature
    let ItemFn {
        attrs, vis, sig, ..
    } = parse_macro_input!(input as ItemFn);

    // assert that visibility must be public
    extract!(vis, Visibility::Public(_) => ());

    let doc_comments = parse_doc_comments(attrs);

    let function = match cook(sig, bootstrap.clone(), doc_comments) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }
    };

    println!("function {}", serde_json::to_string_pretty(&function).unwrap());

    let mut ts = (bootstrap.proof)
        .map(|loc| {
            let comment = link_proof(loc);
            TokenStream::from(quote::quote!(#[doc = #comment]))
        })
        .unwrap_or_else(TokenStream::default);
    ts.extend(original_input);
    ts
}

fn link_proof(relative_path: String) -> String {
    let mut relative_path = PathBuf::from(relative_path);

    // construct absolute path
    let mut absolute_path = PathBuf::from(
        env::var_os("CARGO_MANIFEST_DIR").expect("Failed to determine location of Cargo.toml."),
    )
    .join("src")
    .join(relative_path.clone());

    // enable this when we actually have proofs
    // assert!(absolute_path.exists(), "{:?} does not exist!", absolute_path);

    let target = if cfg!(feature = "local") {
        // build path to local pdf
        absolute_path.set_extension("pdf");
        absolute_path.to_str().unwrap().to_string()
    } else {
        // build path to docs website
        relative_path.set_extension("pdf");
        format!(
            "https://docs.opendp.org/en/{version}/proofs/{relative_path}",
            version = get_version(),
            relative_path = relative_path.display()
        )
    };

    format!("[Link to proof.]({})", target)
}

fn get_version() -> String {
    let version = env::var("CARGO_PKG_VERSION").unwrap().to_string();
    if version == "0.0.0+development" {
        "latest".to_string()
    } else {
        version
    }
}

fn cook(
    signature: Signature,
    bootstrap: Bootstrap,
    mut doc_comments: DocComments,
) -> darling::Result<(String, Function)> {
    let all_generics = (signature.generics.params.iter())
        .map(|param| extract!(param, GenericParam::Type(v) => v))
        .map(|param| param.ident.to_string())
        .chain(
            bootstrap
                .derived_types
                .unwrap_or_else(|| DerivedTypes(HashMap::new()))
                .0
                .keys()
                .cloned(),
        )
        .collect::<HashSet<String>>();

    // let (ret_type, ret_generics) = syntype_to_runtimetype(
    //     extract!(signature.output, ReturnType::Type(_, v) => *v),
    //     &all_generics,
    // )?;

    let function = Function {
        args: signature
            .inputs
            .into_iter()
            .map(|v| extract!(v, FnArg::Typed(v) => v))
            .map(|v| {
                (
                    extract!(*v.pat, Pat::Ident(v) => v.ident.to_string()),
                    *v.ty,
                )
            })
            .map(|(name, ty)| {
                let boot_type = bootstrap.arguments.0.get(&name);
                let (rust_type, generics) = syntype_to_runtimetype(ty, &all_generics)?;
                Ok(Argument {
                    name: Some(name.clone()),
                    c_type: boot_type.as_ref().unwrap().c_type.clone(),
                    rust_type: Some(rust_type),
                    generics: generics.into_iter().collect(),
                    description: doc_comments.arguments.remove(&name).map(|dc| dc.join("\n")),
                    hint: boot_type.and_then(|bt| bt.hint.clone()),
                    default: boot_type.and_then(|bt| bt.default.clone().map(|def| def.0)),
                    is_type: false,
                    do_not_convert: boot_type.map(|bt| bt.do_not_convert).unwrap_or(false),
                    example: None,
                })
            })
            .chain(
                signature
                    .generics
                    .params
                    .into_iter()
                    .map(|v| extract!(v, GenericParam::Type(v) => v))
                    .map(|v: TypeParam| {
                        let name = v.ident.to_string();
                        let boot_type = bootstrap.generics.0.get(&name);
                        Ok(Argument {
                            name: Some(name.clone()),
                            c_type: boot_type.as_ref().unwrap().c_type.clone(),
                            description: doc_comments
                                .generics
                                .remove(&name)
                                .map(|dc| dc.join("\n")),
                            rust_type: None,
                            generics: Vec::new(),
                            hint: boot_type.and_then(|bt| bt.hint.clone()),
                            default: boot_type.and_then(|bt| bt.default.clone().map(|def| def.0)),
                            is_type: true,
                            do_not_convert: false,
                            example: boot_type.and_then(|bt| bt.example.clone()),
                        })
                    }),
            )
            .collect::<darling::Result<Vec<_>>>()?,
        ret: Argument {
            name: None,
            c_type: bootstrap.ret.as_ref().unwrap().c_type.clone(),
            rust_type: bootstrap.ret.clone().unwrap().rust_type,
            generics: Vec::new(),
            description: if doc_comments.ret.is_empty() {
                None
            } else {
                Some(doc_comments.ret.join("\n"))
            },
            hint: None,
            default: None,
            is_type: false,
            do_not_convert: bootstrap.ret.map(|ret| ret.do_not_convert).unwrap_or(false),
            example: None,
        },
        features: bootstrap.features.0,
        derived_types: Vec::new(),
        description: if doc_comments.description.is_empty() {
            None
        } else {
            Some(doc_comments.description.join("\n"))
        },
        proof: bootstrap.proof,
    };

    Ok((signature.ident.to_string(), function))
}

fn syntype_to_runtimetype(
    type_: Type,
    all_generics: &HashSet<String>,
) -> darling::Result<(RuntimeType, HashSet<String>)> {
    let mut found_generics = HashSet::new();
    let runtime_type = match type_ {
        Type::Path(tpath) => {
            let name = path_to_str(tpath.path)?;
            if all_generics.contains(&name) {
                found_generics.insert(name.clone());
            }
            RuntimeType::Name(name)
        }
        Type::Reference(refer) => {
            let (rtype, fg) = syntype_to_runtimetype(*refer.elem, &all_generics)?;
            found_generics.extend(fg);
            rtype
        }
        Type::Tuple(tuple) => {
            let (args, fgs) = (tuple.elems.into_iter())
                .map(|type_| syntype_to_runtimetype(type_, all_generics))
                .collect::<darling::Result<Vec<(_, _)>>>()?
                .into_iter()
                .unzip::<_, _, Vec<RuntimeType>, Vec<HashSet<String>>>();
            fgs.into_iter().for_each(|fg| found_generics.extend(fg));
            RuntimeType::Raise {
                origin: "Tuple".to_string(),
                args,
            }
        }
        _ => todo!(),
    };

    Ok((runtime_type, found_generics))
}

// fn rust_to_c_type(ty: Type) -> String {

// }
