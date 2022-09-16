use std::env;
use std::fs::{File, canonicalize};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use indexmap::map::IndexMap;
use serde_json::Value;

pub mod python;

fn main() {
    // only build the bindings if you're in dev mode
    if env::var("CARGO_PKG_VERSION").unwrap().as_str() != "0.0.0+development" { return }

    let module_names = vec!["combinators", "measurements", "transformations", "data", "core", "accuracy"];

    let get_bootstrap_path = |val: &str|
        Path::new("src").join(val).join("bootstrap.json");

    // Tell Cargo that if the given file changes, to rerun this build script.
    module_names.iter().for_each(|module_name|
        println!("cargo:rerun-if-changed={:?}", get_bootstrap_path(module_name)));

    // allow modules to be unused if no bindings feature flags are enabled
    let _modules = module_names.iter()
        .map(|module_name| {
            println!("parsing module: {}", module_name);
            let mut contents = String::new();
            File::open(get_bootstrap_path(module_name))
                .expect("file not found")
                .read_to_string(&mut contents)
                .expect("failed reading module json");

            (module_name.to_string(), serde_json::from_str(&contents).expect("failed to parse json"))
        })
        .collect::<IndexMap<String, Module>>();

    if cfg!(feature="bindings-python") {
        write_bindings(python::generate_bindings(_modules));
    }
}

#[allow(dead_code)]
fn write_bindings(files: IndexMap<PathBuf, String>) {
    let base_dir = canonicalize("../python/src/opendp").unwrap();
    for (file_path, file_contents) in files {
        File::create(base_dir.join(file_path)).unwrap()
            .write_all(file_contents.as_ref()).unwrap();
    }
}


#[allow(dead_code)]
fn indent(text: String) -> String {
    text.split('\n').map(|v| format!("    {}", v)).collect::<Vec<_>>().join("\n")
}

/// resolve references to derived types
#[allow(dead_code)]
fn flatten_runtime_type(runtime_type: &RuntimeType, derived_types: &Vec<Argument>) -> RuntimeType {
    let resolve_name = |name: &String|
        derived_types.iter()
            .find(|derived| derived.name.as_ref().unwrap() == name)
            .map(|derived_type| flatten_runtime_type(
                derived_type.rust_type.as_ref().unwrap(), derived_types))
            .unwrap_or_else(|| runtime_type.clone());

    match runtime_type {
        RuntimeType::Name(name) =>
            resolve_name(name),
        RuntimeType::Lower { root, index } =>
            RuntimeType::Lower {
                root: Box::new(flatten_runtime_type(root, derived_types)),
                index: *index
            },
        RuntimeType::Raise { origin, args } =>
            RuntimeType::Raise {
                origin: origin.clone(),
                args: args.iter().map(|arg|
                    flatten_runtime_type(arg, derived_types)).collect()
            },
        other => other.clone()
    }
}
