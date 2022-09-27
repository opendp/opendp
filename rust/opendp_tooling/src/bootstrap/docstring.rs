use std::collections::HashMap;

use darling::{Error, Result};
use syn::{Attribute, Lit, Meta, MetaNameValue, Path, PathSegment, ReturnType, Type, TypePath};

#[derive(Debug, Default)]
pub struct BootstrapDocstring {
    pub description: Option<String>,
    pub arguments: HashMap<String, String>,
    pub generics: HashMap<String, String>,
    pub returns: Option<String>,
}

impl BootstrapDocstring {
    pub fn from_attrs(attrs: Vec<Attribute>, output: &ReturnType) -> Result<BootstrapDocstring> {
        let mut doc_sections = parse_docstring_sections(attrs)?;

        if let Some(sup_elements) = parse_sig_output(output)? {
            doc_sections.insert("Supporting Elements".to_string(), sup_elements);
        }

        let mut description = doc_sections.remove("Description").unwrap_or_else(Vec::new);

        let mut insert_section = |section_name: &str| {
            doc_sections.remove(section_name).map(|section| {
                description.push(format!("\n**{section_name}:**\n"));
                description.extend(section)
            })
        };
        // can add more sections here...
        insert_section("Citations");
        insert_section("Supporting Elements");

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
            returns: doc_sections
                .remove("Returns")
                .map(|sec| sec.join("\n").trim().to_string()),
        })
    }
}

fn parse_docstring_args(mut args: Vec<String>) -> HashMap<String, String> {
    args.push("* `".to_string());
    args.iter()
        .enumerate()
        .filter_map(|(i, v)| v.starts_with("* `").then(|| i))
        .collect::<Vec<usize>>()
        .windows(2)
        .map(|window| {
            let mut splitter = args[window[0]].splitn(2, " - ").map(str::to_string);
            let name = splitter.next().unwrap();
            let name = name[3..name.len() - 1].to_string();
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

fn parse_docstring_sections(attrs: Vec<Attribute>) -> Result<HashMap<String, Vec<String>>> {
    let mut docstrings = (attrs.into_iter())
        .filter(|v| v.path.get_ident().map(ToString::to_string).as_deref() == Some("doc"))
        .map(|v| {
            if let Meta::NameValue(MetaNameValue {
                lit: Lit::Str(v), ..
            }) = v.parse_meta()?
            {
                Ok(v.value())
            } else {
                Err(Error::custom("doc attribute must have string literal").with_span(&v))
            }
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .filter_map(|v| v.starts_with(" ").then(|| v[1..].to_string()))
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
                docstrings[window[0] + 1..window[1]].to_vec(),
            )
        })
        .collect::<HashMap<String, Vec<String>>>())
}

fn parse_sig_output(output: &ReturnType) -> Result<Option<Vec<String>>> {
    match output {
        ReturnType::Default => Ok(None),
        ReturnType::Type(_, ty) => parse_supporting_elements(&*ty),
    }
}

fn parse_supporting_elements(ty: &Type) -> Result<Option<Vec<String>>> {
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
        i if i == "Transformation" || i == "Measurement" => {
            match arguments {
                syn::PathArguments::AngleBracketed(ab) => {
                    if ab.args.len() != 4 {
                        return Err(Error::custom(format!(
                            "{i} needs four angle-bracketed arguments"
                        ))
                        .with_span(&ab.args));
                    }
                    let [input_domain, output_domain, input_metric, output_metmeas] =
                        <[_; 4]>::try_from(ab.args.iter().collect::<Vec<_>>()).map_err(|_| {
                            Error::custom(format!("{i} needs four angle-bracketed arguments"))
                                .with_span(&ab.args)
                        })?;

                    let output_distance = match i {
                        i if i == "Transformation" => "Metric: ",
                        i if i == "Measurement" => "Measure:",
                        _ => unreachable!(),
                    };

                    // syn doesn't have a pretty printer but we don't need to add a dep...
                    let pprint = |ty| {
                        quote::quote!(#ty)
                            .to_string()
                            .replace(" ", "")
                            .replace(",", ", ")
                    };

                    Ok(Some(vec![
                        format!("* Input Domain:   `{}`", pprint(input_domain)),
                        format!("* Output Domain:  `{}`", pprint(output_domain)),
                        format!("* Input Metric:   `{}`", pprint(input_metric)),
                        format!("* Output {} `{}`", output_distance, pprint(output_metmeas)),
                    ]))
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
