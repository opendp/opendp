use darling::FromMeta;
use docstring::{path_to_str, DocComments};
use parse::{DerivedTypes, NewRuntimeType};
use proc_macro::TokenStream;
use std::{
    collections::{HashMap, HashSet},
    env,
    path::PathBuf
};
use syn::{
    parse_macro_input, AttributeArgs, FnArg, GenericArgument, GenericParam, ItemFn, Pat,
    PathArguments, ReturnType, Signature, Type, TypeParam, TypePath, Visibility,
};

use opendp_pre_derive::{Argument, Function, RuntimeType, target::find_target_dir};

use crate::{docstring::parse_doc_comments, parse::Bootstrap};

mod docstring;
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

    let (module, name, function) = match make_bootstrap_json(sig, bootstrap.clone(), doc_comments) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }
    };

    if cfg!(feature = "bootstrap-json") {
        let json_str = serde_json::to_string_pretty(&function).expect("failed to serialize function to json");
        println!("{module}::{name}({json_str})");

        let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR must be set."));
        let target_dir = find_target_dir(&out_dir);
        let json_module_dir = target_dir.join("opendp_bootstrap").join(module.clone());

        dbg!(&json_module_dir);

        std::fs::create_dir_all(&json_module_dir)
            .expect(format!("unable to create folder {{target_dir}}/opendp_bootstrap/{module}").as_str());

        let filename = format!("{}.json", name);
        let json_path = json_module_dir.join(filename.clone());
        std::fs::write(&json_path, json_str)
            .expect(format!("unable to write file {{target_dir}}/opendp_bootstrap/{module}/{filename}").as_str());
    }

    let mut ts = (function.proof)
        .map(make_proof_link)
        .map(|link| TokenStream::from(quote::quote!(#[doc = #link])))
        .unwrap_or_else(TokenStream::default);

    ts.extend(original_input);
    ts
}

fn make_proof_link(relative_path: String) -> String {
    let mut relative_path = PathBuf::from(relative_path);

    // construct absolute path
    let mut absolute_path = PathBuf::from(
        env::var_os("CARGO_MANIFEST_DIR").expect("Failed to determine location of Cargo.toml.")
    )
    .join("src")
    .join(relative_path.clone());

    // enable this when we actually have proofs
    // assert!(absolute_path.exists(), "{:?} does not exist!", absolute_path);

    let target = if cfg!(feature = "local") {
        // build path to local pdf
        absolute_path.set_extension("pdf");
        absolute_path
            .to_str()
            .expect("failed to retrieve str")
            .to_string()
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
    let version = env::var("CARGO_PKG_VERSION")
        .expect("CARGO_PKG_VERSION must be set")
        .to_string();
    if version == "0.0.0+development" {
        "latest".to_string()
    } else {
        version
    }
}

fn make_bootstrap_json(
    signature: Signature,
    bootstrap: Bootstrap,
    mut doc_comments: DocComments,
) -> darling::Result<(String, String, Function)> {
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

    let function = Function {
        features: bootstrap.features.0,
        description: if doc_comments.description.is_empty() {
            None
        } else {
            Some(doc_comments.description.join("\n").trim().to_string())
        },
        proof: bootstrap.proof,
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
                // if rust type is given, use it. Otherwise parse the rust type on the function
                let (rust_type, generics) = match boot_type.and_then(|bt| bt.rust_type.clone()) {
                    Some(v) => (
                        v,
                        boot_type
                            .and_then(|bt| bt.generics.clone())
                            .map(|gen| gen.0)
                            .unwrap_or_else(HashSet::new),
                    ),
                    None => syntype_to_runtimetype(ty.clone(), &all_generics)?,
                };
                Ok(Argument {
                    name: Some(name.clone()),
                    c_type: Some(match boot_type.as_ref().and_then(|bt| bt.c_type.as_ref()) {
                        Some(ref v) => v.to_string(),
                        None => rust_to_c_type(ty)?,
                    }),
                    rust_type: Some(rust_type.0),
                    generics: generics.into_iter().collect(),
                    description: doc_comments
                        .arguments
                        .remove(&name)
                        .map(|dc| dc.join("\n").trim().to_string()),
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
                            c_type: None,
                            description: doc_comments
                                .generics
                                .remove(&name)
                                .map(|dc| dc.join("\n").trim().to_string()),
                            rust_type: None,
                            generics: Vec::new(),
                            hint: boot_type.and_then(|bt| bt.hint.clone()),
                            default: boot_type.and_then(|bt| bt.default.clone().map(|def| def.0)),
                            is_type: true,
                            do_not_convert: false,
                            example: boot_type.and_then(|bt| bt.example.clone()).map(|bt| bt.0),
                        })
                    }),
            )
            .collect::<darling::Result<Vec<_>>>()?,
        ret: Argument {
            name: None,
            c_type: Some(
                match bootstrap.ret.as_ref().and_then(|bt| bt.c_type.as_ref()) {
                    Some(ref v) => v.to_string(),
                    None => {
                        rust_to_c_type(extract!(signature.output, ReturnType::Type(_, ty) => *ty))?
                    }
                },
            ),
            rust_type: bootstrap.ret.as_ref().and_then(|bs| bs.rust_type.clone()).map(|bt| bt.0),
            generics: Vec::new(),
            description: if doc_comments.ret.is_empty() {
                None
            } else {
                Some(doc_comments.ret.join("\n").trim().to_string())
            },
            hint: None,
            default: None,
            is_type: false,
            do_not_convert: bootstrap.ret.map(|ret| ret.do_not_convert).unwrap_or(false),
            example: None,
        },
        derived_types: Vec::new(),
    };

    Ok((bootstrap.module, signature.ident.to_string(), function))
}

fn syntype_to_runtimetype(
    type_: Type,
    all_generics: &HashSet<String>,
) -> darling::Result<(NewRuntimeType, HashSet<String>)> {
    let mut found_generics = HashSet::new();
    let runtime_type = match type_ {
        Type::Path(tpath) => {
            let name = path_to_str(tpath.path)?;
            if all_generics.contains(&name) {
                found_generics.insert(name.clone());
            }
            RuntimeType::Name(name).into()
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
                .map(|(rt, gen)| (rt.0, gen))
                .unzip::<_, _, Vec<RuntimeType>, Vec<HashSet<String>>>();
            fgs.into_iter().for_each(|fg| found_generics.extend(fg));
            RuntimeType::Raise {
                origin: "Tuple".to_string(),
                args,
            }.into()
        }
        _ => todo!(),
    };

    Ok((runtime_type, found_generics))
}

fn rust_to_c_type(ty: Type) -> darling::Result<String> {
    Ok(match ty {
        Type::Path(TypePath { path, .. }) => {
            let segment = path
                .segments
                .last()
                .ok_or_else(|| darling::Error::custom("at least one segment required"))?;

            match segment.ident.to_string() {
                i if i == "String" => "AnyObject *".to_string(),
                i if i == "Vec" => "AnyObject *".to_string(),
                i if i == "bool" => "bool".to_string(),
                i if i == "i8" => "int8_t".to_string(),
                i if i == "i16" => "int16_t".to_string(),
                i if i == "i32" => "int32_t".to_string(),
                i if i == "i64" => "int64_t".to_string(),
                i if i == "u8" => "uint8_t".to_string(),
                i if i == "u16" => "uint16_t".to_string(),
                i if i == "u32" => "uint32_t".to_string(),
                i if i == "u64" => "uint64_t".to_string(),
                i if i == "f32" => "float".to_string(),
                i if i == "f64" => "double".to_string(),
                i if i == "usize" => "size_t".to_string(),
                i if i == "Transformation" => "AnyTransformation *".to_string(),
                i if i == "Measurement" => "AnyMeasurement *".to_string(),
                i if i == "Fallible" => {
                    let args = extract!(path.segments.last().unwrap().arguments, PathArguments::AngleBracketed(ref ab) => &ab.args);
                    if args.len() != 1 {
                        return Err(darling::Error::custom("Fallible expects one argument"));
                    }
                    let rtype = extract!(args[0], GenericArgument::Type(ref ty) => ty);
                    format!("FfiResult<{}>", rust_to_c_type(rtype.clone())?)
                }
                i => panic!("unrecognized type: {}", i),
            }
        }
        Type::Tuple(_) => "AnyObject *".to_string(),
        _ => return Err(darling::Error::custom("unrecognized type structure")),
    })
}
