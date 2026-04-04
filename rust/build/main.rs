#[cfg(feature = "derive")]
mod derive;

fn main() {
    let polars = std::env::var_os("CARGO_FEATURE_POLARS").is_some();
    let ffi = std::env::var_os("CARGO_FEATURE_FFI").is_some();
    let polars_ffi = std::env::var_os("CARGO_FEATURE_POLARS_FFI").is_some();

    if polars && ffi && !polars_ffi {
        eprintln!(
            "\nerror: invalid feature combination: `polars` + `ffi` requires `polars-ffi`\n\
             Fix:\n\
               cargo build --features \"polars-ffi\"\n"
        );
        std::process::exit(1);
    }

    #[cfg(feature = "bindings")]
    {
        // https://pyo3.rs/v0.25.1/building-and-distribution.html#macos
        pyo3_build_config::add_extension_module_link_args();
        pyo3_build_config::add_python_framework_link_args();
    }

    #[cfg(feature = "derive")]
    crate::derive::main();
}
