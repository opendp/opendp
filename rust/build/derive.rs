use std::{collections::HashMap, path::Path, str::FromStr};

use opendp_tooling::{
    bootstrap::docstring::{get_proof_path, insert_proof_attribute},
    proven::filesystem::{find_proof_paths, get_src_dir, write_proof_paths},
    Function,
};
use proc_macro2::TokenStream;
use syn::{File, Item, ItemFn};

pub fn main() {
    // rebuild if link paths change
    println!("cargo:rerun-if-env-changed=OPENDP_SPHINX_PORT");
    println!("cargo:rerun-if-env-changed=OPENDP_RUSTDOC_PORT");

    println!("cargo:rerun-if-env-changed=OPENDP_REMOTE_SPHINX_URI");
    println!("cargo:rerun-if-env-changed=OPENDP_REMOTE_RUSTDOC_URI");

    let src_dir = get_src_dir().unwrap();
    let proof_paths = find_proof_paths(&src_dir).unwrap();

    // write out a json file containing the proof paths. To be used by the proc macros later
    write_proof_paths(&proof_paths).unwrap();

    #[cfg(feature = "bindings")]
    generate_header();

    // parse crate into module metadata
    // this function returns None if code is malformed/unparseable
    // if parse_crate returns none, then build script exits without updating bindings
    // the proc macros will render those errors in a way that integrates better with developer tooling
    if let Some(_modules) = parse_crate(&src_dir, proof_paths).unwrap() {
        #[cfg(feature = "bindings")]
        {
            use opendp_tooling::codegen;
            use std::fs::canonicalize;
            // generate and write language bindings based on collected metadata
            let base_dir = canonicalize("../python/src/opendp").unwrap();
            codegen::write_bindings(base_dir, codegen::python::generate_bindings(&_modules));

            let base_dir = canonicalize("../R/opendp").unwrap();
            codegen::write_bindings(base_dir.clone(), codegen::r::generate_bindings(&_modules));
            std::fs::copy("opendp.h", base_dir.join("src/opendp.h")).unwrap();
        }
    }
}

#[cfg(feature = "bindings")]
fn generate_header() {
    use cbindgen::{Config, Language};
    use std::env;

    // write `opendp.h`
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut config = Config::default();
    config.language = Language::C;
    config.pragma_once = true;
    match cbindgen::generate_with_config(&crate_dir, config) {
        Ok(bindings) => bindings.write_to_file("opendp.h"),
        Err(cbindgen::Error::ParseSyntaxError { .. }) => return, // ignore in favor of cargo's syntax check
        Err(err) => panic!("{:?}", err),
    };
}

/// Parses all modules in opendp crate
fn parse_crate(
    src_dir: &Path,
    proof_paths: HashMap<String, Option<String>>,
) -> std::io::Result<Option<HashMap<String, Vec<Function>>>> {
    let mut modules = HashMap::new();
    for entry in std::fs::read_dir(src_dir)? {
        let path = entry?.path();
        println!("cargo:rerun-if-changed={}", path.display());

        let module_name =
            if let Some(name) = path.file_name().expect("paths are canonicalized").to_str() {
                name.to_string()
            } else {
                continue;
            };

        if path.is_dir() {
            if let Some(module) = parse_file_tree(&path, &proof_paths, &module_name)? {
                if !module.is_empty() {
                    // sort module functions
                    let mut module = module.into_iter().collect::<Vec<Function>>();
                    module.sort_by_key(|func| func.name.clone());
                    modules.insert(module_name, module);
                }
            } else {
                // parsing failed
                return Ok(None);
            }
        }
    }
    Ok(Some(modules))
}

/// Search for bootstrap macro invocations by recursing over `dir`
fn parse_file_tree(
    dir: &Path,
    proof_paths: &HashMap<String, Option<String>>,
    module_name: &str,
) -> std::io::Result<Option<Vec<Function>>> {
    // use here to shadow syn::File
    use std::{fs::File, io::Read};

    let mut matches = Vec::new();
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_dir() {
                if let Some(parsed) = parse_file_tree(&path, &proof_paths, &module_name)? {
                    matches.extend(parsed);
                } else {
                    return Ok(None);
                };
            } else {
                // skip non-rust files
                if path.extension().unwrap_or_default() != "rs" {
                    continue;
                }

                let mut contents = String::new();
                File::open(&path)?.read_to_string(&mut contents)?;
                if let Some(funcs) = parse_file(contents, &proof_paths, module_name) {
                    matches.extend(funcs);
                } else {
                    return Ok(None);
                }
            };
        }
    }
    Ok(Some(matches))
}

/// Search a file for bootstrap macro invocations
fn parse_file(
    text: String,
    proof_paths: &HashMap<String, Option<String>>,
    module_name: &str,
) -> Option<Vec<Function>> {
    // ignore files that fail to parse so as not to break IDE tooling
    let ts = TokenStream::from_str(&text).ok()?;

    // use parse2 and TokenStream from proc_macro2 because we are not in a proc_macro context
    let items = syn::parse2::<File>(ts).ok()?.items;

    // flatten and filter the contents of a file into a vector of functions
    fn flatten_fns(item: Item) -> Vec<ItemFn> {
        match item {
            Item::Fn(func) => vec![func],
            Item::Mod(module) => module
                .content
                .map(|v| v.1.into_iter().flat_map(flatten_fns).collect())
                .unwrap_or_else(Vec::new),
            _ => Vec::new(),
        }
    }

    fn path_is_eq(path: &syn::Path, name: &str) -> bool {
        path.get_ident().map(ToString::to_string).as_deref() == Some(name)
    }

    // parse all functions
    (items.into_iter())
        .flat_map(flatten_fns)
        // ignore the function if it doesn't have a bootstrap proc macro invocation
        .filter(|func| (func.attrs.iter()).any(|attr| path_is_eq(&attr.path, "bootstrap")))
        // attempt to parse and simulate the proc macro on the function
        .map(|mut item_fn| {
            // extract the bootstrap attribute
            let idx = (item_fn.attrs.iter())
                .position(|attr| path_is_eq(&attr.path, "bootstrap"))
                .expect("bootstrap attr always exists because of filter");
            let bootstrap_attr = item_fn.attrs.remove(idx);

            // parse args to the bootstrap macro
            let attr_args = match bootstrap_attr.parse_meta().ok()? {
                syn::Meta::List(ml) => ml.nested.into_iter().collect(),
                _ => return None,
            };

            // mutate the docstring to add a proof path
            if let Some(proof_path) = get_proof_path(&attr_args, &item_fn, &proof_paths).ok()? {
                insert_proof_attribute(&mut item_fn.attrs, proof_path).ok()?;
            }

            // use the bootstrap crate to parse a Function
            Function::from_ast(attr_args, item_fn, Some(module_name)).ok()
        })
        .collect()
}
