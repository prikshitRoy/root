//! Data structures and error contracts for project directory scanning.

use std::error::Error;
use std::fmt;
use std::path::PathBuf;

/// Default maximum file size (1 MB) before a source file is skipped in code dumps.
pub const DEFAULT_MAX_FILE_SIZE_BYTES: u64 = 1_048_576;

/// Classification of a file to control how Markdown code bundling treats it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileCategory {
    /// Standard UTF-8 source or configuration file suitable for code blocks.
    TextCode,
    /// Large auto-generated dependency lockfile (shown in tree, skipped in "All Code" by default).
    Lockfile,
    /// Non-text binary or media asset (shown in tree, never embedded as source code).
    Binary,
}

/// Configuration flags controlling how a directory tree is scanned.
#[derive(Debug, Clone)]
pub struct ScanOptions {
    /// Whether dotfiles (like `.env.example` or `.eslintrc`) should be included when not ignored.
    pub should_include_hidden: bool,
    /// Maximum size in bytes allowed for reading file contents during export.
    pub max_file_size_bytes: u64,
}

impl Default for ScanOptions {
    /// Creates default scan options that include unignored dotfiles and cap code files at 1 MB.
    fn default() -> Self {
        Self {
            should_include_hidden: true,
            max_file_size_bytes: DEFAULT_MAX_FILE_SIZE_BYTES,
        }
    }
}

/// A single directory or file entry inside a scanned software project tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileNode {
    /// Display name of the file or folder (e.g., `package.json` or `src`).
    pub name: String,
    /// Path relative to the scanned project root (e.g., `src/main.rs`).
    pub relative_path: PathBuf,
    /// Absolute filesystem path used when reading file contents on export.
    pub full_path: PathBuf,
    /// Whether this node represents a directory (`true`) or a file (`false`).
    pub is_dir: bool,
    /// File size in bytes (`0` for directories).
    pub size_bytes: u64,
    /// Content classification used to filter binary files and lockfiles.
    pub category: FileCategory,
    /// Sorted child nodes when `is_dir` is `true`.
    pub children: Vec<FileNode>,
}

/// Errors that can occur when validating or walking a project directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScanError {
    /// The target path does not exist on disk.
    RootNotFound(PathBuf),
    /// The target path points to a file rather than a directory.
    NotADirectory(PathBuf),
    /// An I/O or ignore-parsing error occurred while reading an entry.
    TraversalFailed { path: PathBuf, reason: String },
}

impl fmt::Display for ScanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RootNotFound(path) => {
                write!(f, "Project root path does not exist: '{}'", path.display())
            }
            Self::NotADirectory(path) => {
                write!(f, "Project root path is not a directory: '{}'", path.display())
            }
            Self::TraversalFailed { path, reason } => {
                write!(f, "Failed while scanning '{}': {}", path.display(), reason)
            }
        }
    }
}

impl Error for ScanError {}
