#[cfg(feature = "derive")]
mod derive;

use std::process::Command;

fn main() {
    // causes patch_polars to update if patch is commented out
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=Cargo.lock");

    // We need to be able to build against two different Polars versions:
    // 1. Polars on Crates.io, tied to a Rust crate release
    // 2. Polars on PyPI, tied to a specific commit hash via a patch
    // Local builds are always variant 1, which is convenient because it allows Python interchange.
    autocfg::emit_possibility("patch_polars");

    let polars = std::env::var_os("CARGO_FEATURE_POLARS").is_some();
    let ffi = std::env::var_os("CARGO_FEATURE_FFI").is_some();
    let polars_ffi = std::env::var_os("CARGO_FEATURE_POLARS_FFI").is_some();

    if polars && resolved_polars_plan_uses_git_patch() {
        println!("cargo:rustc-cfg=patch_polars");
    }

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

fn resolved_polars_plan_uses_git_patch() -> bool {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let manifest_path = format!("{manifest_dir}/Cargo.toml");

    let mut command = Command::new(cargo);
    command.args([
        "metadata",
        "--format-version",
        "1",
        "--manifest-path",
        &manifest_path,
        "--features",
        "polars",
    ]);

    let Ok(output) = command.output() else {
        return false;
    };

    if !output.status.success() {
        return false;
    }

    let Ok(stdout) = String::from_utf8(output.stdout) else {
        return false;
    };

    stdout.contains(
        "\"id\":\"git+https://github.com/pola-rs/polars?tag=py-1.36.1#polars-plan@0.52.0\"",
    )
}
