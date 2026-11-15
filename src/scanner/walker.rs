//! Filesystem directory walker respecting `.gitignore` and `.ignore` rules.

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::scanner::binary_check::{
    classify_file, has_binary_extension, has_known_text_extension, is_heavy_lockfile,
};
use crate::scanner::types::{FileCategory, FileNode, ScanError, ScanOptions};
use ignore::{DirEntry, WalkBuilder};

/// Number of initial bytes inspected from an unknown file to detect null bytes.
const BINARY_SNIFF_BYTES: usize = 512;

/// Version control metadata directory excluded from project trees even when hidden files are shown.
const VCS_DIR_NAME: &str = ".git";

/// Scans `root_path` respecting `.gitignore` and `.ignore` files and returns the root `FileNode`.
pub fn scan_directory(root_path: &Path, options: &ScanOptions) -> Result<FileNode, ScanError> {
    scan_directory_with_progress(root_path, options, |_, _| {})
}

/// Scans `root_path` while invoking `on_progress(entries_scanned, relative_path)` for live UI updates.
pub fn scan_directory_with_progress<F>(
    root_path: &Path,
    options: &ScanOptions,
    mut on_progress: F,
) -> Result<FileNode, ScanError>
where
    F: FnMut(usize, &Path),
{
    validate_root_directory(root_path)?;

    let nodes_by_path = collect_filtered_nodes(root_path, options, &mut on_progress)?;
    let mut root_node = assemble_hierarchy(root_path, nodes_by_path)?;
    sort_children_recursively(&mut root_node);

    Ok(root_node)
}

/// Validates that the given path exists on disk and points to a directory.
fn validate_root_directory(root_path: &Path) -> Result<(), ScanError> {
    if !root_path.exists() {
        return Err(ScanError::RootNotFound(root_path.to_path_buf()));
    }
    if !root_path.is_dir() {
        return Err(ScanError::NotADirectory(root_path.to_path_buf()));
    }
    Ok(())
}

/// Walks the directory tree with `ignore` and collects a map of relative paths to `FileNode`s.
fn collect_filtered_nodes<F>(
    root_path: &Path,
    options: &ScanOptions,
    on_progress: &mut F,
) -> Result<BTreeMap<PathBuf, FileNode>, ScanError>
where
    F: FnMut(usize, &Path),
{
    let walker = WalkBuilder::new(root_path)
        .hidden(!options.should_include_hidden)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .ignore(true)
        .require_git(false)
        .filter_entry(|entry| !is_vcs_directory(entry))
        .build();

    let mut nodes_by_path = BTreeMap::new();
    let mut scanned_count = 0;

    for result in walker {
        let entry = result.map_err(|err| ScanError::TraversalFailed {
            path: root_path.to_path_buf(),
            reason: err.to_string(),
        })?;

        let node = build_node_from_entry(root_path, &entry)?;
        scanned_count += 1;
        on_progress(scanned_count, &node.relative_path);
        nodes_by_path.insert(node.relative_path.clone(), node);
    }

    Ok(nodes_by_path)
}

/// Returns `true` if the entry is the internal `.git` metadata directory.
fn is_vcs_directory(entry: &DirEntry) -> bool {
    entry.file_type().is_some_and(|ft| ft.is_dir()) && entry.file_name() == VCS_DIR_NAME
}

/// Converts a single `ignore::DirEntry` into a `FileNode`.
fn build_node_from_entry(root_path: &Path, entry: &DirEntry) -> Result<FileNode, ScanError> {
    let full_path = entry.path().to_path_buf();
    let relative_path = full_path
        .strip_prefix(root_path)
        .unwrap_or(Path::new(""))
        .to_path_buf();

    let metadata = entry.metadata().map_err(|err| ScanError::TraversalFailed {
        path: full_path.clone(),
        reason: err.to_string(),
    })?;

    let is_dir = metadata.is_dir();
    let size_bytes = if is_dir { 0 } else { metadata.len() };
    let category = if is_dir {
        FileCategory::TextCode
    } else {
        inspect_file_category(&full_path)
    };

    let name = full_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("root")
        .to_string();

    Ok(FileNode {
        name,
        relative_path,
        full_path,
        is_dir,
        size_bytes,
        category,
        children: Vec::new(),
    })
}

/// Classifies a file by its name/extension first, reading 512 bytes from disk only when unknown.
fn inspect_file_category(full_path: &Path) -> FileCategory {
    let file_name = full_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();

    if is_heavy_lockfile(file_name) {
        return FileCategory::Lockfile;
    }
    if has_binary_extension(full_path) {
        return FileCategory::Binary;
    }
    if has_known_text_extension(full_path) {
        return FileCategory::TextCode;
    }

    let mut buffer = [0_u8; BINARY_SNIFF_BYTES];
    let bytes_read = File::open(full_path)
        .and_then(|mut file| file.read(&mut buffer))
        .unwrap_or(0);

    classify_file(full_path, &buffer[..bytes_read])
}

/// Folds a flat map of relative paths bottom-up into a single root `FileNode` tree.
fn assemble_hierarchy(
    root_path: &Path,
    mut nodes_by_path: BTreeMap<PathBuf, FileNode>,
) -> Result<FileNode, ScanError> {
    let paths_deepest_first: Vec<PathBuf> = nodes_by_path.keys().cloned().rev().collect();

    for rel_path in paths_deepest_first {
        if rel_path.as_os_str().is_empty() {
            continue;
        }
        let Some(child_node) = nodes_by_path.remove(&rel_path) else {
            continue;
        };
        let parent_path = rel_path.parent().unwrap_or(Path::new("")).to_path_buf();
        if let Some(parent_node) = nodes_by_path.get_mut(&parent_path) {
            parent_node.children.push(child_node);
        }
    }

    nodes_by_path
        .remove(Path::new(""))
        .ok_or_else(|| ScanError::TraversalFailed {
            path: root_path.to_path_buf(),
            reason: "Missing root node after directory scan".to_string(),
        })
}

/// Sorts directories before files, and alphabetically (case-insensitive) within each group.
fn sort_children_recursively(node: &mut FileNode) {
    node.children.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        _ => a
            .name
            .to_lowercase()
            .cmp(&b.name.to_lowercase())
            .then_with(|| a.name.cmp(&b.name)),
    });

    for child in &mut node.children {
        if child.is_dir {
            sort_children_recursively(child);
        }
    }
}
