use std::{collections::HashMap, env, path::PathBuf};

use darling::{Error, FromMeta, Result};
use proc_macro2::{Literal, Punct, Spacing, TokenStream, TokenTree};
use quote::format_ident;
use syn::{
    AttrStyle, Attribute, AttributeArgs, ItemFn, Lit, Meta, MetaNameValue, Path, PathSegment,
    ReturnType, Type, TypePath,
};

use crate::{
    proven::filesystem::{get_src_dir, make_proof_link},
    Deprecation,
};

use super::arguments::BootstrapArguments;

#[derive(Debug, Default)]
pub struct BootstrapDocstring {
    pub description: Option<String>,
    pub arguments: HashMap<String, String>,
    pub generics: HashMap<String, String>,
    pub returns: Option<String>,
    pub deprecated: Option<Deprecation>,
}

#[derive(Debug, FromMeta, Clone)]
pub struct DeprecationArguments {
    pub since: Option<String>,
    pub note: Option<String>,
}

impl BootstrapDocstring {
    pub fn from_attrs(
        name: &String,
        attrs: Vec<Attribute>,
        output: &ReturnType,
        path: Option<(&str, &str)>,
        features: Vec<String>,
    ) -> Result<BootstrapDocstring> {
        // look for this attr:
        // #[deprecated(note="please use `new_method` instead")]
        let deprecated = attrs
            .iter()
            .find(|attr| {
                attr.path.get_ident().map(ToString::to_string).as_deref() == Some("deprecated")
            })
            .map(|attr| {
                let meta = DeprecationArguments::from_meta(&attr.parse_meta()?)?;
                Result::Ok(Deprecation {
                    since: meta.since.ok_or_else(|| {
                        Error::custom("`since` must be specified").with_span(&attr)
                    })?,
                    note: meta.note.ok_or_else(|| {
                        Error::custom("`note` must be specified").with_span(&attr)
                    })?,
                })
            })
            .transpose()?;

        let mut doc_sections = parse_docstring_sections(attrs)?;

        const HONEST_SECTION: &str = "Why honest-but-curious?";
        const HONEST_FEATURE: &str = "honest-but-curious";
        let has_honest_section = doc_sections.keys().any(|key| key == HONEST_SECTION);
        let has_honest_feature = features
            .clone()
            .into_iter()
            .any(|feature| feature == HONEST_FEATURE);
        if has_honest_feature && !has_honest_section {
            let msg = format!(
                "{name} requires \"{HONEST_FEATURE}\" but is missing \"{HONEST_SECTION}\" section"
            );
            return Err(Error::custom(msg));
        }
        if has_honest_section && !has_honest_feature {
            let msg = format!(
                "{name} has \"{HONEST_SECTION}\" section but is missing \"{HONEST_FEATURE}\" feature"
            );
            return Err(Error::custom(msg));
        }

        if let Some(sup_elements) = parse_sig_output(output)? {
            doc_sections.insert("Supporting Elements".to_string(), sup_elements);
        }

        let mut description = Vec::from_iter(doc_sections.remove("Description"));

        if !features.is_empty() {
            let features_list = features
                .into_iter()
                .map(|f| format!("`{f}`"))
                .collect::<Vec<_>>()
                .join(", ");
            description.push(format!("\n\nRequired features: {features_list}"));
        }

        // add a link to rust documentation (with a gap line)
        if let Some((module, name)) = &path {
            description.push(String::new());
            description.push(make_rustdoc_link(module, name)?)
        }

        let mut add_section_to_description = |section_name: &str| {
            doc_sections.remove(section_name).map(|section| {
                description.push(format!("\n**{section_name}:**\n"));
                description.push(section)
            })
        };
        // can add more sections here...
        add_section_to_description(HONEST_SECTION);
        add_section_to_description("Citations");
        add_section_to_description("Supporting Elements");
        add_section_to_description("Proof Definition");

        Ok(BootstrapDocstring {
            description: if description.is_empty() {
                None
            } else {
                Some(description.join("\n").trim().to_string())
            },
            arguments: doc_sections
                .remove("Arguments")
                .map(parse_docstring_args)
                .unwrap_or_else(HashMap::new),
            generics: doc_sections
                .remove("Generics")
                .map(parse_docstring_args)
                .unwrap_or_else(HashMap::new),
            returns: doc_sections.remove("Returns"),
            deprecated,
        })
    }
}

/// Parses a section that is delimited by bullets into a hashmap.
///
/// The keys are the arg names and values are the descriptions.
///
/// # Example
///
/// ```text
/// # Arguments
/// * `a` - a description for argument a
/// * `b` - a description for argument b
///         ...multiple lines of description
/// * `c` - a description for argument c
/// ```
///
fn parse_docstring_args(args: String) -> HashMap<String, String> {
    // split by newlines
    let mut args = args
        .split("\n")
        .map(ToString::to_string)
        .collect::<Vec<_>>();

    // add a trailing delimiter so that we can use .windows
    args.push("* `".to_string());

    // find the row indexes where each argument starts
    (args.iter().enumerate())
        .filter_map(|(i, v)| v.starts_with("* `").then(|| i))
        .collect::<Vec<usize>>()
        // each window corresponds to the documentation for one argument
        .windows(2)
        .map(|window| {
            // split the variable name from the first line
            let mut splitter = args[window[0]].splitn(2, " - ").map(str::to_string);
            let name = splitter.next().unwrap();
            let name = name[3..name.len() - 1].to_string();

            // retrieve the rest of the first line, as well as any other lines, trim all, and join them together with newlines
            let description = vec![splitter.next().unwrap_or_else(String::new)]
                .into_iter()
                .chain(
                    args[window[0] + 1..window[1]]
                        .iter()
                        .map(|v| v.trim().to_string()),
                )
                .collect::<Vec<String>>()
                .join("\n")
                .trim()
                .to_string();
            (name, description)
        })
        .collect::<HashMap<String, String>>()
}

/// Break a vector of syn Attributes into a hashmap.
///
/// Keys represent section names, and values are the text under the section
fn parse_docstring_sections(attrs: Vec<Attribute>) -> Result<HashMap<String, String>> {
    let mut docstrings = (attrs.into_iter())
        .filter(|v| v.path.get_ident().map(ToString::to_string).as_deref() == Some("doc"))
        .map(parse_doc_attribute)
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .filter_map(|v| {
            if v.is_empty() {
                Some(String::new())
            } else {
                v.starts_with(" ").then(|| v[1..].to_string())
            }
        })
        .collect::<Vec<String>>();

    // wrap in headers to prepare for parsing
    docstrings.insert(0, "# Description".to_string());
    docstrings.push("# End".to_string());

    Ok(docstrings
        .iter()
        .enumerate()
        .filter_map(|(i, v)| v.starts_with("# ").then(|| i))
        .collect::<Vec<usize>>()
        .windows(2)
        .map(|window| {
            (
                docstrings[window[0]]
                    .strip_prefix("# ")
                    .expect("won't panic (because of filter)")
                    .to_string(),
                docstrings[window[0] + 1..window[1]]
                    .to_vec()
                    .join("\n")
                    .trim()
                    .to_string(),
            )
        })
        .collect())
}

/// Parses the return type into a markdown-formatted summary
fn parse_sig_output(output: &ReturnType) -> Result<Option<String>> {
    match output {
        ReturnType::Default => Ok(None),
        ReturnType::Type(_, ty) => parse_supporting_elements(&*ty),
    }
}

fn parse_supporting_elements(ty: &Type) -> Result<Option<String>> {
    let PathSegment { ident, arguments } = match &ty {
        syn::Type::Path(TypePath {
            path: Path { segments, .. },
            ..
        }) => segments.last().ok_or_else(|| {
            Error::custom("return type cannot be an empty path").with_span(&segments)
        })?,
        _ => return Ok(None),
    };

    match ident {
        i if i == "Fallible" => parse_supporting_elements(match arguments {
            syn::PathArguments::AngleBracketed(ab) => {
                if ab.args.len() != 1 {
                    return Err(Error::custom("Fallible needs one angle-bracketed argument")
                        .with_span(&ab.args));
                }
                match ab.args.first().expect("unreachable due to if statement") {
                    syn::GenericArgument::Type(ty) => ty,
                    arg => {
                        return Err(
                            Error::custom("argument to Fallible must to be a type").with_span(&arg)
                        )
                    }
                }
            }
            arg => {
                return Err(
                    Error::custom("Fallible needs an angle-bracketed argument").with_span(arg)
                )
            }
        }),
        i if i == "Transformation" || i == "Measurement" || i == "Function" => {
            match arguments {
                syn::PathArguments::AngleBracketed(ab) => {
                    let num_args = if i == "Function" { 2 } else { 4 };

                    if ab.args.len() != num_args {
                        return Err(Error::custom(format!(
                            "{i} needs {num_args} angle-bracketed arguments"
                        ))
                        .with_span(&ab.args));
                    }

                    let [input_domain, output_domain] = [&ab.args[0], &ab.args[1]];

                    // syn doesn't have a pretty printer but we don't need to add a dep...
                    let pprint = |ty| {
                        quote::quote!(#ty)
                            .to_string()
                            .replace(" ", "")
                            .replace(",", ", ")
                    };

                    let input_label = match i {
                        i if i == "Transformation" => "Domain:",
                        i if i == "Measurement" => "Domain:",
                        i if i == "Function" => "Type:  ",
                        _ => unreachable!(),
                    };

                    let output_label = match i {
                        i if i == "Transformation" => "Domain:",
                        i if i == "Measurement" => "Type:  ",
                        i if i == "Function" => "Type:  ",
                        _ => unreachable!(),
                    };

                    let mut lines = vec![
                        format!("* Input {}   `{}`", input_label, pprint(input_domain)),
                        format!("* Output {}  `{}`", output_label, pprint(output_domain)),
                    ];

                    if i != "Function" {
                        let output_distance = match i {
                            i if i == "Transformation" => "Metric: ",
                            i if i == "Measurement" => "Measure:",
                            _ => unreachable!(),
                        };
                        let [input_metric, output_metmeas] = [&ab.args[2], &ab.args[3]];
                        lines.extend([
                            format!("* Input Metric:   `{}`", pprint(input_metric)),
                            format!("* Output {} `{}`", output_distance, pprint(output_metmeas)),
                        ]);
                    }

                    Ok(Some(lines.join("\n")))
                }
                arg => {
                    return Err(
                        Error::custom("Fallible needs an angle-bracketed argument").with_span(arg)
                    )
                }
            }
        }
        _ => Ok(None),
    }
}

/// extract the string inside a doc comment attribute
fn parse_doc_attribute(attr: Attribute) -> Result<String> {
    match attr.parse_meta()? {
        Meta::NameValue(MetaNameValue {
            lit: Lit::Str(v), ..
        }) => Ok(v.value()),
        _ => Err(Error::custom("doc attribute must be a string literal").with_span(&attr)),
    }
}

/// Obtain a relative path to a proof, given all available information
pub fn get_proof_path(
    attr_args: &AttributeArgs,
    item_fn: &ItemFn,
    proof_paths: &HashMap<String, Option<String>>,
) -> Result<Option<String>> {
    let BootstrapArguments {
        name,
        proof_path,
        unproven,
        ..
    } = BootstrapArguments::from_attribute_args(&attr_args)?;

    let name = name.unwrap_or_else(|| item_fn.sig.ident.to_string());
    if unproven && proof_path.is_some() {
        return Err(Error::custom("proof_path is invalid when unproven"));
    }
    Ok(match proof_path {
        Some(proof_path) => Some(proof_path),
        None => match proof_paths.get(&name) {
            Some(None) => return Err(Error::custom(format!("more than one file named {name}.tex. Please specify `proof_path = \"{{module}}/path/to/proof.tex\"` in the macro arguments."))),
            Some(proof_path) => proof_path.clone(),
            None => None
        }
    })
}

/// add attributes containing the proof link
pub fn insert_proof_attribute(attributes: &mut Vec<Attribute>, proof_path: String) -> Result<()> {
    let source_dir = get_src_dir()?;
    let proof_path = PathBuf::from(proof_path);
    let repo_path = PathBuf::from("rust/src");
    let proof_link = format!(
        " [(Proof Document)]({}) ",
        make_proof_link(source_dir, proof_path, repo_path)?
    );

    let position = (attributes.iter())
        .position(|attr| {
            if attr.path.get_ident().map(ToString::to_string).as_deref() != Some("doc") {
                return false;
            }
            if let Ok(comment) = parse_doc_attribute(attr.clone()) {
                comment.starts_with(" # Proof Definition")
            } else {
                false
            }
        })
        // point to the next line after the header, if found
        .map(|i| i + 1)
        // insert a header to the end, if not found
        .unwrap_or_else(|| {
            attributes.push(new_comment_attribute(" "));
            attributes.push(new_comment_attribute(" # Proof Definition"));
            attributes.len()
        });

    attributes.insert(position, new_comment_attribute(&proof_link));

    Ok(())
}

/// construct an attribute representing a documentation comment
fn new_comment_attribute(comment: &str) -> Attribute {
    Attribute {
        pound_token: Default::default(),
        style: AttrStyle::Outer,
        bracket_token: Default::default(),
        path: Path::from(format_ident!("doc")),
        tokens: TokenStream::from_iter(
            [
                TokenTree::Punct(Punct::new('=', Spacing::Alone)),
                TokenTree::Literal(Literal::string(comment)),
            ]
            .into_iter(),
        ),
    }
}

pub fn make_rustdoc_link(module: &str, name: &str) -> Result<String> {
    // link from foreign library docs to rust docs
    let proof_uri = if let Ok(rustdoc_port) = std::env::var("OPENDP_RUSTDOC_PORT") {
        format!("http://localhost:{rustdoc_port}")
    } else {
        // find the docs uri
        let docs_uri =
            env::var("OPENDP_REMOTE_RUSTDOC_URI").unwrap_or_else(|_| "https://docs.rs".to_string());

        // find the version
        let mut version = env!("CARGO_PKG_VERSION");
        if version.ends_with("-dev") {
            version = "latest";
        };

        format!("{docs_uri}/opendp/{version}")
    };

    Ok(format!(
        // RST does not support nested markup, so do not try `{name}`!
        "[{name} in Rust documentation.]({proof_uri}/opendp/{module}/fn.{name}.html)"
    ))
}
