#[cfg(feature="python")]
pub mod python;

use std::path::{Path, PathBuf};
use std::fs::File;
use std::{io, env};
use std::io::{Read, Write};
use serde::Deserialize;
use indexmap::map::IndexMap;

#[derive(Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
enum RuntimeTypeSelector {
    Name(String),
    Arg {arg: (Box<RuntimeTypeSelector>, i32)}
}

impl<S: Into<String>> From<S> for RuntimeTypeSelector {
    fn from(name: S) -> Self {
        RuntimeTypeSelector::Name(name.into())
    }
}

#[derive(Deserialize, Debug, Default)]
pub struct Argument {
    name: Option<String>,
    c_type: Option<String>,
    rust_type: Option<RuntimeTypeSelector>,
    hint: Option<String>,
    description: Option<String>,
    default: Option<String>,
    #[serde(default)]
    is_type: bool
}
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
    // fn c_type_arg(&self, index: usize) -> String {
    //     let c_type = self.c_type();
    //
    // }
}

#[derive(Deserialize, Debug)]
pub struct Function {
    #[serde(default)]
    args: Vec<Argument>,
    #[serde(default)]
    ret: Argument,
    #[serde(default)]
    derived_types: Vec<Argument>,
    description: Option<String>
}

type Module = IndexMap<String, Function>;

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

    let modules = module_names.iter()
        .map(|module_name| {
            println!("parsing module: {}", module_name);
            let mut contents = String::new();
            File::open(get_bootstrap_path(module_name))
                .expect("file not found")
                .read_to_string(&mut contents)
                .expect("failed reading module json");

            Ok((module_name.to_string(), serde_json::from_str(&contents).unwrap()))
        })
        .collect::<Result<IndexMap<String, Module>, io::Error>>().unwrap();

    if cfg!(feature="python") {
        write_bindings(python::make_bindings(modules))
    }

    panic!()
}