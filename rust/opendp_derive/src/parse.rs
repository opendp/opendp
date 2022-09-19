use std::collections::{HashMap, HashSet};

use darling::FromMeta;
use serde_json::{Number, Value};
use syn::{parse, GenericArgument, Lit, Meta, MetaList, NestedMeta, Path, Type};

use crate::extract;
use opendp_pre_derive::RuntimeType;

#[derive(Debug, FromMeta, Clone)]
pub(crate) struct Bootstrap {
    pub proof: Option<String>,
    pub module: String,
    pub features: Features,
    pub generics: BootTypes,
    pub arguments: BootTypes,
    pub derived_types: Option<DerivedTypes>,
    pub ret: Option<BootType>,
}

#[derive(Debug, Clone)]
pub struct DerivedTypes(pub HashMap<String, NewRuntimeType>);

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

                    let type_ = NewRuntimeType::from_nested_meta(&nested.first().expect("nested must have length at least 1"))?;
                    Ok((type_name, type_))
                } else {
                    Err(darling::Error::custom("expected metalist"))
                }
            })
            .collect::<darling::Result<HashMap<String, NewRuntimeType>>>()
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
pub(crate) struct BootTypes(pub HashMap<String, BootType>);

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
                    let type_ =
                        BootType::from_list(&nested.into_iter().cloned().collect::<Vec<_>>())?;
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
pub(crate) struct BootType {
    pub c_type: Option<String>,
    pub rust_type: Option<NewRuntimeType>,
    pub generics: Option<RuntimeTypeGenerics>,
    pub hint: Option<String>,
    pub default: Option<NewValue>,
    #[darling(default)]
    pub do_not_convert: bool,
    #[darling(map = "runtimetype_first")]
    pub example: Option<NewRuntimeType>,
}

#[derive(Debug, Clone)]
pub struct NewRuntimeType(pub RuntimeType);

#[derive(Debug, Clone)]
pub struct NewValue(pub Value);

impl From<RuntimeType> for NewRuntimeType {
    fn from(v: RuntimeType) -> Self {
        NewRuntimeType(v)
    }
}

impl FromMeta for NewValue {
    fn from_value(value: &syn::Lit) -> darling::Result<Self> {
        Ok(NewValue(match value {
            syn::Lit::Str(str) => Value::String(str.value()),
            syn::Lit::Int(int) => Value::Number(int.base10_parse::<i64>()?.into()),
            syn::Lit::Float(float) => {
                Value::Number(Number::from_f64(float.base10_parse::<f64>()?).expect("failed to parse f64"))
            }
            syn::Lit::Bool(bool) => Value::Bool(bool.value),
            _ => return Err(darling::Error::custom("unrecognized type")),
        }))
    }
}

/// retrieve the first child of the first node
fn runtimetype_first(v: Option<NewRuntimeType>) -> Option<NewRuntimeType> {
    Some(match v?.0 {
        RuntimeType::Function { mut params, .. } => params.remove(0),
        RuntimeType::Raise { mut args, .. } => args.remove(0),
        _ => unimplemented!(),
    }.into())
}

impl FromMeta for NewRuntimeType {
    fn from_nested_meta(item: &NestedMeta) -> darling::Result<Self> {
        match item {
            NestedMeta::Meta(meta) => Self::from_meta(meta),
            NestedMeta::Lit(lit) => Self::from_value(lit),
        }
    }
    fn from_meta(item: &Meta) -> darling::Result<Self> {
        fn path_to_rtype(path: Path) -> RuntimeType {
            let segment = path.segments.last().expect("no last segment");
            match segment.arguments.clone() {
                syn::PathArguments::None => RuntimeType::Name(segment.ident.to_string()),
                syn::PathArguments::AngleBracketed(angle_bracketed) => RuntimeType::Raise {
                    origin: segment.ident.to_string(),
                    args: angle_bracketed
                        .args
                        .into_iter()
                        .map(|arg| extract!(arg, GenericArgument::Type(v) => v))
                        .map(type_to_rtype)
                        .collect(),
                },
                syn::PathArguments::Parenthesized(_) => panic!("parenthesized paths are not supported"),
            }
        }

        fn type_to_rtype(type_: Type) -> RuntimeType {
            match type_ {
                Type::Path(path) => path_to_rtype(path.path),
                Type::Tuple(tuple) => RuntimeType::Raise {
                    origin: String::from("Tuple"),
                    args: tuple.elems.into_iter().map(type_to_rtype).collect(),
                },
                _ => panic!("unexpected type_ variant {:?}", type_),
            }
        }

        if let Meta::List(MetaList { path, nested, .. }) = item {
            let (name_values, nested): (Vec<NestedMeta>, Vec<NestedMeta>) = nested
                .iter()
                .cloned()
                .partition(|nm| matches!(nm, NestedMeta::Meta(Meta::NameValue(_))));

            let id_lit = (name_values.iter())
                .map(|nv| extract!(nv, NestedMeta::Meta(Meta::NameValue(nv)) => nv))
                .find(|nv| {
                    nv.path
                        .get_ident()
                        .map(|nv| nv == "id")
                        .unwrap_or(false)
                })
                .map(|nv| nv.lit.clone());

            Ok(if let Some(id_lit) = id_lit {
                let ts: proc_macro::TokenStream = extract!(id_lit, Lit::Str(litstr) => litstr.value().parse()
                    .map_err(|e| darling::Error::custom(format!("syn error: {:?}", e)))?);
                type_to_rtype(parse::<Type>(ts)?).into()
            } else {
                RuntimeType::Function {
                    function: (path.get_ident())
                        .ok_or_else(|| {
                            darling::Error::custom("path must consist of a single ident")
                        })?
                        .to_string(),
                    params: (nested.iter())
                        .map(NewRuntimeType::from_nested_meta)
                        .map(|nrt| nrt.map(|nrt| nrt.0))
                        .collect::<darling::Result<Vec<RuntimeType>>>()?,
                }.into()
            })
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

#[derive(Debug, Clone)]
pub(crate) struct RuntimeTypeGenerics(pub HashSet<String>);

impl FromMeta for RuntimeTypeGenerics {
    fn from_list(items: &[NestedMeta]) -> darling::Result<Self> {
        items.iter()
            .map(|item| extract!(item, NestedMeta::Lit(lit) => lit))
            .map(|lit| if let syn::Lit::Str(value) = lit {
                Ok(value.value())
            } else {
                Err(darling::Error::custom("expected string"))
            })
            .collect::<darling::Result<_>>().map(RuntimeTypeGenerics)
    }
}