//! Unit and integration tests for tree formatting and Markdown code bundling.

use std::fs;
use std::path::Path;

use tempfile::TempDir;

use crate::detector::ProjectKind;
use crate::markdown::{
    build_markdown_document, calculate_progress_fraction, code_fence_for_content,
    collect_exportable_files, format_ascii_tree, language_tag_for_path, save_markdown_file,
};
use crate::scanner::{scan_directory, ScanOptions, DEFAULT_MAX_FILE_SIZE_BYTES};
use crate::selection::{ExportMode, SelectionState};

#[test]
fn formats_nested_directory_structure_into_box_drawing_tree() {
    // Arrange
    let temp = TempDir::new().unwrap();
    fs::create_dir_all(temp.path().join("src")).unwrap();
    fs::write(temp.path().join("src/main.rs"), "fn main() {}").unwrap();
    fs::write(temp.path().join("Cargo.toml"), "[package]").unwrap();

    let root = scan_directory(temp.path(), &ScanOptions::default()).unwrap();

    // Act
    let tree_text = format_ascii_tree(&root);

    // Assert
    let expected = format!(
        "{}/\n├── src/\n│   └── main.rs\n└── Cargo.toml",
        root.name
    );
    assert_eq!(tree_text, expected);
}

#[test]
fn expands_code_fence_when_source_contains_triple_backticks() {
    // Arrange
    let markdown_with_fence = "Here is an example:\n```rust\nfn main() {}\n```";

    // Act
    let fence = code_fence_for_content(markdown_with_fence);

    // Assert
    assert_eq!(fence, "````");
}

#[test]
fn maps_common_extensions_to_markdown_syntax_tags() {
    // Arrange & Act & Assert
    assert_eq!(language_tag_for_path(Path::new("src/main.rs")), "rust");
    assert_eq!(language_tag_for_path(Path::new("app/page.tsx")), "tsx");
    assert_eq!(language_tag_for_path(Path::new("Cargo.toml")), "toml");
    assert_eq!(language_tag_for_path(Path::new("unknown.xyz")), "text");
}

#[test]
fn builds_and_saves_markdown_respecting_selected_files() {
    // Arrange
    let temp = TempDir::new().unwrap();
    fs::create_dir_all(temp.path().join("src")).unwrap();
    fs::write(temp.path().join("src/main.rs"), "fn main() {}").unwrap();
    fs::write(temp.path().join("README.md"), "# Demo").unwrap();

    let root = scan_directory(temp.path(), &ScanOptions::default()).unwrap();
    let mut selection = SelectionState::from_tree(&root);

    // Uncheck README.md so only src/main.rs remains selected
    selection.set_file_checked(Path::new("README.md").to_path_buf(), false);

    // Act
    let md_output = build_markdown_document(
        &root,
        ProjectKind::RustCargo,
        &selection,
        DEFAULT_MAX_FILE_SIZE_BYTES,
    );
    let output_path = temp.path().join("EXPORT.md");
    save_markdown_file(&output_path, &md_output).unwrap();
    let saved_content = fs::read_to_string(&output_path).unwrap();

    // Assert: both files appear in the tree, but only `src/main.rs` has a code block
    assert!(saved_content.contains("README.md"));
    assert!(saved_content.contains("### `src/main.rs`"));
    assert!(!saved_content.contains("### `README.md`"));

    // Act & Assert: switching to TreeOnly omits the Source Files section entirely
    selection.export_mode = ExportMode::TreeOnly;
    let tree_only_md = build_markdown_document(
        &root,
        ProjectKind::RustCargo,
        &selection,
        DEFAULT_MAX_FILE_SIZE_BYTES,
    );
    assert!(!tree_only_md.contains("## Source Files"));
}

#[test]
fn calculates_progress_fraction_safely_including_zero_total() {
    // Arrange & Act & Assert
    assert_eq!(calculate_progress_fraction(0, 0), 1.0);
    assert_eq!(calculate_progress_fraction(2, 4), 0.5);
    assert_eq!(calculate_progress_fraction(10, 4), 1.0);
}

#[test]
fn collects_only_exportable_files_matching_selection_mode() {
    // Arrange
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("a.rs"), "pub fn a() {}").unwrap();
    fs::write(temp.path().join("b.rs"), "pub fn b() {}").unwrap();

    let root = scan_directory(temp.path(), &ScanOptions::default()).unwrap();
    let mut selection = SelectionState::from_tree(&root);
    selection.set_file_checked(Path::new("b.rs").to_path_buf(), false);

    // Act
    let selected_list = collect_exportable_files(&root, &selection, DEFAULT_MAX_FILE_SIZE_BYTES);
    selection.export_mode = ExportMode::TreeOnly;
    let tree_only_list = collect_exportable_files(&root, &selection, DEFAULT_MAX_FILE_SIZE_BYTES);

    // Assert
    assert_eq!(selected_list.len(), 1);
    assert_eq!(selected_list[0].name, "a.rs");
    assert!(tree_only_list.is_empty());
}
