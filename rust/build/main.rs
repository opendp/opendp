use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
    str::FromStr,
};

use opendp_bootstrap::Function;
use proc_macro2::TokenStream;
use syn::{File, Item, ItemFn};

mod codegen;


// sets OUT_DIR for proc-macro usage
fn main() {
    let modules = parse_modules().unwrap();

    let _modules = modules.into_iter().map(|(name, module)| {
        let mut module = module.into_iter().filter(|(_k, v)| !v.is_empty()).collect::<Vec<(String, Function)>>();
        module.sort_by_key(|(k, _)| k.clone());
        (name, module)
    }).collect();

    if cfg!(feature = "bindings-python") {
        codegen::write_bindings(codegen::python::generate_bindings(_modules));
    }
}

fn parse_modules() -> std::io::Result<HashMap<String, HashMap<String, Function>>> {
    let manifest_dir =
        env::var_os("CARGO_MANIFEST_DIR").expect("Failed to determine location of Cargo.toml.");
    let src_dir = PathBuf::from(manifest_dir).join("src");

    let mut modules = HashMap::new();
    for entry in std::fs::read_dir(src_dir)? {
        let path = entry?.path();
        let module_name = path
            .file_name()
            .expect("file name must not be empty")
            .to_str()
            .unwrap()
            .to_string();

        if path.is_dir() {
            modules.insert(module_name, parse_file_tree(&path)?);
        }
    }
    Ok(modules)
}

fn parse_file_tree(dir: &Path) -> std::io::Result<HashMap<String, Function>> {
    use std::{fs::File, io::Read};

    let mut matches = HashMap::new();
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_dir() {
                matches.extend(parse_file_tree(&path)?);
            } else {
                // skip non-rust files
                if path.extension().unwrap_or_default() != "rs" {
                    continue;
                }

                let mut contents = String::new();
                File::open(&path)?.read_to_string(&mut contents)?;
                matches.extend(parse_file(contents))
            };
        }
    }
    Ok(matches)
}

fn parse_file(text: String) -> HashMap<String, Function> {
    let items = TokenStream::from_str(&text)
        .ok()
        .and_then(|ts| syn::parse2::<File>(ts).ok())
        .map(|file| file.items)
        .unwrap_or_else(Vec::new);

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
    (items.into_iter())
        .flat_map(flatten_fns)
        .filter(|func| {
            func.attrs
                .iter()
                .any(|attr| path_is_eq(&attr.path, "bootstrap"))
        })
        .filter_map(|mut func| {
            let idx = func
                .attrs
                .iter()
                .position(|attr| path_is_eq(&attr.path, "bootstrap")).unwrap();
            let attr = func.attrs.remove(idx);
            let attr_args = match attr.parse_meta().expect("failed to parse bootstrap meta") {
                syn::Meta::List(ml) => ml.nested.into_iter().collect(),
                _ => return None
            };

            Some(Function::from_ast(attr_args, func).unwrap())
        })
        .collect()
}
