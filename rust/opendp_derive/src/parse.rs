use std::collections::HashMap;

use darling::FromMeta;
use serde_json::{Value, Number};
use syn::{Meta, MetaList, NestedMeta};

use crate::json::RuntimeType;

#[derive(Debug, FromMeta, Clone)]
pub struct Bootstrap {
    pub proof: Option<String>,
    pub module: String,
    pub features: Features,
    pub generics: BootTypes,
    pub arguments: BootTypes,
    pub derived_types: Option<DerivedTypes>,
    pub ret: Option<BootType>
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
                        return Err(darling::Error::custom("nested must consist of a single meta"))
                    }

                    let type_ = RuntimeType::from_nested_meta(&nested.first().unwrap())?;
                    Ok((type_name, type_))
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

#[derive(Debug, Clone)]
pub struct BootTypes(pub HashMap<String, BootType>);

impl FromMeta for BootTypes {
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
                    let type_ = BootType::from_list(&nested.into_iter().cloned().collect::<Vec<_>>())?;
                    Ok((type_name, type_))
                } else {
                    Err(darling::Error::custom("expected metalist"))
                }
            })
            .collect::<darling::Result<HashMap<String, BootType>>>()
            .map(BootTypes)
    }
}

#[derive(Debug, FromMeta, Clone)]
pub struct BootType {
    pub c_type: Option<String>,
    pub rust_type: Option<RuntimeType>,
    pub hint: Option<String>,
    pub default: Option<NewValue>,
    #[darling(default)]
    pub do_not_convert: bool,
    #[darling(map="example_first")]
    pub example: Option<RuntimeType>,
}

#[derive(Debug, Clone)]
pub struct NewValue(pub Value);

impl FromMeta for NewValue {
    fn from_value(value: &syn::Lit) -> darling::Result<Self> {
        Ok(NewValue(match value {
            syn::Lit::Str(str) => Value::String(str.value()),
            syn::Lit::Int(int) => Value::Number(int.base10_parse::<i64>()?.into()),
            syn::Lit::Float(float) => Value::Number(Number::from_f64(float.base10_parse::<f64>()?).unwrap()),
            syn::Lit::Bool(bool) => Value::Bool(bool.value),
            _ => return Err(darling::Error::custom("unrecognized type"))
        }))
    }
}

/// retrieve the first child of the first node
fn example_first(v: Option<RuntimeType>) -> Option<RuntimeType> {
    if let Some(RuntimeType::Function { mut params, .. }) = v {
        Some(params.remove(0))
    } else {
        None
    }
}

impl FromMeta for RuntimeType {
    fn from_nested_meta(item: &NestedMeta) -> darling::Result<Self> {
        match item {
            NestedMeta::Meta(meta) => Self::from_meta(meta),
            NestedMeta::Lit(lit) => Self::from_value(lit),
        }
    }
    fn from_meta(item: &Meta) -> darling::Result<Self> {
        if let Meta::List(MetaList { path, nested, .. }) = item {
            Ok(RuntimeType::Function {
                function: path.get_ident()
                    .ok_or_else(|| darling::Error::custom("path must consist of a single ident"))?
                    .to_string(),
                params: nested
                    .into_iter()
                    .map(RuntimeType::from_nested_meta)
                    .collect::<darling::Result<Vec<RuntimeType>>>()?
            })
        } else {
            Err(darling::Error::custom("expected metalist"))
        }
    }
    fn from_value(value: &syn::Lit) -> darling::Result<Self> {
        if let syn::Lit::Str(value) = value {
            Ok(RuntimeType::Name(value.value()))
        } else {
            Err(darling::Error::custom("expected string"))
        }
    }
}
