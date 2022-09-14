use proc_macro::TokenStream;
use syn::{parse_macro_input, AttributeArgs, NestedMeta, Lit};
use std::{env, path::PathBuf};

#[proc_macro_attribute]
pub fn proven_by(attr: TokenStream, item: TokenStream) -> TokenStream {
    let config = parse_macro_input!(attr as AttributeArgs);

    let mut relative_path = if let NestedMeta::Lit(Lit::Str(path)) = &config[0] {
        PathBuf::from(path.value())
    } else {
        panic!("Proof must be passed a string file path.");
    };

    // construct absolute path
    let mut absolute_path = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR")
        .expect("Failed to determine location of Cargo.toml."))
        .join("src")
        .join(relative_path.clone());

    // enable this when we actually have proofs
    // assert!(absolute_path.exists(), "{:?} does not exist!", absolute_path);

    let target = if cfg!(feature = "local") {
        // build path to local pdf
        absolute_path.set_extension("pdf");
        absolute_path.to_str().unwrap().to_string()
    } else {
        // build path to docs website
        relative_path.set_extension("pdf");
        format!(
            "https://docs.opendp.org/en/{version}/proofs/{relative_path}", 
            version=get_version(), 
            relative_path=relative_path.display())
    };

    let comment = format!("[Link to proof.]({})", target);

    let mut ts = TokenStream::from(quote::quote!(#[doc = #comment]));
    ts.extend(item);
    ts
}

fn get_version() -> String {
    let version = env::var("CARGO_PKG_VERSION").unwrap().to_string();
    if version == "0.0.0+development" { 
        "latest".to_string()
    } else {
        version
    }
}