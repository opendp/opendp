use darling::{Error, FromMeta, Result};
use proc_macro2::TokenStream;
use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};
use syn::{Lit, Meta, MetaList, MetaNameValue, NestedMeta, Type};

use crate::{RuntimeType, Value};

use super::syn_type_to_runtime_type;

#[derive(Debug, FromMeta, Clone)]
pub struct Bootstrap {
    #[allow(dead_code)]
    pub module: Option<String>,
    pub proof: Option<String>,
    pub features: Features,
    #[darling(default)]
    pub generics: BootstrapTypes,
    #[darling(default)]
    pub arguments: BootstrapTypes,
    pub derived_types: Option<DerivedTypes>,
    pub returns: Option<BootstrapType>,
}

impl Bootstrap {
    pub fn from_attribute_args(items: &[NestedMeta]) -> darling::Result<Self> {
        Self::from_list(items)
    }
}

#[derive(Debug, Clone)]
pub struct DerivedTypes(pub HashMap<String, RuntimeType>);

impl FromMeta for DerivedTypes {
    fn from_list(items: &[NestedMeta]) -> Result<Self> {
        items
            .iter()
            .map(|nested| {
                if let NestedMeta::Meta(Meta::List(MetaList { path, nested, .. })) = nested {
                    let type_name = path
                        .get_ident()
                        .ok_or_else(|| {
                            Error::custom("path must consist of a single ident").with_span(path)
                        })?
                        .to_string();

                    if nested.len() != 1 {
                        return Err(Error::custom("nested must consist of a single meta")
                            .with_span(&nested));
                    }

                    let ty =
                        OptionRuntimeType::from_nested_meta(nested.first().ok_or_else(|| {
                            Error::custom("must have length at least 1").with_span(nested)
                        })?)?;
                    Ok((
                        type_name,
                        ty.0.ok_or_else(|| {
                            Error::custom("derived types must have valid type information")
                                .with_span(&nested)
                        })?,
                    ))
                } else {
                    Err(Error::custom("expected metalist").with_span(nested))
                }
            })
            .collect::<Result<HashMap<String, RuntimeType>>>()
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
pub struct BootstrapTypes(pub HashMap<String, BootstrapType>);

impl FromMeta for BootstrapTypes {
    fn from_list(items: &[NestedMeta]) -> darling::Result<Self> {
        (items.iter())
            .map(|nested| {
                if let NestedMeta::Meta(Meta::List(MetaList { path, nested, .. })) = nested {
                    let type_name = path
                        .get_ident()
                        .ok_or_else(|| {
                            darling::Error::custom("path must consist of a single ident")
                                .with_span(path)
                        })?
                        .to_string();
                    let type_ =
                        BootstrapType::from_list(&nested.into_iter().cloned().collect::<Vec<_>>())?;
                    Ok((type_name, type_))
                } else {
                    Err(darling::Error::custom("expected metalist").with_span(nested))
                }
            })
            .collect::<darling::Result<HashMap<String, BootstrapType>>>()
            .map(BootstrapTypes)
    }
}

#[derive(Debug, FromMeta, Clone)]
pub struct BootstrapType {
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
            syn::Lit::Int(int) => Value::Integer(int.base10_parse::<i64>()?),
            syn::Lit::Float(float) => Value::Float(float.base10_parse::<f64>()?),
            syn::Lit::Bool(bool) => Value::Bool(bool.value),
            lit => return Err(darling::Error::unexpected_lit_type(lit).with_span(value)),
        })))
    }
}

/// retrieve the first child of the first node
fn runtimetype_first(v: OptionRuntimeType) -> OptionRuntimeType {
    OptionRuntimeType(v.0.map(|v| match v {
        RuntimeType::Function { mut params, .. } => params.remove(0),
        RuntimeType::Nest { mut args, .. } => args.remove(0),
        _ => unreachable!(),
    }))
}

impl FromMeta for OptionRuntimeType {
    fn from_nested_meta(item: &NestedMeta) -> darling::Result<Self> {
        match item {
            NestedMeta::Meta(meta) => Self::from_meta(meta),
            NestedMeta::Lit(lit) => Self::from_value(lit),
        }
    }

    fn from_meta(item: &Meta) -> Result<Self> {
        if let Meta::List(MetaList { path, nested, .. }) = item {
            let first_child = nested.iter().next();

            // check if the left hand side of a MetaNameValue is equal to value
            let lhs_eq = |nv: &MetaNameValue, value| {
                nv.path.get_ident().map(|nv| nv == value).unwrap_or(false)
            };

            Ok(match first_child {
                Some(NestedMeta::Meta(Meta::NameValue(mnv))) if lhs_eq(mnv, "id") => {
                    let ts = TokenStream::from_str(
                        match &mnv.lit {
                            Lit::Str(ref litstr) => litstr.value(),
                            lit => {
                                return Err(Error::custom("type id literals must be strings")
                                    .with_span(&lit))
                            }
                        }
                        .as_str(),
                    )
                    .map_err(|e| {
                        Error::custom(format!("error while lexing: {:?}", e)).with_span(mnv)
                    })?;
                    let ty = syn::parse2::<Type>(ts.clone()).map_err(|e| {
                        Error::custom(format!(
                            "error while parsing type {}: {}",
                            ts,
                            e.to_string()
                        ))
                        .with_span(&mnv.lit)
                    })?;
                    syn_type_to_runtime_type(&ty)?
                }
                None => RuntimeType::None,
                _ => RuntimeType::Function {
                    function: (path.get_ident())
                        .ok_or_else(|| {
                            darling::Error::custom("path must consist of a single ident")
                                .with_span(path)
                        })?
                        .to_string(),
                    params: (nested.iter())
                        .map(OptionRuntimeType::from_nested_meta)
                        .map(|ort| ort.map(|ort| ort.0.unwrap_or(RuntimeType::None)))
                        .collect::<darling::Result<Vec<RuntimeType>>>()?,
                },
            }
            .into())
        } else {
            Err(Error::custom("expected metalist").with_span(item))
        }
    }
    fn from_value(value: &Lit) -> Result<Self> {
        match value {
            Lit::Str(litstr) => Ok(RuntimeType::Name(litstr.value()).into()),
            _ => Err(Error::custom("expected string").with_span(value)),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct DefaultGenerics(pub HashSet<String>);

impl FromMeta for DefaultGenerics {
    fn from_list(items: &[NestedMeta]) -> Result<Self> {
        (items.iter())
            .map(|item| {
                if let NestedMeta::Lit(Lit::Str(litstr)) = item {
                    Ok(litstr.value())
                } else {
                    Err(Error::custom("expected string").with_span(item))
                }
            })
            .collect::<Result<_>>()
            .map(DefaultGenerics)
    }
}
