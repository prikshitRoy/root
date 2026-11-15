//! Public API for scanning software project directories while respecting ignore files.

mod binary_check;
mod types;
mod walker;

#[cfg(test)]
mod tests;

pub use binary_check::{classify_file, is_heavy_lockfile};
pub use types::{
     DEFAULT_MAX_FILE_SIZE_BYTES, FileCategory, FileNode, ScanError, ScanOptions,
};
pub use walker::scan_directory;
