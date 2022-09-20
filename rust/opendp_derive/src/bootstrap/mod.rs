use attribute::DerivedTypes;
use darling::FromMeta;
use docstring::DocComments;
use std::{
    collections::{HashMap, HashSet},
    env,
    path::PathBuf,
};
use syn::{
    AttributeArgs, FnArg, GenericArgument, GenericParam, ItemFn, Pat,
    PathArguments, ReturnType, Signature, Type, TypeParam, TypePath,
};

use opendp_pre_derive::{target::find_target_dir, Argument, Function, RuntimeType};

use crate::bootstrap::{attribute::BootstrapAttribute, docstring::parse_doc_comments};
use crate::extract;

mod attribute;
mod docstring;


pub fn write_json(module: String, attr: AttributeArgs, input: ItemFn, proof_link: Option<String>) -> darling::Result<()> {
    // Parse the attributes and function signature
    let bootstrap = BootstrapAttribute::from_list(&attr)?;
    let ItemFn {attrs, sig, ..} = input;

    let doc_comments = parse_doc_comments(attrs, proof_link);

    let (name, function) = make_bootstrap_json(sig, bootstrap.clone(), doc_comments)?;

    let json_str =
        serde_json::to_string_pretty(&function)
        .expect("failed to serialize function to json");
    // println!("{module}::{name}({json_str})");

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR must be set."));
    let target_dir = find_target_dir(&out_dir);
    let json_module_dir = target_dir.join("opendp_bootstrap").join(module.clone());

    // dbg!(&json_module_dir);

    std::fs::create_dir_all(&json_module_dir).expect(
        format!("unable to create folder {{target_dir}}/opendp_bootstrap/{module}").as_str(),
    );

    let filename = format!("{}.json", name);
    let json_path = json_module_dir.join(filename.clone());
    std::fs::write(&json_path, json_str).expect(
        format!("unable to write file {{target_dir}}/opendp_bootstrap/{module}/{filename}")
            .as_str(),
    );
    Ok(())
}

fn make_bootstrap_json(
    signature: Signature,
    bootstrap: BootstrapAttribute,
    mut doc_comments: DocComments,
) -> darling::Result<(String, Function)> {
    let all_generics = (signature.generics.params.iter())
        .map(|param| extract!(param, GenericParam::Type(v) => v))
        .map(|param| param.ident.to_string())
        .chain(
            bootstrap
                .derived_types.clone()
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
            .map(|v| (
                extract!(*v.pat, Pat::Ident(v) => v.ident.to_string()),
                *v.ty,
            ))
            .map(|(name, ty)| {
                let boot_type = bootstrap.arguments.0.get(&name);
                // if rust type is given, use it. Otherwise parse the rust type on the function
                let rust_type = match boot_type.and_then(|bt| bt.rust_type.0.clone()) {
                    Some(v) => v,
                    None => syntype_to_runtimetype(ty.clone())?,
                };
                Ok(Argument {
                    name: Some(name.clone()),
                    c_type: Some(match boot_type.as_ref().and_then(|bt| bt.c_type.as_ref()) {
                        Some(ref v) => v.to_string(),
                        None => rust_to_c_type(ty, &all_generics)?,
                    }),
                    rust_type: Some(rust_type),
                    generics: boot_type.map(|bt| Vec::from_iter(bt.generics.0.iter().cloned())).unwrap_or_else(Vec::new),
                    description: doc_comments
                        .arguments
                        .remove(&name)
                        .map(|dc| dc.join("\n").trim().to_string()),
                    hint: boot_type.and_then(|bt| bt.hint.clone()),
                    default: boot_type.and_then(|bt| bt.default.0.clone()),
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
                            generics: boot_type.map(|bt| Vec::from_iter(bt.generics.0.iter().cloned())).unwrap_or_else(Vec::new),
                            hint: boot_type.and_then(|bt| bt.hint.clone()),
                            default: boot_type.and_then(|bt| bt.default.0.clone()),
                            is_type: true,
                            do_not_convert: false,
                            example: boot_type.and_then(|bt| bt.example.0.clone()),
                        })
                    }),
            )
            .collect::<darling::Result<Vec<_>>>()?,
        ret: Argument {
            name: None,
            c_type: Some(
                match bootstrap.returns.as_ref().and_then(|bt| bt.c_type.as_ref()) {
                    Some(ref v) => v.to_string(),
                    None => rust_to_c_type(
                        extract!(signature.output, ReturnType::Type(_, ty) => *ty),
                        &all_generics
                    )?
                },
            ),
            rust_type: bootstrap
                .returns
                .as_ref()
                .and_then(|bs| bs.rust_type.0.clone()),
            generics: Vec::new(),
            description: if doc_comments.ret.is_empty() {
                None
            } else {
                Some(doc_comments.ret.join("\n").trim().to_string())
            },
            hint: None,
            default: None,
            is_type: false,
            do_not_convert: bootstrap.returns.map(|ret| ret.do_not_convert).unwrap_or(false),
            example: None,
        },
        derived_types: bootstrap.derived_types.map(|dt| dt.0).unwrap_or_else(HashMap::new).into_iter().map(|(name, rt)| Argument {
            name: Some(name),
            c_type: None,
            rust_type: Some(rt),
            generics: Vec::new(),
            hint: None,
            description: None,
            default: None,
            is_type: false,
            do_not_convert: false,
            example: None,
        }).collect(),
    };

    let fn_name = bootstrap.name.unwrap_or_else(|| signature.ident.to_string());

    Ok((fn_name, function))
}

fn syntype_to_runtimetype(
    type_: Type
) -> darling::Result<RuntimeType> {
    let runtime_type = match type_ {
        Type::Path(tpath) => {
            let segment = tpath.path.segments.last().expect("paths must have at least one segment");
            let name = segment.ident.to_string();
            match &segment.arguments {
                PathArguments::None => RuntimeType::Name(name),
                PathArguments::AngleBracketed(ab) => {
                    let args = (ab.args.iter())
                        .map(|arg| extract!(arg, GenericArgument::Type(ty) => syntype_to_runtimetype(ty.clone())))
                        .collect::<darling::Result<Vec<_>>>()?;
                    
                    RuntimeType::Nest {
                        origin: name,
                        args: args
                    }
                },
                PathArguments::Parenthesized(_) => return Err(darling::Error::custom("parenthesized paths are not supported")),
            }.into()
        }
        Type::Reference(refer) => syntype_to_runtimetype(*refer.elem)?,
        Type::Tuple(tuple) => RuntimeType::Nest {
                origin: "Tuple".to_string(),
                args: (tuple.elems.into_iter())
                    .map(|type_| syntype_to_runtimetype(type_))
                    .collect::<darling::Result<Vec<_>>>()?
            }.into(),
        t => panic!("unrecognized Type {:?}", t),
    };

    Ok(runtime_type)
}

fn rust_to_c_type(ty: Type, generics: &HashSet<String>) -> darling::Result<String> {
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
                    format!("FfiResult<{}>", rust_to_c_type(rtype.clone(), generics)?)
                }
                i if generics.contains(&i) => "AnyObject *".to_string(),
                i => panic!("unrecognized type: {}", i),
            }
        }
        Type::Tuple(_) => "AnyObject *".to_string(),
        _ => return Err(darling::Error::custom("unrecognized type structure")),
    })
}
