//! Public API for detecting software project frameworks and tracking user verification.

mod rules;
mod types;

#[cfg(test)]
mod tests;

use std::fs;
use std::path::Path;

use crate::scanner::FileNode;

pub use rules::detect_from_signals;
pub use types::{ProjectKind, ProjectProfile, ProjectSignals};

/// Maximum bytes to read from a root manifest file (`package.json` or `Cargo.toml`).
const MAX_MANIFEST_BYTES: u64 = 256 * 1024;

/// Inspects a scanned project root `FileNode` and returns its detected `ProjectProfile`.
pub fn detect_project_profile(root_node: &FileNode) -> ProjectProfile {
    let root_entry_names: Vec<String> = root_node
        .children
        .iter()
        .map(|child| child.name.clone())
        .collect();

    let package_json_content = read_optional_manifest(&root_node.full_path, "package.json");
    let cargo_toml_content = read_optional_manifest(&root_node.full_path, "Cargo.toml");

    let signals = ProjectSignals {
        root_entry_names,
        package_json_content,
        cargo_toml_content,
    };

    detect_from_signals(&signals)
}

/// Reads a manifest file from `root_dir` into a UTF-8 string if it exists and is under 256 KB.
fn read_optional_manifest(root_dir: &Path, file_name: &str) -> Option<String> {
    let manifest_path = root_dir.join(file_name);
    let metadata = fs::metadata(&manifest_path).ok()?;
    if !metadata.is_file() || metadata.len() > MAX_MANIFEST_BYTES {
        return None;
    }

    fs::read_to_string(manifest_path).ok()
}
