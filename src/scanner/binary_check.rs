//! Pure functions for identifying binary files and heavy dependency lockfiles.

use std::path::Path;

use crate::scanner::types::FileCategory;

/// Known lockfiles that should appear in the directory tree but be excluded from "All Code" dumps.
const KNOWN_LOCKFILES: &[&str] = &[
    "Cargo.lock",
    "yarn.lock",
    "package-lock.json",
    "pnpm-lock.yaml",
    "bun.lockb",
    "poetry.lock",
    "composer.lock",
    "Gemfile.lock",
];

/// File extensions that are always binary or media assets.
const BINARY_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "webp", "ico", "bmp", "tiff", "pdf", "zip", "tar", "gz", "7z",
    "rar", "exe", "dll", "so", "dylib", "wasm", "class", "pyc", "ttf", "otf", "woff", "woff2",
    "eot", "mp3", "mp4", "wav", "ogg", "mov", "avi", "sqlite", "db",
];

/// Null byte value used by Git and ripgrep heuristics to detect binary content.
const NULL_BYTE: u8 = 0x00;

/// Classifies a file based on its filename, extension, and initial header bytes.
pub fn classify_file(path: &Path, header_sample: &[u8]) -> FileCategory {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();

    if is_heavy_lockfile(file_name) {
        return FileCategory::Lockfile;
    }

    if has_binary_extension(path) || contains_null_byte(header_sample) {
        return FileCategory::Binary;
    }

    FileCategory::TextCode
}

/// Returns `true` if the filename matches a known package manager lockfile.
pub fn is_heavy_lockfile(file_name: &str) -> bool {
    KNOWN_LOCKFILES
        .iter()
        .any(|&lockfile| lockfile.eq_ignore_ascii_case(file_name))
}

/// Returns `true` if the path has a known binary, archive, font, or media extension.
pub fn has_binary_extension(path: &Path) -> bool {
    let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
        return false;
    };

    BINARY_EXTENSIONS
        .iter()
        .any(|&binary_ext| binary_ext.eq_ignore_ascii_case(ext))
}

/// Returns `true` if the inspected byte slice contains at least one null byte (`0x00`).
pub fn contains_null_byte(sample: &[u8]) -> bool {
    sample.contains(&NULL_BYTE)
}
