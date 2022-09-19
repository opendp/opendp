use std::{
    env,
    ffi::OsStr,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use proc_macro::TokenStream;
use syn::{parse_macro_input, AttributeArgs, ItemFn, Lit, Meta, NestedMeta};

#[cfg(feature = "bootstrap-json")]
mod bootstrap;

#[proc_macro_attribute]
pub fn bootstrap(attr: TokenStream, input: TokenStream) -> TokenStream {
    let original_input = input.clone();

    let attr_args = parse_macro_input!(attr as AttributeArgs);
    let item_fn = parse_macro_input!(input as ItemFn);

    let features = (attr_args.iter())
        // filter down to NameValues
        .filter_map(|nm| match nm {
            NestedMeta::Meta(Meta::List(ml)) => Some(ml),
            _ => None,
        })
        // find the features NestedMeta
        .find(|ml| {
            ml.path
                .get_ident()
                .map(|ident| ident.to_string() == "features")
                .unwrap_or(false)
        })
        // extract a vector of Strings from the features NestedMeta
        .map(|meta_feats| {
            meta_feats
                .nested
                .iter()
                .map(|feat| extract!(feat, NestedMeta::Lit(Lit::Str(lit)) => lit.value()))
                .collect()
        })
        .unwrap_or_else(Vec::new);

    let manifest_dir =
        env::var_os("CARGO_MANIFEST_DIR").expect("Failed to determine location of Cargo.toml.");
    let src_dir = PathBuf::from(manifest_dir).join("src");
    let func_name = item_fn.sig.ident.to_string();

    // an optional relative PathBuf
    let proof_path = (attr_args.iter())
        // filter down to NameValues
        .filter_map(|nm| match nm {
            NestedMeta::Meta(Meta::NameValue(mnv)) => Some(mnv),
            _ => None,
        })
        // find the proof NameValue
        .find(|mnv| {
            mnv.path
                .get_ident()
                .map(|ident| ident.to_string() == "proof")
                .unwrap_or(false)
        })
        // extract the Value
        .map(|mnv| extract!(mnv.lit, Lit::Str(ref litstr) => PathBuf::from(litstr.value())))
        // otherwise if no proof path was provided, search for it
        .or_else(|| {
            find_file_path(&OsStr::new(format!("{}.tex", func_name).as_str()), &src_dir)
                .unwrap()
                // turn into relative PathBuf
                .map(|pb| pb.strip_prefix(&src_dir).unwrap().to_path_buf())
        });

    #[cfg(feature = "bootstrap-json")]
    // first, retrieve a relative path (either to the proof or to the source)
    let module_name = (proof_path.clone())
        .unwrap_or_else(|| {
            // detect location of call site
            find_source_path(&format!("pub fn {func_name}"), &src_dir)
                .unwrap()
                .expect("No matching source file found.")
                .strip_prefix(&src_dir)
                .unwrap()
                .to_path_buf()
        })
        // retrieve the first component as a string
        .components()
        .next()
        .unwrap()
        .as_os_str()
        .to_str()
        .expect("module name must be non-empty")
        .to_string();

    let proof_link = proof_path.map(|relative| make_proof_link(&src_dir, &relative));

    #[cfg(feature = "bootstrap-json")]
    if let Err(e) = bootstrap::write_json(module_name, attr_args, item_fn, proof_link.clone()) {
        return TokenStream::from(e.write_errors());
    }

    // embed link to proof in documentation
    let mut output = proof_link
        .map(|link| TokenStream::from(quote::quote!(#[doc = #link])))
        .unwrap_or_else(TokenStream::default);

    features.iter()
        .for_each(|feat| output.extend(TokenStream::from(quote::quote!(#[cfg(feature=#feat)]))));

    output.extend(original_input);

    output
}

macro_rules! extract {
    ($value:expr, $pattern:pat => $extracted_value:expr) => {
        match $value {
            $pattern => $extracted_value,
            _ => panic!(concat!(
                stringify!($value),
                " doesn't match ",
                stringify!($pattern)
            )),
        }
    };
}
pub(crate) use extract;

fn find_file_path(file_name: &OsStr, dir: &Path) -> std::io::Result<Option<PathBuf>> {
    let mut matches = Vec::new();
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_dir() {
                matches.extend(find_file_path(file_name, &path)?);
            } else {
                if path.file_name() == Some(file_name) {
                    matches.push(path);
                }
            };
        }
    }
    if matches.len() > 1 {
        panic!("multiple matching proofs found. Please specify `proof = \"{{module}}/path/to/proof\"` in the bootstrap attributes.")
    }
    Ok(matches.get(0).cloned())
}

fn find_source_path(source: &str, dir: &Path) -> std::io::Result<Option<PathBuf>> {
    let mut matches = Vec::new();
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_dir() {
                matches.extend(find_source_path(source, &path)?);
            } else {
                // skip non-rust files
                if path.extension().unwrap_or_default() != "rs" {
                    continue;
                }

                let mut contents = String::new();
                File::open(&path)?.read_to_string(&mut contents)?;
                if contents.contains(source) {
                    matches.push(path)
                }
            };
        }
    }
    if matches.len() > 1 {
        panic!("multiple matching source files found. Please specify `module = \"modulename\"` in the bootstrap attributes.")
    }
    Ok(matches.pop())
}

fn make_proof_link(src_path: &PathBuf, relative_path: &PathBuf) -> String {
    // construct absolute path
    let absolute_path = src_path.join(relative_path);

    assert!(
        absolute_path.exists(),
        "{:?} does not exist!",
        absolute_path
    );

    let target = if cfg!(feature = "local") {
        absolute_path
            .to_str()
            .expect("failed to retrieve str")
            .to_string()
    } else {
        format!(
            "https://docs.opendp.org/en/{version}/proofs/{relative_path}",
            version = get_version(),
            relative_path = relative_path.display()
        )
    };

    format!("[Link to proof.]({})", target)
}

fn get_version() -> String {
    let version = env::var("CARGO_PKG_VERSION")
        .expect("CARGO_PKG_VERSION must be set")
        .to_string();
    if version == "0.0.0+development" {
        "latest".to_string()
    } else {
        version
    }
}
