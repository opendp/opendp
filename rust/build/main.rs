use std::env;
use std::fs::{File, canonicalize};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use indexmap::map::IndexMap;
use serde::{Deserialize, Deserializer};
use serde_json::Value;

pub mod python;

// a module contains functions by name
type Module = IndexMap<String, Function>;

// metadata for each function in a module
#[derive(Deserialize, Debug)]
pub struct Function {
    #[serde(default)]
    args: Vec<Argument>,
    // metadata for return type
    #[serde(default)]
    ret: Argument,
    // required feature flags to execute function
    #[serde(default)]
    features: Vec<String>,
    // metadata for constructing new types based on existing types or introspection
    #[serde(default)]
    derived_types: Vec<Argument>,
    // plaintext description of the function used to generate documentation
    description: Option<String>,
    // URL pointing to the location of the DP proof for the function
    proof: Option<String>
}

// Metadata for function arguments, derived types and returns.
#[derive(Deserialize, Debug, Default, Clone)]
pub struct Argument {
    // argument name. Optional for return types
    name: Option<String>,
    // c type to translate to/from for FFI. Optional for derived types
    c_type: Option<String>,
    // RuntimeType expressed in terms of rust types with generics.
    // Includes various RuntimeType constructors
    rust_type: Option<RuntimeType>,
    // a list of names in the rust_type that should be considered generics
    #[serde(default)]
    generics: Vec<String>,
    // type hint- a more abstract type that all potential arguments inherit from
    hint: Option<String>,
    // plaintext description of the argument used to generate documentation
    description: Option<String>,
    // default value for the argument
    #[serde(default, deserialize_with = "deserialize_some")]
    default: Option<Value>,
    // set to true if the argument represents a type
    #[serde(default)]
    is_type: bool,
    // most functions convert c_to_py or py_to_c. Set to true to leave the value as-is
    // an example usage is slice_as_object,
    //  to prevent the returned AnyObject from getting converted back to python
    #[serde(default)]
    do_not_convert: bool,
    // when is_type, use this as an example to infer the type
    #[serde(default)]
    example: Option<RuntimeType>
}

// deserialize "k": null as `Some(Value::Null)` and no key as `None`.
fn deserialize_some<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
    where T: Deserialize<'de>, D: Deserializer<'de> {
    Deserialize::deserialize(deserializer).map(Some)
}

#[allow(dead_code)]
impl Argument {
    fn name(&self) -> String {
        self.name.clone().expect("unknown name when parsing argument")
    }
    fn c_type(&self) -> String {
        if self.is_type {
            if self.c_type.is_some() { panic!("c_type should not be specified when is_type") }
            return "char *".to_string()
        }
        self.c_type.clone().expect("unknown c_type when parsing argument")
    }
    fn c_type_origin(&self) -> String {
        self.c_type().split("<").next().unwrap().to_string()
    }
}

// RuntimeType contains the metadata to generate code that evaluates to a rust type name
#[derive(Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
enum RuntimeType {
    // reference an existing RuntimeType
    Name(String),
    // get the ith subtype of an existing RuntimeType
    Lower { root: Box<RuntimeType>, index: i32 },
    // build a higher level RuntimeType
    Raise { origin: String, args: Vec<Box<RuntimeType>> },
    // construct the RuntimeType via function call
    Function { function: String, params: Vec<Box<RuntimeType>> },
}

impl<S: Into<String>> From<S> for RuntimeType {
    fn from(name: S) -> Self {
        RuntimeType::Name(name.into())
    }
}

fn main() {
    // only build the bindings if you're in dev mode
    if env::var("CARGO_PKG_VERSION").unwrap().as_str() != "0.0.0-development" { return }

    let module_names = vec!["comb", "meas", "trans", "data", "core", "accuracy"];

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
    text.split("\n").map(|v| format!("    {}", v)).collect::<Vec<_>>().join("\n")
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
                    Box::new(flatten_runtime_type(arg, derived_types))).collect()
            },
        other => other.clone()
    }
}
