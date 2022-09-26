use std::collections::{HashMap, HashSet};
use syn::{
    AttributeArgs, FnArg, GenericArgument, GenericParam, ItemFn, Pat, PathArguments, ReturnType,
    Signature, Type, TypeParam, TypePath, TypePtr, TypeReference,
};

use crate::{Argument, Function, RuntimeType};

pub mod arguments;
pub mod docstring;

use darling::{Error, Result};

use crate::bootstrap::{arguments::Bootstrap, docstring::Docstring};

use self::arguments::BootstrapType;

impl Function {
    pub fn from_ast(
        attr_args: AttributeArgs,
        item_fn: ItemFn,
        proof_paths: &HashMap<String, Option<String>>,
    ) -> Result<Function> {
        // Parse the proc bootstrap macro args
        let arguments = Bootstrap::from_attribute_args(&attr_args)?;

        // Parse the docstring
        let docstring = Docstring::from_attrs(item_fn.attrs, &item_fn.sig.output)?;

        // aggregate info from all sources
        reconcile_function(arguments, docstring, item_fn.sig, proof_paths)
    }
}

fn syn_generic_to_syn_type_param(generic: &GenericParam) -> Result<&TypeParam> {
    match generic {
        GenericParam::Type(v) => Ok(v),
        GenericParam::Lifetime(l) => {
            Err(Error::custom("lifetimes are not supported in bootstrap functions").with_span(l))
        }
        GenericParam::Const(c) => {
            Err(Error::custom("consts are not supported in bootstrap functions").with_span(c))
        }
    }
}

pub fn reconcile_function(
    bootstrap: Bootstrap,
    mut doc_comments: Docstring,
    signature: Signature,
    proof_paths: &HashMap<String, Option<String>>,
) -> Result<Function> {
    // extract all generics from function
    // used to identify when a type is a generic argument
    let all_generics = (signature.generics.params.iter())
        .map(syn_generic_to_syn_type_param)
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .map(|param| param.ident.to_string())
        .chain(
            bootstrap
                .derived_types
                .clone()
                .unwrap_or_default()
                .0
                .iter()
                .map(|v| v.0.clone()),
        )
        .collect::<HashSet<String>>();

    let name = signature.ident.to_string();

    Ok(Function {
        name: name.clone(),
        features: bootstrap.features.0,
        description: if doc_comments.description.is_empty() {
            None
        } else {
            Some(doc_comments.description.join("\n").trim().to_string())
        },
        proof_path: reconcile_proof_path(
            bootstrap.proof_path,
            &name,
            proof_paths
        )?,
        args: reconcile_arguments(
            &bootstrap.arguments.0,
            &mut doc_comments.arguments,
            signature.inputs.into_iter().collect(),
            &all_generics,
        )?
        .into_iter()
        .chain(reconcile_generics(
            &bootstrap.generics.0,
            &mut doc_comments.generics,
            signature.generics.params.into_iter().collect(),
        )?)
        .collect::<Vec<_>>(),
        ret: reconcile_return(
            bootstrap.returns,
            doc_comments.ret,
            signature.output,
            &all_generics,
        )?,
        derived_types: bootstrap
            .derived_types
            .map(|dt| dt.0)
            .unwrap_or_default()
            .into_iter()
            .map(|(name, rt)| Argument {
                name: Some(name),
                rust_type: Some(rt),
                ..Default::default()
            })
            .collect(),
    })
}

fn reconcile_proof_path(
    bootstrap: Option<String>,
    name: &str,
    proof_paths: &HashMap<String, Option<String>>
) -> Result<Option<String>> {
    Ok(match bootstrap {
        Some(proof_path) => Some(proof_path),
        None => match proof_paths.get(name) {
            Some(None) => return Err(Error::custom(format!("more than one file named {name}.tex. Please specify `proof_path = \"{{module}}/path/to/proof.tex\"` in the macro attributes."))),
            Some(proof_path) => proof_path.clone(),
            None => None
        }
    })
}

fn reconcile_arguments(
    bootstrap_args: &HashMap<String, BootstrapType>,
    doc_comments: &mut HashMap<String, Vec<String>>,
    inputs: Vec<FnArg>,
    all_generics: &HashSet<String>,
) -> Result<Vec<Argument>> {
    (inputs.into_iter())
        .map(|v| {
            let pat_type = match v {
                FnArg::Receiver(r) => {
                    return Err(Error::custom(
                        "bootstrap functions don't support receiver (self) args",
                    )
                    .with_span(&r))
                }
                FnArg::Typed(t) => t,
            };

            let name = syn_pat_to_name(*pat_type.pat)?;

            // struct of additional metadata for this argument supplied by bootstrap macro
            let boot_type = bootstrap_args.get(&name);

            // if rust type is given, use it. Otherwise parse the rust type on the function
            let rust_type = match boot_type.and_then(|bt| bt.rust_type.clone()) {
                Some(v) => v,
                None => syn_type_to_runtime_type(&*pat_type.ty)?,
            };
            Ok(Argument {
                name: Some(name.clone()),
                c_type: Some(match boot_type.as_ref().and_then(|bt| bt.c_type.as_ref()) {
                    Some(ref v) => v.to_string(),
                    None => syn_type_to_c_type(*pat_type.ty, &all_generics)?,
                }),
                rust_type: Some(rust_type),
                generics: boot_type
                    .map(|bt| Vec::from_iter(bt.generics.0.iter().cloned()))
                    .unwrap_or_else(Vec::new),
                description: doc_comments
                    .remove(&name)
                    .map(|dc| dc.join("\n").trim().to_string()),
                hint: boot_type.and_then(|bt| bt.hint.clone()),
                default: boot_type.and_then(|bt| bt.default.clone()),
                is_type: false,
                do_not_convert: boot_type.map(|bt| bt.do_not_convert).unwrap_or(false),
                example: None,
            })
        })
        .collect()
}

fn reconcile_generics(
    bootstrap_args: &HashMap<String, BootstrapType>,
    doc_comments: &mut HashMap<String, Vec<String>>,
    inputs: Vec<GenericParam>,
) -> Result<Vec<Argument>> {
    (inputs.into_iter())
        .map(|generic: GenericParam| {
            let param = syn_generic_to_syn_type_param(&generic)?;
            let name = param.ident.to_string();
            let boot_type = bootstrap_args.get(&name);
            Ok(Argument {
                name: Some(name.clone()),
                c_type: None,
                description: doc_comments
                    .remove(&name)
                    .map(|dc| dc.join("\n").trim().to_string()),
                rust_type: None,
                generics: boot_type
                    .map(|bt| Vec::from_iter(bt.generics.0.iter().cloned()))
                    .unwrap_or_else(Vec::new),
                hint: boot_type.and_then(|bt| bt.hint.clone()),
                default: boot_type.and_then(|bt| bt.default.clone()),
                is_type: true,
                do_not_convert: false,
                example: boot_type.and_then(|bt| bt.example.clone()),
            })
        })
        .collect()
}

fn reconcile_return(
    bootstrap: Option<BootstrapType>,
    doc_comment: Vec<String>,
    output: ReturnType,
    all_generics: &HashSet<String>,
) -> Result<Argument> {
    Ok(Argument {
        c_type: Some(match bootstrap.as_ref().and_then(|bt| bt.c_type.as_ref()) {
            Some(ref v) => v.to_string(),
            None => syn_type_to_c_type(
                match output {
                    ReturnType::Default => {
                        return Err(Error::custom(
                            "default return types are not supported in bootstrap functions",
                        )
                        .with_span(&output))
                    }
                    ReturnType::Type(_, ty) => *ty,
                },
                &all_generics,
            )?,
        }),
        rust_type: bootstrap.as_ref().and_then(|bs| bs.rust_type.clone()),
        description: if doc_comment.is_empty() {
            None
        } else {
            Some(doc_comment.join("\n").trim().to_string())
        },
        do_not_convert: bootstrap.map(|ret| ret.do_not_convert).unwrap_or(false),
        ..Default::default()
    })
}
/// extract name from pattern
fn syn_pat_to_name(pat: Pat) -> Result<String> {
    match pat {
        Pat::Box(b) => syn_pat_to_name(*b.pat),
        Pat::Ident(i) => Ok(i.ident.to_string()),
        Pat::Reference(r) => syn_pat_to_name(*r.pat),
        Pat::Type(t) => syn_pat_to_name(*t.pat),
        token => Err(Error::custom("unrecognized pattern in argument").with_span(&token)),
    }
}

fn syn_type_to_runtime_type(ty: &Type) -> Result<RuntimeType> {
    Ok(match ty {
        Type::Path(tpath) => {
            let segment = (tpath.path.segments.last()).ok_or_else(|| {
                Error::custom("paths must have at least one segment").with_span(ty)
            })?;

            let name = segment.ident.to_string();
            match &segment.arguments {
                PathArguments::None => RuntimeType::Name(name),
                PathArguments::AngleBracketed(ab) => {
                    let args = (ab.args.iter())
                        .map(|arg| syn_type_to_runtime_type(syn_generic_arg_to_syn_type(arg)?))
                        .collect::<Result<Vec<_>>>()?;

                    RuntimeType::Nest {
                        origin: name,
                        args: args,
                    }
                }
                PathArguments::Parenthesized(p) => {
                    return Err(Error::custom("parenthesized paths are not supported").with_span(p))
                }
            }
            .into()
        }
        Type::Reference(refer) => syn_type_to_runtime_type(&*refer.elem)?,
        Type::Tuple(tuple) => RuntimeType::Nest {
            origin: "Tuple".to_string(),
            args: (tuple.elems.iter())
                .map(|ty| syn_type_to_runtime_type(ty))
                .collect::<Result<Vec<_>>>()?,
        }
        .into(),
        Type::Ptr(ptr) => syn_type_to_runtime_type(&*ptr.elem)?,
        t => return Err(Error::custom("unrecognized type for RuntimeType").with_span(t)),
    })
}

fn syn_generic_arg_to_syn_type(arg: &GenericArgument) -> Result<&Type> {
    match arg {
        GenericArgument::Lifetime(l) => {
            Err(Error::custom("lifetimes are not supported in bootstrap functions").with_span(l))
        }
        GenericArgument::Type(ty) => Ok(ty),
        GenericArgument::Binding(b) => {
            Err(Error::custom("bindings are not supported in bootstrap functions").with_span(b))
        }
        GenericArgument::Constraint(c) => {
            Err(Error::custom("type constraints are invalid in this context").with_span(c))
        }
        GenericArgument::Const(c) => {
            Err(Error::custom("consts are not supported in bootstrap functions").with_span(c))
        }
    }
}

fn syn_type_to_c_type(ty: Type, generics: &HashSet<String>) -> Result<String> {
    Ok(match ty {
        Type::Path(TypePath { path, .. }) => {
            let segment = (path.segments.last())
                .ok_or_else(|| Error::custom("at least one segment required").with_span(&path))?;

            match segment.ident.to_string() {
                i if i == "Option" => {
                    let first_arg = if let PathArguments::AngleBracketed(ab) = &segment.arguments {
                        ab.args.first().ok_or_else(|| {
                            Error::custom("Option must have one argument").with_span(&ab)
                        })?
                    } else {
                        return Err(
                            Error::custom("Option must have angle brackets").with_span(segment)
                        );
                    };

                    let inner_c_type = if let GenericArgument::Type(ty) = first_arg {
                        syn_type_to_c_type(ty.clone(), generics)?
                    } else {
                        return Err(
                            Error::custom("Option's argument must be a Type").with_span(segment)
                        );
                    };
                    match inner_c_type.as_str() {
                        "AnyObject *" => "AnyObject *".to_string(),
                        "char *" => "char *".to_string(),
                        _ => "void *".to_string(),
                    }
                }
                i if i == "String" || i == "c_char" => "AnyObject *".to_string(),
                i if i == "AnyObject" => "AnyObject *".to_string(),
                i if i == "Vec" => "AnyObject *".to_string(),
                i if i == "HashSet" => "AnyObject *".to_string(),
                i if i == "bool" || i == "c_bool" => "bool".to_string(),
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
                i if i == "FfiSlice" => "FfiSlice *".to_string(),
                i if i == "Transformation" => "AnyTransformation *".to_string(),
                i if i == "Measurement" => "AnyMeasurement *".to_string(),
                i if i == "AnyTransformation" => "AnyTransformation *".to_string(),
                i if i == "AnyMeasurement" => "AnyMeasurement *".to_string(),
                i if i == "Fallible" || i == "FfiResult" => {
                    let args = match &segment.arguments {
                        PathArguments::AngleBracketed(ref ab) => &ab.args,
                        args => {
                            return Err(Error::custom("Fallible expects one type argument")
                                .with_span(&args))
                        }
                    };

                    if args.len() != 1 {
                        return Err(Error::custom("Fallible expects one argument"));
                    }
                    let rtype = syn_generic_arg_to_syn_type(&args[0])?;
                    format!(
                        "FfiResult<{}>",
                        syn_type_to_c_type(rtype.clone(), generics)?
                    )
                }
                i if generics.contains(&i) => "AnyObject *".to_string(),
                _ => {
                    return Err(Error::custom(
                        "Unrecognized rust type. Failed to convert to C type.",
                    )
                    .with_span(segment))
                }
            }
        }
        Type::Tuple(_) => "AnyObject *".to_string(),
        Type::Reference(TypeReference { elem, .. }) => syn_type_to_c_type(*elem, generics)?,
        Type::Ptr(TypePtr { elem, .. }) => syn_type_to_c_type(*elem, generics)?,
        ty => {
            return Err(Error::custom(
                "Unrecognized rust type structure. Failed to convert to C type.",
            )
            .with_span(&ty))
        }
    })
}
