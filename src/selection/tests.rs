//! Unit tests for `SelectionState` and `ExportMode`.

use std::path::PathBuf;

use crate::scanner::{FileCategory, FileNode};
use crate::selection::{ExportMode, SelectionState};

/// Helper to construct an in-memory `FileNode` file for unit testing.
fn sample_file(name: &str, rel_path: &str, category: FileCategory) -> FileNode {
    FileNode {
        name: name.to_string(),
        relative_path: PathBuf::from(rel_path),
        full_path: PathBuf::from(rel_path),
        is_dir: false,
        size_bytes: 120,
        category,
        children: Vec::new(),
    }
}

/// Helper to construct an in-memory `FileNode` tree for unit testing.
fn sample_project_tree() -> FileNode {
    FileNode {
        name: "demo".to_string(),
        relative_path: PathBuf::new(),
        full_path: PathBuf::from("/tmp/demo"),
        is_dir: true,
        size_bytes: 0,
        category: FileCategory::TextCode,
        children: vec![
            FileNode {
                name: "src".to_string(),
                relative_path: PathBuf::from("src"),
                full_path: PathBuf::from("/tmp/demo/src"),
                is_dir: true,
                size_bytes: 0,
                category: FileCategory::TextCode,
                children: vec![
                    sample_file("main.rs", "src/main.rs", FileCategory::TextCode),
                    sample_file("lib.rs", "src/lib.rs", FileCategory::TextCode),
                ],
            },
            sample_file("Cargo.lock", "Cargo.lock", FileCategory::Lockfile),
            sample_file("icon.png", "icon.png", FileCategory::Binary),
        ],
    }
}

#[test]
fn preselects_text_files_and_skips_lockfiles_and_binaries_by_default() {
    // Arrange
    let root = sample_project_tree();

    // Act
    let state = SelectionState::from_tree(&root);

    // Assert
    assert_eq!(state.count_included_files(&root), 2);
    assert!(state.is_path_checked(&PathBuf::from("src/main.rs")));
    assert!(!state.is_path_checked(&PathBuf::from("Cargo.lock")));
    assert!(!state.is_path_checked(&PathBuf::from("icon.png")));
}

#[test]
fn toggles_all_files_inside_directory_recursively() {
    // Arrange
    let root = sample_project_tree();
    let mut state = SelectionState::from_tree(&root);
    let src_dir = &root.children[0];

    // Act: uncheck the entire `src` folder
    state.set_directory_checked(src_dir, false);

    // Assert
    assert!(!state.is_directory_checked(src_dir));
    assert_eq!(state.count_included_files(&root), 0);

    // Act: re-check the `src` folder
    state.set_directory_checked(src_dir, true);

    // Assert
    assert!(state.is_directory_checked(src_dir));
    assert_eq!(state.count_included_files(&root), 2);
}

#[test]
fn respects_tree_only_and_all_code_export_modes() {
    // Arrange
    let root = sample_project_tree();
    let mut state = SelectionState::from_tree(&root);

    // Act & Assert: TreeOnly includes zero files even when files are checked
    state.export_mode = ExportMode::TreeOnly;
    assert_eq!(state.count_included_files(&root), 0);

    // Act & Assert: AllCode includes all TextCode files even after clearing checkboxes
    state.clear_all();
    state.export_mode = ExportMode::AllCode;
    assert_eq!(state.count_included_files(&root), 2);
}
