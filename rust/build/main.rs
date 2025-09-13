#[cfg(feature = "derive")]
mod derive;

fn main() {
    #[cfg(feature = "bindings")]
    {
        // https://pyo3.rs/v0.25.1/building-and-distribution.html#macos
        pyo3_build_config::add_extension_module_link_args();
        pyo3_build_config::add_python_framework_link_args();
    }

    #[cfg(feature = "derive")]
    crate::derive::main();
}
