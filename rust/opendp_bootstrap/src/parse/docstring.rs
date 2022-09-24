use std::{collections::HashMap, env, ffi::OsStr, path::PathBuf};

use darling::{Error, Result};
use syn::{Attribute, Lit, Meta, MetaNameValue, Path, PathSegment, ReturnType, Type, TypePath};

#[derive(Debug, Default)]
pub struct Docstring {
    pub description: Vec<String>,
    pub arguments: HashMap<String, Vec<String>>,
    pub generics: HashMap<String, Vec<String>>,
    pub ret: Vec<String>,
}

impl Docstring {
    pub fn from_attrs(attrs: Vec<Attribute>, output: &ReturnType) -> Result<Docstring> {
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

        Ok(Docstring {
            description,
            arguments: doc_sections
                .remove("Arguments")
                .map(parse_docstring_args)
                .unwrap_or_else(HashMap::new),
            generics: doc_sections
                .remove("Generics")
                .map(parse_docstring_args)
                .unwrap_or_else(HashMap::new),
            ret: doc_sections.remove("Returns").unwrap_or_else(Vec::new),
        })
    }
}

fn parse_docstring_args(mut args: Vec<String>) -> HashMap<String, Vec<String>> {
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
                .collect::<Vec<String>>();
            (name, description)
        })
        .collect::<HashMap<String, Vec<String>>>()
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

pub fn find_relative_proof_path(func_name: &str) -> Option<String> {
    let src_dir = get_src_dir();

    fn find_absolute_path(
        file_name: &OsStr,
        dir: &std::path::Path,
    ) -> std::io::Result<Option<PathBuf>> {
        let mut matches = Vec::new();
        if dir.is_dir() {
            for entry in std::fs::read_dir(dir)? {
                let path = entry?.path();
                if path.is_dir() {
                    matches.extend(find_absolute_path(file_name, &path)?);
                } else {
                    if path.file_name() == Some(file_name) {
                        matches.push(path);
                    }
                };
            }
        }
        if matches.len() > 1 {
            panic!("multiple matching proofs found. Please specify `proof = \"{{module}}/path/to/proof\"` in the bootstrap attributes.")
        }
        Ok(matches.get(0).cloned())
    }

    find_absolute_path(&OsStr::new(format!("{func_name}.tex").as_str()), &src_dir)
        .expect("failed to read crate source")
        // turn into relative PathBuf
        .map(|pb| {
            pb.strip_prefix(&src_dir)
                .expect("failed to strip src_dir from proof path")
                .to_path_buf()
                .to_str()
                .expect("relative proof path is empty")
                .to_string()
        })
}

pub fn get_src_dir() -> PathBuf {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
        .expect("Failed to determine location of Cargo.toml.");
    PathBuf::from(manifest_dir).join("src")
}

pub fn make_proof_link(relative_path: PathBuf) -> String {
    // construct absolute path
    let absolute_path = get_src_dir().join(&relative_path);

    assert!(
        absolute_path.exists(),
        "{:?} does not exist!",
        absolute_path
    );

    let target = if cfg!(feature = "local") {
        absolute_path
            .to_str()
            .expect("absolute path is empty")
            .to_string()
    } else {
        format!(
            "https://docs.opendp.org/en/{version}/proofs/{relative_path}",
            version = get_version(),
            relative_path = relative_path.display()
        )
    };

    format!("[Link to proof.]({})", target)
}

fn get_version() -> String {
    let version = env::var("CARGO_PKG_VERSION")
        .expect("CARGO_PKG_VERSION must be set")
        .to_string();
    if version == "0.0.0+development" {
        "latest".to_string()
    } else {
        version
    }
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
                    return Err(Error::custom("Fallible needs one angle-bracketed argument").with_span(&ab.args))
                }
                match ab.args.first().unwrap() {
                    syn::GenericArgument::Type(ty) => ty,
                    arg => return Err(Error::custom("argument to Fallible must to be a type").with_span(&arg))
                }

            },
            arg => return Err(Error::custom("Fallible needs an angle-bracketed argument").with_span(arg)),
        }),
        i if i == "Transformation" || i == "Measurement" => {
            match arguments {
                syn::PathArguments::AngleBracketed(ab) => {
                    if ab.args.len() != 4 {
                        return Err(Error::custom(format!("{i} needs four angle-bracketed arguments")).with_span(&ab.args))
                    }
                    let [input_domain, output_domain, input_metric, output_metmeas] = <[_; 4]>::try_from(ab.args.iter().collect::<Vec<_>>())
                        .map_err(|_| Error::custom(format!("{i} needs four angle-bracketed arguments")).with_span(&ab.args))?;
                    
                    let output_distance = match i {
                        i if i == "Transformation" => "Metric: ",
                        i if i == "Measurement" => "Measure:",
                        _ => unreachable!()
                    };

                    // syn doesn't have a pretty printer but we don't need to add a dep...
                    let pprint = |ty| quote::quote!(#ty).to_string().replace(" ", "").replace(",", ", ");

                    Ok(Some(vec![
                        format!("* Input Domain:   `{}`", pprint(input_domain)),
                        format!("* Output Domain:  `{}`", pprint(output_domain)),
                        format!("* Input Metric:   `{}`", pprint(input_metric)),
                        format!("* Output {} `{}`", output_distance, pprint(output_metmeas)),
                    ]))
                },
                arg => return Err(Error::custom("Fallible needs an angle-bracketed argument").with_span(arg)),
            }
        }
        _ => Ok(None)
    }
}
