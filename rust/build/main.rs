use cbindgen::{Config, Language};
use std::env;


#[cfg(feature = "derive")]
mod derive;

fn main() {

    #[cfg(feature = "derive")]
    crate::derive::main();

    // write `opendp.h`
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut config = Config::default();
    config.language = Language::C;
    match cbindgen::generate_with_config(&crate_dir, config) {
        Ok(bindings) => bindings.write_to_file("opendp.h"),
        Err(cbindgen::Error::ParseSyntaxError { .. }) => return, // ignore in favor of cargo's syntax check
        Err(err) => panic!("{:?}", err)
    };
}

