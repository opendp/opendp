use std::env;
use std::fs::{canonicalize, File};
use std::io::{Read, Write};
use std::path::PathBuf;

use indexmap::map::IndexMap;
use opendp_pre_derive::{target::find_target_dir, Argument, Function, RuntimeType};
pub(crate) type Module = IndexMap<String, Function>;

pub mod python;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR must be set."));
    let target_dir = find_target_dir(&out_dir);
    let json_dir = target_dir.join("opendp_bootstrap");

    dbg!(&json_dir);

    // allow modules to be unused if no bindings feature flags are enabled
    let _modules = std::fs::read_dir(json_dir)
        .expect("failed to read json_dir")
        .map(|mod_entry| {
            let mod_dir = mod_entry
                .expect("failed to read module folder filesystem entry")
                .path();

            let module_name = (&mod_dir)
                .file_name()
                .expect("module name must be non-empty")
                .to_str()
                .expect("module name must be valid unicode")
                .to_string();

            let module_fns = std::fs::read_dir(&mod_dir)
                .expect("failed to read mod_dir")
                .map(|fn_entry| {
                    let fn_path = fn_entry
                        .expect("failed to read module folder filesystem entry")
                        .path();

                    let fn_name = (&fn_path)
                        .file_stem()
                        .expect("function name must be non-empty")
                        .to_str()
                        .expect("module name must be valid unicode")
                        .to_string();

                    let mut contents = String::new();
                    File::open(fn_path)
                        .expect("fn not found")
                        .read_to_string(&mut contents)
                        .expect("failed reading function json");
                    let fn_body = serde_json::from_str(&contents).expect("failed to parse json");
                    (fn_name, fn_body)
                })
                .collect();

            (module_name, module_fns)
        })
        .collect();

    if cfg!(feature = "bindings-python") {
        write_bindings(python::generate_bindings(_modules));
    }
}

#[allow(dead_code)]
fn write_bindings(files: IndexMap<PathBuf, String>) {
    let base_dir = canonicalize("../../python/src/opendp").unwrap();
    for (file_path, file_contents) in files {
        File::create(base_dir.join(file_path))
            .unwrap()
            .write_all(file_contents.as_ref())
            .unwrap();
    }
}

#[allow(dead_code)]
fn indent(text: String) -> String {
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
        RuntimeType::Lower { root, index } => RuntimeType::Lower {
            root: Box::new(flatten_runtime_type(root, derived_types)),
            index: *index,
        },
        RuntimeType::Raise { origin, args } => RuntimeType::Raise {
            origin: origin.clone(),
            args: args
                .iter()
                .map(|arg| flatten_runtime_type(arg, derived_types))
                .collect(),
        },
        other => other.clone(),
    }
}
