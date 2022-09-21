use darling::{Error, FromMeta, Result};
use proc_macro2::TokenStream;
use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};
use syn::{Lit, Meta, MetaList, MetaNameValue, NestedMeta, Path, Type};

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

#[derive(Debug, Clone, Default)]
pub struct DerivedTypes(pub Vec<(String, RuntimeType)>);

impl FromMeta for DerivedTypes {
    fn from_list(items: &[NestedMeta]) -> Result<Self> {
        // each item should be a metalist consisting of the derived type name and runtime type info: T(id = "X")
        (items.iter())
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
                        NewRuntimeType::from_nested_meta(nested.first().ok_or_else(|| {
                            Error::custom("must have length at least 1").with_span(nested)
                        })?)?;
                    Ok((type_name, ty.0))
                } else {
                    Err(Error::custom("expected metalist in DerivedTypes").with_span(nested))
                }
            })
            .collect::<Result<Vec<(String, RuntimeType)>>>()
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
                    Err(
                        darling::Error::custom("expected metalist in BootstrapTypes")
                            .with_span(nested),
                    )
                }
            })
            .collect::<darling::Result<HashMap<String, BootstrapType>>>()
            .map(BootstrapTypes)
    }
}

#[derive(Debug, FromMeta, Clone)]
pub struct BootstrapType {
    pub c_type: Option<String>,
    #[darling(map = "runtimetype_first")]
    pub rust_type: Option<NewRuntimeType>,
    pub hint: Option<String>,
    #[darling(default)]
    pub default: OptionValue,
    #[darling(default)]
    pub generics: DefaultGenerics,
    #[darling(default)]
    pub do_not_convert: bool,
    #[darling(map = "runtimetype_first")]
    pub example: Option<NewRuntimeType>,
}

#[derive(Debug, Clone)]
pub struct NewRuntimeType(pub RuntimeType);

#[derive(Debug, Default, Clone)]
pub struct OptionValue(pub Option<Value>);

impl From<RuntimeType> for NewRuntimeType {
    fn from(v: RuntimeType) -> Self {
        NewRuntimeType(v)
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
    fn from_list(items: &[NestedMeta]) -> Result<Self> {
        if items.is_empty() {
            Ok(OptionValue(Some(Value::Null)))
        } else {
            Err(Error::custom("option meta list must be empty to denote a null value"))
        }
    }
}

/// retrieve the first child of the first node
fn runtimetype_first(v: Option<NewRuntimeType>) -> Option<NewRuntimeType> {
    v.map(|v| v.0)
        .map(|v| {
            match v {
                RuntimeType::Function { params, .. } => params,
                RuntimeType::Nest { args, .. } => args,
                _ => unreachable!(),
            }
            .first().cloned().unwrap_or(RuntimeType::None)
        })
        .map(NewRuntimeType)
}

impl FromMeta for NewRuntimeType {
    fn from_list(items: &[NestedMeta]) -> Result<Self> {
        match items.len() {
            0 => Ok(NewRuntimeType(RuntimeType::None)),
            1 => NewRuntimeType::from_nested_meta(&items[0]),
            _ => {
                Err(darling::Error::custom("rust_type given too many arguments")
                    .with_span(&items[1]))
            }
        }
    }
    fn from_meta(item: &Meta) -> Result<Self> {
        let extract_ident = |path: &Path| {
            Result::Ok(
                path.get_ident()
                    .ok_or_else(|| {
                        darling::Error::custom("path must consist of a single ident")
                            .with_span(&path)
                    })?
                    .to_string(),
            )
        };
        Ok(NewRuntimeType(match item {
            Meta::List(MetaList { path, nested, .. }) => RuntimeType::Function {
                function: extract_ident(path)?,
                params: (nested.iter())
                    .map(NewRuntimeType::from_nested_meta)
                    .map(|ort| ort.map(|ort| ort.0))
                    .collect::<darling::Result<Vec<RuntimeType>>>()?,
            },
            Meta::NameValue(MetaNameValue { path, lit, .. }) => {
                if extract_ident(path)?.as_str() != "id" {
                    return Err(darling::Error::custom(
                        "The only supported NameValue argument is \"id\"",
                    )
                    .with_span(&path));
                }
                let ts = TokenStream::from_str(
                    match &lit {
                        Lit::Str(ref litstr) => litstr.value(),
                        lit => {
                            return Err(
                                Error::custom("type id literals must be strings").with_span(&lit)
                            )
                        }
                    }
                    .as_str(),
                )
                .map_err(|e| {
                    Error::custom(format!("error while lexing: {:?}", e)).with_span(lit)
                })?;
                let ty = syn::parse2::<Type>(ts.clone()).map_err(|e| {
                    Error::custom(format!(
                        "error while parsing type {}: {}",
                        ts,
                        e.to_string()
                    ))
                    .with_span(&lit)
                })?;
                syn_type_to_runtime_type(&ty)?
            }
            Meta::Path(path) => {
                return Err(
                    Error::custom("paths are invalid arguments to rust_type").with_span(path)
                )
            }
        }))
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
