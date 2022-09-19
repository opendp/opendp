// modified from https://github.com/dtolnay/cxx/blob/83075824a424afbbe975e744ee5ff78e3e54787e/gen/build/src/target.rs

use std::{path::PathBuf, ffi::OsStr};

pub fn find_target_dir(out_dir: &PathBuf) -> PathBuf {
    if let Some(target_dir) = std::env::var_os("CARGO_TARGET_DIR").map(PathBuf::from) {
        if !target_dir.is_absolute() {
            panic!("CARGO_TARGET_DIR passed, but must be absolute")
        }
        return target_dir;
    }

    // fs::canonicalize on Windows produces UNC paths which cl.exe is unable to
    // handle in includes.
    // https://github.com/rust-lang/rust/issues/42869
    // https://github.com/alexcrichton/cc-rs/issues/169
    let mut also_try_canonical = cfg!(not(windows));

    let mut dir = out_dir.to_owned();
    loop {
        if dir.join(".rustc_info.json").exists()
            || dir.join("CACHEDIR.TAG").exists()
            || dir.file_name() == Some(OsStr::new("target"))
                && dir
                    .parent()
                    .map_or(false, |parent| parent.join("Cargo.toml").exists())
        {
            return dir;
        }
        if dir.pop() {
            continue;
        }
        if also_try_canonical {
            if let Ok(canonical_dir) = out_dir.canonicalize() {
                dir = canonical_dir;
                also_try_canonical = false;
                continue;
            }
        }
        panic!("failed to detect target directory");
    }
}
