use std::collections::HashSet;

use darling::{Error, Result};
use syn::{
    FnArg, GenericArgument, GenericParam, Pat, Path, PathArguments, ReturnType, Signature, Type,
    TypeParam, TypePath, TypePtr, TypeReference,
};

use crate::TypeRecipe;

use super::partial::supports_partial;

// try to keep syn parsing insanity contained in this file
// extract what we need out of the syn signature into parsimonious OpenDP structures.

pub struct BootstrapSignature {
    pub name: String,
    pub arguments: Vec<(String, BootSigArgType)>,
    pub generics: Vec<String>,
    pub output_c_type: Result<String>,
    pub supports_partial: bool,
}

pub struct BootSigArgType {
    pub c_type: Result<String>,
    pub rust_type: Result<TypeRecipe>,
}

impl BootstrapSignature {
    pub fn from_syn(sig: Signature) -> Result<Self> {
        let supports_partial = supports_partial(&sig);

        let generics = sig
            .generics
            .params
            .into_iter()
            .map(|generic| syn_generic_to_syn_type_param(&generic).map(|v| v.ident.to_string()))
            .collect::<Result<Vec<_>>>()?;

        Ok(BootstrapSignature {
            name: sig.ident.to_string(),
            arguments: sig
                .inputs
                .into_iter()
                .map(|fn_arg| {
                    let (pat, ty) = syn_fnarg_to_syn_pattype(fn_arg)?;

                    Ok((
                        syn_pat_to_string(&pat)?,
                        BootSigArgType {
                            rust_type: syn_type_to_type_recipe(&ty),
                            c_type: syn_type_to_c_type(ty, &HashSet::from_iter(generics.clone())),
                        },
                    ))
                })
                .collect::<Result<Vec<_>>>()?,
            generics: generics.clone(),
            output_c_type: syn_type_to_c_type(
                match sig.output {
                    ReturnType::Default => {
                        return Err(Error::custom(
                            "default return types are not supported in bootstrap functions",
                        )
                        .with_span(&sig.output))
                    }
                    ReturnType::Type(_, ty) => *ty,
                },
                &HashSet::from_iter(generics),
            ),
            supports_partial,
        })
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

pub(super) fn syn_path_to_string(path: &Path) -> Result<String> {
    match path.get_ident() {
        Some(ident) => Ok(ident.to_string()),
        None => Err(Error::custom("path must be consist of a single identifier").with_span(&path)),
    }
}

/// extract name from pattern
fn syn_pat_to_string(pat: &Pat) -> Result<String> {
    match pat {
        Pat::Box(b) => syn_pat_to_string(&*b.pat),
        Pat::Ident(i) => Ok(i.ident.to_string()),
        Pat::Reference(r) => syn_pat_to_string(&*r.pat),
        Pat::Type(t) => syn_pat_to_string(&*t.pat),
        token => Err(Error::custom("unrecognized pattern in argument").with_span(&token)),
    }
}

pub(super) fn syn_type_to_type_recipe(ty: &Type) -> Result<TypeRecipe> {
    Ok(match ty {
        Type::Path(tpath) => {
            let segment = (tpath.path.segments.last()).ok_or_else(|| {
                Error::custom("paths must have at least one segment").with_span(ty)
            })?;

            let name = segment.ident.to_string();
            match &segment.arguments {
                PathArguments::None => TypeRecipe::Name(name),
                PathArguments::AngleBracketed(ab) => {
                    let args = (ab.args.iter())
                        .map(|arg| syn_type_to_type_recipe(syn_generic_arg_to_syn_type(arg)?))
                        .collect::<Result<Vec<_>>>()?;

                    TypeRecipe::Nest {
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
        Type::Reference(refer) => syn_type_to_type_recipe(&*refer.elem)?,
        Type::Tuple(tuple) => TypeRecipe::Nest {
            origin: "Tuple".to_string(),
            args: (tuple.elems.iter())
                .map(|ty| syn_type_to_type_recipe(ty))
                .collect::<Result<Vec<_>>>()?,
        }
        .into(),
        Type::Ptr(ptr) => syn_type_to_type_recipe(&*ptr.elem)?,
        t => return Err(Error::custom("unrecognized type for TypeRecipe").with_span(t)),
    })
}

fn syn_generic_arg_to_syn_type(arg: &GenericArgument) -> Result<&Type> {
    if let GenericArgument::Type(ty) = arg {
        Ok(ty)
    } else {
        Err(Error::custom("generic arguments in this position must be a type").with_span(arg))
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
                i if i == "String" => "AnyObject *".to_string(),
                i if i == "str" => "char *".to_string(),
                i if i == "c_char" => "char *".to_string(),
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
                i if i == "DataFrame" => "AnyObject *".to_string(),
                i if i == "LazyFrame" => "AnyObject *".to_string(),
                i if i == "Expr" => "AnyObject *".to_string(),
                i if i == "LazyFrameDomain" => "AnyDomain *".to_string(),
                i if i == "FfiSlice" => "FfiSlice *".to_string(),
                i if i == "Transformation" => "AnyTransformation *".to_string(),
                i if i == "ExtrinsicObject" => "ExtrinsicObject *".to_string(),
                i if i == "Measurement" => "AnyMeasurement *".to_string(),
                i if i == "Function" => "AnyFunction *".to_string(),
                i if i == "AnyFunction" => "AnyFunction *".to_string(),
                i if i == "AnyTransformation" => "AnyTransformation *".to_string(),
                i if i == "AnyMeasurement" => "AnyMeasurement *".to_string(),
                i if i == "AnyQueryable" => "AnyQueryable *".to_string(),
                i if i == "AnyDomain" => "AnyDomain *".to_string(),
                i if i == "AnyMetric" => "AnyMetric *".to_string(),
                i if i == "AnyMeasure" => "AnyMeasure *".to_string(),
                i if i == "CallbackFn" => "CallbackFn".to_string(),
                i if i == "TransitionFn" => "TransitionFn".to_string(),
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

pub fn syn_fnarg_to_syn_pattype(v: FnArg) -> Result<(Pat, Type)> {
    match v {
        FnArg::Receiver(r) => {
            let msg = "bootstrapped functions don't support receiver (self) args";
            Err(Error::custom(msg).with_span(&r))
        }
        FnArg::Typed(t) => Ok((*t.pat, *t.ty)),
    }
}
