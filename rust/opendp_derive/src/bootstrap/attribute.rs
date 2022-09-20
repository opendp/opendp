use std::collections::{HashMap, HashSet};

use darling::FromMeta;
use serde_json::{Number, Value};
use syn::{parse, GenericArgument, Lit, Meta, MetaList, NestedMeta, Path, Type, MetaNameValue};

use opendp_pre_derive::RuntimeType;

use crate::extract;

#[derive(Debug, FromMeta, Clone)]
pub(crate) struct BootstrapAttribute {
    #[allow(dead_code)]
    pub module: Option<String>,
    #[allow(dead_code)]
    pub name: Option<String>,
    pub proof: Option<String>,
    pub features: Features,
    #[darling(default)]
    pub generics: BootstrapTypes,
    #[darling(default)]
    pub arguments: BootstrapTypes,
    pub derived_types: Option<DerivedTypes>,
    pub returns: Option<BootstrapType>,
}

#[derive(Debug, Clone)]
pub struct DerivedTypes(pub HashMap<String, RuntimeType>);

impl FromMeta for DerivedTypes {
    fn from_list(items: &[NestedMeta]) -> darling::Result<Self> {
        items
            .iter()
            .map(|nested| {
                if let NestedMeta::Meta(Meta::List(MetaList { path, nested, .. })) = nested {
                    let type_name = path
                        .get_ident()
                        .ok_or_else(|| {
                            darling::Error::custom("path must consist of a single ident")
                        })?
                        .to_string();

                    if nested.len() != 1 {
                        return Err(darling::Error::custom(
                            "nested must consist of a single meta",
                        ));
                    }

                    let type_ = OptionRuntimeType::from_nested_meta(
                        &nested.first().expect("nested must have length at least 1"),
                    )?;
                    Ok((
                        type_name,
                        type_
                            .0
                            .expect("derived types must have valid type information"),
                    ))
                } else {
                    Err(darling::Error::custom("expected metalist"))
                }
            })
            .collect::<darling::Result<HashMap<String, RuntimeType>>>()
            .map(DerivedTypes)
    }
}

#[derive(Debug, Clone)]
pub struct Features(pub Vec<String>);

impl FromMeta for Features {
    fn from_list(items: &[NestedMeta]) -> darling::Result<Self> {
        items
            .iter()
            .map(String::from_nested_meta)
            .collect::<darling::Result<Vec<String>>>()
            .map(Features)
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct BootstrapTypes(pub HashMap<String, BootstrapType>);

impl FromMeta for BootstrapTypes {
    fn from_list(items: &[NestedMeta]) -> darling::Result<Self> {
        items
            .iter()
            .map(|nested| {
                if let NestedMeta::Meta(Meta::List(MetaList { path, nested, .. })) = nested {
                    let type_name = path
                        .get_ident()
                        .ok_or_else(|| {
                            darling::Error::custom("path must consist of a single ident")
                        })?
                        .to_string();
                    let type_ =
                        BootstrapType::from_list(&nested.into_iter().cloned().collect::<Vec<_>>())?;
                    Ok((type_name, type_))
                } else {
                    Err(darling::Error::custom("expected metalist"))
                }
            })
            .collect::<darling::Result<HashMap<String, BootstrapType>>>()
            .map(BootstrapTypes)
    }
}

#[derive(Debug, FromMeta, Clone)]
pub(crate) struct BootstrapType {
    pub c_type: Option<String>,
    #[darling(default)]
    pub rust_type: OptionRuntimeType,
    pub hint: Option<String>,
    #[darling(default)]
    pub default: OptionValue,
    #[darling(default)]
    pub generics: DefaultGenerics,
    #[darling(default)]
    pub do_not_convert: bool,
    #[darling(map = "runtimetype_first", default)]
    pub example: OptionRuntimeType,
}

#[derive(Debug, Clone, Default)]
pub struct OptionRuntimeType(pub Option<RuntimeType>);

#[derive(Debug, Default, Clone)]
pub struct OptionValue(pub Option<Value>);

impl From<RuntimeType> for OptionRuntimeType {
    fn from(v: RuntimeType) -> Self {
        OptionRuntimeType(Some(v))
    }
}

impl FromMeta for OptionValue {
    fn from_value(value: &syn::Lit) -> darling::Result<Self> {
        Ok(OptionValue(Some(match value {
            syn::Lit::Str(str) => Value::String(str.value()),
            syn::Lit::Int(int) => Value::Number(int.base10_parse::<i64>()?.into()),
            syn::Lit::Float(float) => Value::Number(
                Number::from_f64(float.base10_parse::<f64>()?).expect("failed to parse f64"),
            ),
            syn::Lit::Bool(bool) => Value::Bool(bool.value),
            _ => return Err(darling::Error::custom("unrecognized type")),
        })))
    }
}

/// retrieve the first child of the first node
fn runtimetype_first(v: OptionRuntimeType) -> OptionRuntimeType {
    OptionRuntimeType(v.0.map(|v| match v {
        RuntimeType::Function { mut params, .. } => params.remove(0),
        RuntimeType::Nest { mut args, .. } => args.remove(0),
        _ => unimplemented!(),
    }))
}

impl FromMeta for OptionRuntimeType {
    fn from_nested_meta(item: &NestedMeta) -> darling::Result<Self> {
        match item {
            NestedMeta::Meta(meta) => Self::from_meta(meta),
            NestedMeta::Lit(lit) => Self::from_value(lit),
        }
    }

    fn from_meta(item: &Meta) -> darling::Result<Self> {
        
        if let Meta::List(MetaList { path, nested, .. }) = item {
            
            let first_child = nested.iter().next();

            // check if the left hand side of a MetaNameValue is equal to value
            let lhs_eq = |nv: &MetaNameValue, value| nv.path.get_ident().map(|nv| nv == value).unwrap_or(false);

            Ok(match first_child {
                Some(NestedMeta::Meta(Meta::NameValue(mnv))) if lhs_eq(mnv, "id") => {
                    let ts: proc_macro::TokenStream = extract!(mnv.lit, Lit::Str(ref litstr) => litstr)
                        .value().parse()
                        .map_err(|e| darling::Error::custom(format!("syn error: {:?}", e)))?;
                    type_to_rtype(parse::<Type>(ts)?)
                }
                None => RuntimeType::None,
                _ => RuntimeType::Function {
                    function: (path.get_ident())
                        .ok_or_else(|| darling::Error::custom("path must consist of a single ident"))?
                        .to_string(),
                    params: (nested.iter())
                        .map(OptionRuntimeType::from_nested_meta)
                        .map(|ort| ort.map(|ort| ort.0.unwrap()))
                        .collect::<darling::Result<Vec<RuntimeType>>>()?,
                },
            }.into())
        } else {
            Err(darling::Error::custom("expected metalist"))
        }
    }
    fn from_value(value: &syn::Lit) -> darling::Result<Self> {
        if let syn::Lit::Str(value) = value {
            Ok(RuntimeType::Name(value.value()).into())
        } else {
            Err(darling::Error::custom("expected string"))
        }
    }
}

fn path_to_rtype(path: Path) -> RuntimeType {
    let segment = path.segments.last().expect("no last segment");
    match segment.arguments.clone() {
        syn::PathArguments::None => RuntimeType::Name(segment.ident.to_string()),
        syn::PathArguments::AngleBracketed(angle_bracketed) => RuntimeType::Nest {
            origin: segment.ident.to_string(),
            args: angle_bracketed
                .args
                .into_iter()
                .map(|arg| extract!(arg, GenericArgument::Type(v) => v))
                .map(type_to_rtype)
                .collect(),
        },
        syn::PathArguments::Parenthesized(_) => {
            panic!("parenthesized paths are not supported")
        }
    }
}

fn type_to_rtype(type_: Type) -> RuntimeType {
    match type_ {
        Type::Path(path) => path_to_rtype(path.path),
        Type::Tuple(tuple) => RuntimeType::Nest {
            origin: String::from("Tuple"),
            args: tuple.elems.into_iter().map(type_to_rtype).collect(),
        },
        _ => panic!("unexpected type_ variant {:?}", type_),
    }
}


#[derive(Debug, Default, Clone)]
pub(crate) struct DefaultGenerics(pub HashSet<String>);

impl FromMeta for DefaultGenerics {
    fn from_list(items: &[NestedMeta]) -> darling::Result<Self> {
        items
            .iter()
            .map(|item| extract!(item, NestedMeta::Lit(lit) => lit))
            .map(|lit| {
                if let syn::Lit::Str(value) = lit {
                    Ok(value.value())
                } else {
                    Err(darling::Error::custom("expected string"))
                }
            })
            .collect::<darling::Result<_>>()
            .map(DefaultGenerics)
    }
}
