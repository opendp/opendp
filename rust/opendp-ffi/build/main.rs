use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use indexmap::map::IndexMap;
use serde::Deserialize;

#[cfg(feature="python")]
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
    // metadata for constructing new types based on existing types or introspection
    #[serde(default)]
    derived_types: Vec<Argument>,
    // plaintext description of the function used to generate documentation
    description: Option<String>
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
    // type hint- a more abstract type that all potential arguments inherit from
    hint: Option<String>,
    // plaintext description of the argument used to generate documentation
    description: Option<String>,
    // default value for the argument
    default: Option<String>,
    // set to true if the argument represents a type
    #[serde(default)]
    is_type: bool,
    // most functions call c_to_py on return values. Set to true to leave the return value as-is
    // this is a special case for _slice_as_object,
    //  to prevent the returned AnyObject from getting converted back to python
    #[serde(default)]
    keep_as_c: bool
}

#[allow(dead_code)]
impl Argument {
    fn name(&self) -> String {
        self.name.clone().expect("unknown name when parsing argument")
    }
    fn c_type(&self) -> String {
        self.c_type.clone().expect("unknown c_type when parsing argument")
    }
    fn c_type_origin(&self) -> String {
        self.c_type().split("<").next().unwrap().to_string()
    }
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
enum RuntimeType {
    Name(String),
    // get the ith subtype of an existing RuntimeType
    Lower { root: Box<RuntimeType>, index: i32 },
    // build a higher level RuntimeType
    Raise { origin: String, args: Vec<Box<RuntimeType>> },
    // construct the RuntimeType via function call
    Function { function: String, params: Vec<String> },
}

impl<S: Into<String>> From<S> for RuntimeType {
    fn from(name: S) -> Self {
        RuntimeType::Name(name.into())
    }
}

#[allow(dead_code)]
fn write_bindings(files: IndexMap<PathBuf, String>) {
    let base_dir = PathBuf::from(env::var("OPENDP_PYTHON_SRC_DIR")
        .expect("failed to read environment variable OPENDP_PYTHON_SRC_DIR"));
    for (file_path, file_contents) in files {
        File::create(base_dir.join(file_path)).unwrap()
            .write_all(file_contents.as_ref()).unwrap();
    }
}

fn main() {
    let module_names = vec!["core", "meas", "trans", "data"];

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

    #[cfg(feature="python")] write_bindings(python::generate_bindings(_modules));
}