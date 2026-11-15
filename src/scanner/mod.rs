//! Public API for scanning software project directories while respecting ignore files.

mod binary_check;
mod scan_job;
mod types;
mod walker;

#[cfg(test)]
mod tests;

pub use binary_check::{classify_file, has_known_text_extension, is_heavy_lockfile};
pub use scan_job::{spawn_scan_job, ScanJobHandle, ScanState};
pub use types::{FileCategory, FileNode, ScanError, ScanOptions, DEFAULT_MAX_FILE_SIZE_BYTES};
pub use walker::{scan_directory, scan_directory_with_progress};
