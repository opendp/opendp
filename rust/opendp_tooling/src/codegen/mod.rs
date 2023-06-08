use std::collections::HashMap;
use std::fs::{canonicalize, File};
use std::io::Write;
use std::path::PathBuf;

use crate::{Argument, TypeRecipe};

pub mod python;

#[allow(dead_code)]
pub fn write_bindings(files: HashMap<PathBuf, String>) {
    let base_dir = canonicalize("../python/src/opendp").unwrap();
    for (file_path, file_contents) in files {
        File::create(base_dir.join(file_path))
            .unwrap()
            .write_all(file_contents.as_ref())
            .unwrap();
    }
}

#[allow(dead_code)]
pub(crate) fn indent(text: String) -> String {
    text.split('\n')
        .map(|v| format!("    {}", v))
        .collect::<Vec<_>>()
        .join("\n")
}

/// resolve references to derived types
#[allow(dead_code)]
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
                return Some(format!("Tuple[{}]", vec!["Any"; args.len()].join(", ")));
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
            if c_type.ends_with("AnyOdometer *") {
                return Some("Odometer".to_string());
            }
            if c_type.ends_with("AnyFunction *") {
                return Some("Function".to_string());
            }
            if c_type.ends_with("AnyObject *") {
                // py_to_object converts Any to AnyObjectPtr
                return Some("Any".to_string());
            }
            if c_type.ends_with("FfiSlice *") {
                // py_to_c converts Any to FfiSlicePtr
                return Some("Any".to_string());
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

impl TypeRecipe {
    /// translate the abstract derived_types info into python RuntimeType constructors
    pub fn to_python(&self) -> String {
        match self {
            Self::Name(name) => name.clone(),
            Self::Function { function, params } => format!(
                "{function}({params})",
                function = function,
                params = params
                    .iter()
                    .map(|v| v.to_python())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::Nest { origin, args } => format!(
                "RuntimeType(origin='{origin}', args=[{args}])",
                origin = origin,
                args = args
                    .iter()
                    .map(|arg| arg.to_python())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::None => "None".to_string(),
        }
    }
}
