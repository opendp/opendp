use std::collections::HashSet;
use syn::{
    AttributeArgs, FnArg, GenericArgument, GenericParam, ItemFn, Pat, PathArguments, ReturnType,
    Signature, Type, TypeParam, TypePath, TypeReference,
};

use crate::{Argument, Function, RuntimeType};

pub mod bootstrap;
pub mod docstring;

use darling::{Error, Result};

use crate::parse::{bootstrap::Bootstrap, docstring::Docstring};

use self::docstring::find_relative_proof_path;

impl Function {
    pub fn from_ast(attr_args: AttributeArgs, item_fn: ItemFn) -> Result<(String, Function)> {
        // Parse the proc bootstrap macro args
        let mut bootstrap = Bootstrap::from_attribute_args(&attr_args)?;

        // Parse the function signature
        let ItemFn { attrs, sig, .. } = item_fn;
        let func_name = sig.ident.to_string();

        // Try to enrich the bootstrap with a proof file
        if let None = bootstrap.proof {
            bootstrap.proof = find_relative_proof_path(&func_name);
        }

        // aggregate info from all sources
        let function = reconcile_function(bootstrap, Docstring::from_attrs(attrs)?, sig)?;

        Ok((func_name, function))
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
) -> Result<Function> {
    // extract all generics from function
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
                .iter().map(|v| v.0.clone()),
        )
        .collect::<HashSet<String>>();

    Ok(Function {
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
                let boot_type = bootstrap.arguments.0.get(&name);

                // if rust type is given, use it. Otherwise parse the rust type on the function
                let rust_type = match boot_type.and_then(|bt| bt.rust_type.clone().map(|rt| rt.0)) {
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
                (signature.generics.params.into_iter()).map(|generic: GenericParam| {
                    let param = syn_generic_to_syn_type_param(&generic)?;
                    let name = param.ident.to_string();
                    let boot_type = bootstrap.generics.0.get(&name);
                    Ok(Argument {
                        name: Some(name.clone()),
                        c_type: None,
                        description: doc_comments
                            .generics
                            .remove(&name)
                            .map(|dc| dc.join("\n").trim().to_string()),
                        rust_type: None,
                        generics: boot_type
                            .map(|bt| Vec::from_iter(bt.generics.0.iter().cloned()))
                            .unwrap_or_else(Vec::new),
                        hint: boot_type.and_then(|bt| bt.hint.clone()),
                        default: boot_type.and_then(|bt| bt.default.0.clone()),
                        is_type: true,
                        do_not_convert: false,
                        example: boot_type.and_then(|bt| bt.example.clone().map(|rt| rt.0)),
                    })
                }),
            )
            .collect::<Result<Vec<_>>>()?,
        ret: Argument {
            c_type: Some(
                match bootstrap.returns.as_ref().and_then(|bt| bt.c_type.as_ref()) {
                    Some(ref v) => v.to_string(),
                    None => {
                        syn_type_to_c_type(
                            match signature.output {
                                ReturnType::Default => return Err(Error::custom(
                                    "default return types are not supported in bootstrap functions",
                                )
                                .with_span(&signature.output)),
                                ReturnType::Type(_, ty) => *ty,
                            },
                            &all_generics,
                        )?
                    }
                },
            ),
            rust_type: bootstrap
                .returns
                .as_ref()
                .and_then(|bs| bs.rust_type.clone().map(|rt| rt.0)),
            description: if doc_comments.ret.is_empty() {
                None
            } else {
                Some(doc_comments.ret.join("\n").trim().to_string())
            },
            do_not_convert: bootstrap
                .returns
                .map(|ret| ret.do_not_convert)
                .unwrap_or(false),
            ..Default::default()
        },
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
                        ab.args.first().ok_or_else(|| Error::custom("Option must have one argument").with_span(&ab))?
                    } else {
                        return Err(Error::custom("Option must have angle brackets").with_span(segment))
                    };

                    let inner_c_type = if let GenericArgument::Type(ty) = first_arg {
                        syn_type_to_c_type(ty.clone(), generics)?
                    } else {
                        return Err(Error::custom("Option's argument must be a Type").with_span(segment))
                    };
                    match inner_c_type.as_str() {
                        "AnyObject *" => "AnyObject *".to_string(),
                        "char *" => "char *".to_string(),
                        _ => "void *".to_string()
                    }
                },
                i if i == "String" => "AnyObject *".to_string(),
                i if i == "AnyObject" => "AnyObject *".to_string(),
                i if i == "Vec" => "AnyObject *".to_string(),
                i if i == "HashSet" => "AnyObject *".to_string(),
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
                i if i == "AnyTransformation" => "AnyTransformation *".to_string(),
                i if i == "AnyMeasurement" => "AnyMeasurement *".to_string(),
                i if i == "Fallible" => {
                    let args = match &segment.arguments {
                        PathArguments::AngleBracketed(ref ab) => &ab.args,
                        args => return Err(Error::custom("Fallible expects one type argument").with_span(&args)),
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
                _ => return Err(Error::custom("Unrecognized rust type. Failed to convert to C type.").with_span(segment)),
            }
        }
        Type::Tuple(_) => "AnyObject *".to_string(),
        Type::Reference(TypeReference {elem, ..}) => syn_type_to_c_type(*elem, generics)?,
        ty => return Err(Error::custom("Unrecognized rust type structure. Failed to convert to C type.").with_span(&ty)),
    })
}
