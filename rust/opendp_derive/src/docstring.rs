use std::collections::HashMap;

use syn::{Attribute, Path, Meta, Lit};
use crate::extract;
use quote::quote;
use darling::Result;

pub(crate) fn path_to_str(path: Path) -> Result<String> {
    path.get_ident()
        .ok_or_else(|| darling::Error::custom(format!("Path must be of length 1! {:?}", quote!(#path).to_string())))
        .map(ToString::to_string)
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
        .filter(|v| path_to_str(v.path.clone()).ok().as_deref() == Some("doc"))
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
        generics: doc_sections.remove("Generics")
            .map(parse_doc_comment_args).unwrap_or_else(HashMap::new),
        ret: doc_sections.remove("Return").unwrap_or_else(Vec::new)
    }
}