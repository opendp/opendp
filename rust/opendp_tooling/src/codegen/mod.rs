use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use crate::{Argument, TypeRecipe};

pub mod python;
pub mod r;

pub fn write_bindings(base_dir: PathBuf, files: HashMap<PathBuf, String>) {
    for (file_path, file_contents) in files {
        File::create(base_dir.join(file_path))
            .unwrap()
            .write_all(file_contents.as_ref())
            .unwrap();
    }
}

fn tab(number_of_spaces: usize, text: String) -> String {
    text.split('\n')
        .map(|v| {
            if v.is_empty() {
                String::new()
            } else {
                format!("{}{}", " ".repeat(number_of_spaces), v)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) fn tab_r(text: String) -> String {
    tab(2, text)
}

pub(crate) fn tab_py(text: String) -> String {
    tab(4, text)
}

pub(crate) fn tab_c(text: String) -> String {
    tab(4, text)
}

/// resolve references to derived types
fn flatten_type_recipe(type_recipe: &TypeRecipe, derived_types: &Vec<Argument>) -> TypeRecipe {
    let resolve_name = |name: &String| {
        derived_types
            .iter()
            .find(|derived| derived.name.as_ref().unwrap() == name)
            .map(|derived_type| {
                flatten_type_recipe(derived_type.rust_type.as_ref().unwrap(), derived_types)
            })
            .unwrap_or_else(|| type_recipe.clone())
    };

    match type_recipe {
        TypeRecipe::Name(name) => resolve_name(name),
        TypeRecipe::Nest { origin, args } => TypeRecipe::Nest {
            origin: origin.clone(),
            args: args
                .iter()
                .map(|arg| flatten_type_recipe(arg, derived_types))
                .collect(),
        },
        other => other.clone(),
    }
}

impl Argument {
    /// retrieve the python ctype corresponding to the type inside FfiResult<*>
    pub fn python_unwrapped_ctype(&self, typemap: &HashMap<String, String>) -> String {
        let c_type = self.c_type();
        assert_eq!(&c_type[..9], "FfiResult");
        typemap
            .get(&c_type[10..c_type.len() - 1])
            .expect(format!("unrecognized c_type: {c_type}").as_str())
            .clone()
    }
    /// retrieve the python ctypes corresponding to the origin of a type (subtypes/args omitted)
    pub fn python_origin_ctype(&self, typemap: &HashMap<String, String>) -> String {
        typemap.get(&self.c_type_origin()).cloned().expect(&format!(
            "ctype not recognized in typemap: {:?}",
            self.c_type_origin()
        ))
    }
    pub fn python_type_hint(&self, hierarchy: &HashMap<String, Vec<String>>) -> Option<String> {
        if self.hint.is_some() {
            return self.hint.clone();
        }
        if self.is_type {
            return Some("RuntimeTypeDescriptor".to_string());
        }
        if let Some(TypeRecipe::Nest { origin, args }) = &self.rust_type {
            if origin == "Tuple" {
                return Some(format!("tuple[{}]", vec!["Any"; args.len()].join(", ")));
            }
        }
        self.c_type.clone().and_then(|mut c_type| {
            if c_type.starts_with("FfiResult<") {
                c_type = c_type[10..c_type.len() - 1].to_string();
            }
            if c_type.ends_with("AnyTransformation *") {
                return Some("Transformation".to_string());
            }
            if c_type.ends_with("AnyMeasurement *") {
                return Some("Measurement".to_string());
            }
            if c_type.ends_with("AnyFunction *") {
                return Some("Function".to_string());
            }
            if c_type.ends_with("AnyDomain *") {
                return Some("Domain".to_string());
            }
            if c_type.ends_with("AnyMetric *") {
                return Some("Metric".to_string());
            }
            if c_type.ends_with("AnyMeasure *") {
                return Some("Measure".to_string());
            }
            if c_type.ends_with("AnyObject *") || c_type.ends_with("FfiSlice *") {
                // Returning "Any" doesn't strengthen type checking,
                // and sometimes seems odd in the docs.
                return None;
            }

            hierarchy
                .iter()
                .find(|(_k, members)| members.contains(&c_type))
                .and_then(|(k, _)| {
                    Some(match k.as_str() {
                        k if k == "integer" => "int",
                        k if k == "float" => "float",
                        k if k == "string" => "str",
                        k if k == "bool" => "bool",
                        _ => return None,
                    })
                })
                .map(|v| v.to_string())
        })
    }
}

impl Argument {
    pub fn name(&self) -> String {
        self.name
            .clone()
            .expect("unknown name when parsing argument")
    }
    pub fn c_type(&self) -> String {
        if self.is_type {
            return "char *".to_string();
        }
        self.c_type
            .clone()
            .expect("unknown c_type when parsing argument")
    }
    pub fn c_type_origin(&self) -> String {
        self.c_type().split('<').next().unwrap().to_string()
    }
}

#[cfg(test)]
mod test;
