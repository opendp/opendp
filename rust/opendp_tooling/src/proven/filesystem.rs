use std::{collections::HashMap, env, ffi::OsStr, path::PathBuf};

use darling::{Error, Result};

/// Traverses the filesystem, starting at src_dir, looking for .tex files.
/// If more than one file is discovered with the same name, the value becomes None
pub fn find_proof_paths(
    src_dir: &std::path::Path,
) -> std::io::Result<HashMap<String, Option<String>>> {
    let mut proof_paths = HashMap::new();
    find_unique_file_names_with_extension(&mut proof_paths, &OsStr::new("tex"), src_dir, src_dir)?;
    Ok(proof_paths)
}

/// Writes a collection of proof paths to {OUT_DIR}/proof_paths.json.
pub fn write_proof_paths(proof_paths: &HashMap<String, Option<String>>) -> Result<()> {
    std::fs::write(
        get_out_dir()?.join("proof_paths.json"),
        serde_json::to_string(proof_paths).map_err(Error::custom)?,
    )
    .map_err(Error::custom)
}

/// Load proof paths from {OUT_DIR}/proof_paths.json.
/// Assumes the file was written in the build script.
pub fn load_proof_paths() -> Result<HashMap<String, Option<String>>> {
    serde_json::from_str(
        &std::fs::read_to_string(get_out_dir()?.join("proof_paths.json")).map_err(Error::custom)?,
    )
    .map_err(Error::custom)
}

/// The inner function for find_proof_paths
fn find_unique_file_names_with_extension(
    matches: &mut HashMap<String, Option<String>>,
    file_extension: &OsStr,
    root_dir: &std::path::Path,
    dir: &std::path::Path,
) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_dir() {
                find_unique_file_names_with_extension(matches, file_extension, root_dir, &path)?;
            } else {
                if path.extension() != Some(file_extension) {
                    continue;
                }
                if let Some(file_name) = path.file_stem() {
                    matches
                        .entry(file_name.to_string_lossy().to_string())
                        .and_modify(|v| drop(v.take()))
                        .or_insert_with(|| {
                            Some(
                                path.strip_prefix(root_dir)
                                    .expect("unreachable")
                                    .to_string_lossy()
                                    .to_string(),
                            )
                        });
                }
            };
        }
    }
    Ok(())
}

pub fn get_src_dir() -> Result<PathBuf> {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
        .ok_or_else(|| Error::custom("Failed to determine location of Cargo.toml."))?;
    Ok(PathBuf::from(manifest_dir).join("src"))
}
fn get_out_dir() -> Result<PathBuf> {
    let manifest_dir =
        std::env::var_os("OUT_DIR").ok_or_else(|| Error::custom("Failed to determine OUT_DIR."))?;
    Ok(PathBuf::from(manifest_dir))
}

pub fn make_proof_link(relative_path: PathBuf) -> Result<String> {
    // construct absolute path
    let absolute_path = get_src_dir()?.join(&relative_path);

    if !absolute_path.exists() {
        return Err(Error::custom(format!("{absolute_path:?} does not exist!")));
    }

    let version = env!("CARGO_PKG_VERSION");
    let target = if version == "0.0.0+development" {
        (absolute_path.to_str())
            .ok_or_else(|| Error::custom("absolute path is empty"))?
            .to_string()
    } else {
        let docs_url =
            env::var("OPENDP_DOCS_SITE").unwrap_or_else(|_| "docs.opendp.org".to_string());
        format!(
            "https://{docs_url}/en/{version}/proofs/{relative_path}",
            relative_path = relative_path.display()
        )
    };

    Ok(format!("[Link to proof.]({target})"))
}
