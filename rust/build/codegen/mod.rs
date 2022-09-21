use std::collections::HashMap;
use std::fs::{canonicalize, File};
use std::io::Write;
use std::path::PathBuf;

use opendp_bootstrap::{Argument, RuntimeType};

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
fn flatten_runtime_type(runtime_type: &RuntimeType, derived_types: &Vec<Argument>) -> RuntimeType {
    let resolve_name = |name: &String| {
        derived_types
            .iter()
            .find(|derived| derived.name.as_ref().unwrap() == name)
            .map(|derived_type| {
                flatten_runtime_type(derived_type.rust_type.as_ref().unwrap(), derived_types)
            })
            .unwrap_or_else(|| runtime_type.clone())
    };

    match runtime_type {
        RuntimeType::Name(name) => resolve_name(name),
        RuntimeType::Nest { origin, args } => RuntimeType::Nest {
            origin: origin.clone(),
            args: args
                .iter()
                .map(|arg| flatten_runtime_type(arg, derived_types))
                .collect(),
        },
        other => other.clone(),
    }
}
