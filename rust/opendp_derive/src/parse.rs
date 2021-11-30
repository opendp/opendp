use std::collections::HashMap;
use std::default::Default;

use quote::quote;
use syn::{Attribute, AttributeArgs, FnArg, GenericParam, Lit, Meta, NestedMeta, Pat, Path, ReturnType, Signature, TypeParam};

use crate::{Argument, Dispatch, extract, Function, Generic};

pub(crate) fn path_to_str(path: Path) -> String {
    if path.segments.len() != 1 { panic!("Path must be of length 1! {:?}", quote!(#path)) }
    path.segments[0].ident.to_string()
}

#[derive(Default)]
pub(crate) struct MacroConfig {
    pub module: Vec<String>,
    pub features: Vec<String>,
    pub arguments: HashMap<String, Vec<NestedMeta>>,
    pub generics: HashMap<String, Vec<NestedMeta>>,
    pub dispatch: Vec<Dispatch>,
}

pub(crate) fn parse_macro_config(attribute_args: AttributeArgs) -> MacroConfig {
    let mut dispatch = <Vec<Dispatch>>::new();
    let mut macro_args = HashMap::new();

    attribute_args.into_iter()
        .map(|v| extract!(v, NestedMeta::Meta(v) => v))
        .map(|v| extract!(v, Meta::List(v) => v))
        .for_each(|v| {
            let key = path_to_str(v.path);
            if &*key == "dispatch" {
                let mut iter = v.nested.iter();
                dispatch.push(Dispatch {
                    cond: v.nested.first()
                        .and_then(|first| match first {
                            NestedMeta::Lit(Lit::Str(v)) => Some(v.value()),
                            _ => None
                        })
                        .map(|first| {
                            iter.next();
                            first
                        }),
                    prod: iter
                        .map(|v| extract!(v, NestedMeta::Meta(v) => v))
                        .map(|v| extract!(v.clone(), Meta::NameValue(v) =>
                            (path_to_str(v.path), extract!(v.lit, Lit::Str(v) => v.value()))))
                        .collect(),
                })
            } else {
                macro_args.insert(key, v.nested.into_iter().collect());
            }
        });

    let extract_strings = |data: Vec<NestedMeta>| data.into_iter()
        .map(|v| extract!(v, NestedMeta::Lit(v) => v))
        .map(|v| extract!(v, Lit::Str(v) => v.value()))
        .collect();

    MacroConfig {
        module: extract_strings(macro_args.remove("module")
            .expect("module is a required argument")),
        features: macro_args.remove("features")
            .map(extract_strings).unwrap_or_else(Vec::new),
        arguments: macro_args.remove("arguments").map(|data| data.into_iter()
            .map(|v| extract!(v, NestedMeta::Meta(v) => v))
            .map(|v| extract!(v, Meta::List(v) => v))
            .map(|l| (path_to_str(l.path), l.nested.into_iter().collect()))
            .collect()).unwrap_or_else(HashMap::new),
        generics: macro_args.remove("types").map(|data| data.into_iter()
            .map(|v| extract!(v, NestedMeta::Meta(v) => v))
            .map(|v| extract!(v, Meta::List(v) => v))
            .map(|l| (path_to_str(l.path), l.nested.into_iter().collect()))
            .collect()).unwrap_or_else(HashMap::new),
        dispatch,
    }
}

fn parse_doc_comment_args(mut args: Vec<String>) -> HashMap<String, Vec<String>> {
    args.push("* `".to_string());
    args.iter().enumerate()
        .filter_map(|(i, v)| v.starts_with("* `").then(|| i))
        .collect::<Vec<usize>>()
        .windows(2)
        .map(|window| {
            let mut splitter = args[window[0]].splitn(2, " - ").map(str::to_string);
            let name = splitter.next().unwrap();
            let name = name[3..name.len() - 1].to_string();
            let description = vec![splitter.next().unwrap_or_else(String::new)].into_iter()
                .chain(args[window[0] + 1..window[1]].iter().map(|v| v.trim().to_string()))
                .collect::<Vec<String>>();
            (name, description)
        })
        .collect::<HashMap<String, Vec<String>>>()
}

fn parse_doc_comment_sections(attrs: Vec<Attribute>) -> HashMap<String, Vec<String>> {
    let mut docstrings = attrs.into_iter()
        .filter(|v| path_to_str(v.path.clone()) == "doc".to_string())
        .map(|v| v.parse_meta().unwrap())
        .map(|v| extract!(v, Meta::NameValue(v) => v.lit))
        .map(|v| extract!(v, Lit::Str(v) => v.value()))
        .filter_map(|v| v.starts_with(" ").then(|| v[1..].to_string()))
        .collect::<Vec<String>>();

    // wrap in headers to prepare for parsing
    docstrings.insert(0, "# Description".to_string());
    docstrings.push("# End".to_string());

    docstrings.iter().enumerate()
        .filter_map(|(i, v)| v.starts_with("# ").then(|| i))
        .collect::<Vec<usize>>()
        .windows(2)
        .map(|window| (
            docstrings[window[0]].strip_prefix("# ").unwrap().to_string(),
            docstrings[window[0] + 1..window[1]].to_vec()))
        .collect::<HashMap<String, Vec<String>>>()
}

#[derive(Debug, Default)]
pub(crate) struct DocComments {
    pub description: Vec<String>,
    pub arguments: HashMap<String, Vec<String>>,
    pub generics: HashMap<String, Vec<String>>,
    pub ret: Vec<String>
}

pub(crate) fn parse_doc_comments(attrs: Vec<Attribute>) -> DocComments {
    let mut doc_sections = parse_doc_comment_sections(attrs);

    DocComments {
        description: doc_sections.remove("Description").unwrap_or_else(Vec::new),
        arguments: doc_sections.remove("Arguments")
            .map(parse_doc_comment_args).unwrap_or_else(HashMap::new),
        generics: doc_sections.remove("Type Arguments")
            .map(parse_doc_comment_args).unwrap_or_else(HashMap::new),
        ret: doc_sections.remove("Return").unwrap_or_else(Vec::new)
    }
}

pub(crate) fn normalize_function(sig: Signature, mut doc_comments: DocComments, mut config: MacroConfig) -> Function {
    Function {
        name: sig.ident.to_string(),
        module: config.module,
        features: config.features,
        description: doc_comments.description,

        arguments: sig.inputs.into_iter()
            .map(|v| extract!(v, FnArg::Typed(v) => v))
            .map(|v| (extract!(*v.pat, Pat::Ident(v) => v.ident.to_string()), *v.ty))
            .map(|(name, ty)| Argument {
                description: doc_comments.arguments.remove(&name).unwrap_or_else(Vec::new),
                meta: config.arguments.remove(&name).unwrap_or_else(Vec::new),
                rust_type: ty,
                name: Some(name),
            })
            .collect(),
        generics: sig.generics.params.into_iter()
            .map(|v| extract!(v, GenericParam::Type(v) => v))
            .map(|v: TypeParam| {
                let name = v.ident.to_string();
                Generic {
                    description: doc_comments.generics.remove(&name).unwrap_or_else(Vec::new),
                    meta: config.generics.remove(&name).unwrap_or_else(Vec::new),
                    bounds: v.bounds,
                    name,
                }
            })
            .collect(),
        ret: Argument {
            name: None,
            rust_type: *extract!(sig.output, ReturnType::Type(_, v) => v),
            description: doc_comments.ret,
            meta: Vec::new()
        },

        // TODO: derive derived_types from arguments
        derived_types: vec![],
        where_clause: sig.generics.where_clause,
        dispatch: config.dispatch
    }
}

