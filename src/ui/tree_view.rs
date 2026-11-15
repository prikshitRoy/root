//! Scrollable collapsible tree view with interactive checkboxes, summary stats, and search filtering.

use eframe::egui;
use eframe::egui::collapsing_header::CollapsingState;

use crate::scanner::{FileCategory, FileNode};
use crate::selection::{ExportMode, SelectionState};

/// Byte threshold for displaying file sizes in kilobytes.
const BYTES_PER_KB: u64 = 1024;

/// Byte threshold for displaying file sizes in megabytes.
const BYTES_PER_MB: u64 = 1024 * 1024;

/// Aggregated counts for the currently scanned project tree.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TreeSummary {
    pub total_dirs: usize,
    pub total_files: usize,
    pub included_bytes: u64,
}

/// Renders the project header card, summary statistics pills, and interactive directory tree.
pub fn render_project_tree(
    ui: &mut egui::Ui,
    root_node: &FileNode,
    selection: &mut SelectionState,
    search_query: &str,
) {
    let summary = compute_tree_summary(root_node, selection);
    let included_count = selection.count_included_files(root_node);

    render_summary_header(ui, root_node, summary, included_count);
    ui.add_space(4.0);

    let normalized_query = search_query.trim().to_lowercase();

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let mut has_any_match = false;
            for child in &root_node.children {
                if node_matches_query(child, &normalized_query) {
                    has_any_match = true;
                    render_tree_node(ui, root_node, child, selection, &normalized_query, 0);
                }
            }

            if !has_any_match {
                ui.weak("No files or folders match the current filter.");
            }
        });
}

/// Renders the project root title and summary badges for folders, files, and bundle size.
fn render_summary_header(
    ui: &mut egui::Ui,
    root_node: &FileNode,
    summary: TreeSummary,
    included_count: usize,
) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.heading(format!("📦 {}", root_node.name));
            ui.weak(format!("{}", root_node.full_path.display()));
        });
        ui.horizontal_wrapped(|ui| {
            ui.small(format!("📁 {} folders", summary.total_dirs));
            ui.separator();
            ui.small(format!("📄 {} files", summary.total_files));
            ui.separator();
            ui.small(format!(
                "✨ {} files in .md bundle ({})",
                included_count,
                format_byte_size(summary.included_bytes)
            ));
        });
    });
}

/// Recursively renders a directory with a unique ID and `CollapsingState` or a file row.
fn render_tree_node(
    ui: &mut egui::Ui,
    root_node: &FileNode,
    node: &FileNode,
    selection: &mut SelectionState,
    normalized_query: &str,
    depth: usize,
) {
    // Push unique ID per relative path so checkboxes never collide across rows
    ui.push_id(&node.relative_path, |ui| {
        if node.is_dir {
            let is_default_open = depth == 0 || !normalized_query.is_empty();
            let header_id = ui.make_persistent_id("folder_header");

            CollapsingState::load_with_default_open(ui.ctx(), header_id, is_default_open)
                .show_header(ui, |ui| {
                    let mut is_dir_checked = is_directory_checked_in_ui(node, selection);
                    if ui.checkbox(&mut is_dir_checked, "").changed() {
                        apply_directory_toggle_from_ui(root_node, node, selection, is_dir_checked);
                    }
                    ui.label(format!("📁 {} ({})", node.name, node.children.len()));
                })
                .body(|ui| {
                    for child in &node.children {
                        if node_matches_query(child, normalized_query) {
                            render_tree_node(
                                ui,
                                root_node,
                                child,
                                selection,
                                normalized_query,
                                depth + 1,
                            );
                        }
                    }
                });
        } else {
            render_file_row(ui, root_node, node, selection);
        }
    });
}

/// Renders a single file entry with its selection checkbox, size, and optional category badge.
fn render_file_row(
    ui: &mut egui::Ui,
    root_node: &FileNode,
    node: &FileNode,
    selection: &mut SelectionState,
) {
    ui.horizontal(|ui| {
        if node.category == FileCategory::Binary {
            let mut disabled_checked = false;
            ui.add_enabled(false, egui::Checkbox::new(&mut disabled_checked, ""));
        } else {
            let mut is_checked = selection.is_file_included(node);
            if ui.checkbox(&mut is_checked, "").changed() {
                apply_file_toggle_from_ui(root_node, node, selection, is_checked);
            }
        }

        ui.label(format!("📄 {}", node.name));
        ui.weak(format_byte_size(node.size_bytes));

        if let Some(badge) = category_badge(node.category) {
            ui.small(badge);
        }
    });
}

/// Returns whether a directory checkbox should display as checked under the current `ExportMode`.
pub fn is_directory_checked_in_ui(dir_node: &FileNode, selection: &SelectionState) -> bool {
    match selection.export_mode {
        ExportMode::TreeOnly => false,
        ExportMode::AllCode => dir_node.children.iter().any(|c| {
            if c.is_dir {
                is_directory_checked_in_ui(c, selection)
            } else {
                c.category == FileCategory::TextCode
            }
        }),
        ExportMode::SelectedCode => selection.is_directory_checked(dir_node),
    }
}

/// Ensures `selection` is in `SelectedCode` mode and toggles all files inside `dir_node`.
pub fn apply_directory_toggle_from_ui(
    root_node: &FileNode,
    dir_node: &FileNode,
    selection: &mut SelectionState,
    should_check: bool,
) {
    sync_selection_set_before_manual_toggle(root_node, selection);
    selection.set_directory_checked(dir_node, should_check);
}

/// Ensures `selection` is in `SelectedCode` mode and toggles `file_node`.
pub fn apply_file_toggle_from_ui(
    root_node: &FileNode,
    file_node: &FileNode,
    selection: &mut SelectionState,
    should_check: bool,
) {
    sync_selection_set_before_manual_toggle(root_node, selection);
    selection.set_file_checked(file_node.relative_path.clone(), should_check);
}

/// Transitions `AllCode` or `TreeOnly` into `SelectedCode` before applying a manual checkbox click.
fn sync_selection_set_before_manual_toggle(root_node: &FileNode, selection: &mut SelectionState) {
    match selection.export_mode {
        ExportMode::AllCode => {
            selection.select_all_text_files(root_node);
            selection.export_mode = ExportMode::SelectedCode;
        }
        ExportMode::TreeOnly => {
            selection.clear_all();
            selection.export_mode = ExportMode::SelectedCode;
        }
        ExportMode::SelectedCode => {}
    }
}

/// Returns `true` if `node` or any of its descendants match `normalized_query` (lowercase).
pub fn node_matches_query(node: &FileNode, normalized_query: &str) -> bool {
    if normalized_query.is_empty() {
        return true;
    }

    if node.name.to_lowercase().contains(normalized_query) {
        return true;
    }

    node.children
        .iter()
        .any(|child| node_matches_query(child, normalized_query))
}

/// Computes total directory count, total file count, and included byte size for `root_node`.
pub fn compute_tree_summary(root_node: &FileNode, selection: &SelectionState) -> TreeSummary {
    let mut summary = TreeSummary::default();
    accumulate_summary(root_node, selection, &mut summary);
    summary
}

fn accumulate_summary(node: &FileNode, selection: &SelectionState, summary: &mut TreeSummary) {
    for child in &node.children {
        if child.is_dir {
            summary.total_dirs += 1;
            accumulate_summary(child, selection, summary);
        } else {
            summary.total_files += 1;
            if selection.is_file_included(child) {
                summary.included_bytes += child.size_bytes;
            }
        }
    }
}

/// Formats a raw byte count into a compact human-readable string (`B`, `KB`, or `MB`).
pub fn format_byte_size(size_bytes: u64) -> String {
    if size_bytes >= BYTES_PER_MB {
        format!("{:.1} MB", size_bytes as f64 / BYTES_PER_MB as f64)
    } else if size_bytes >= BYTES_PER_KB {
        format!("{:.1} KB", size_bytes as f64 / BYTES_PER_KB as f64)
    } else {
        format!("{size_bytes} B")
    }
}

/// Returns an informational tag for non-standard code files (lockfiles and binaries).
pub fn category_badge(category: FileCategory) -> Option<&'static str> {
    match category {
        FileCategory::TextCode => None,
        FileCategory::Lockfile => Some("[lockfile]"),
        FileCategory::Binary => Some("[binary]"),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        apply_directory_toggle_from_ui, apply_file_toggle_from_ui, category_badge,
        format_byte_size, node_matches_query,
    };
    use crate::scanner::{FileCategory, FileNode};
    use crate::selection::{ExportMode, SelectionState};

    #[test]
    fn formats_bytes_below_one_kilobyte_as_raw_bytes() {
        assert_eq!(format_byte_size(512), "512 B");
    }

    #[test]
    fn formats_kilobyte_and_megabyte_boundaries_with_one_decimal() {
        let one_kb = 1024;
        let two_and_half_mb = 2 * 1024 * 1024 + 512 * 1024;

        assert_eq!(format_byte_size(one_kb), "1.0 KB");
        assert_eq!(format_byte_size(two_and_half_mb), "2.5 MB");
    }

    #[test]
    fn returns_badge_only_for_lockfile_and_binary_categories() {
        assert_eq!(category_badge(FileCategory::TextCode), None);
        assert_eq!(category_badge(FileCategory::Lockfile), Some("[lockfile]"));
        assert_eq!(category_badge(FileCategory::Binary), Some("[binary]"));
    }

    #[test]
    fn matches_directory_when_nested_child_matches_search_query() {
        let folder = FileNode {
            name: "components".to_string(),
            relative_path: PathBuf::from("src/components"),
            full_path: PathBuf::from("/app/src/components"),
            is_dir: true,
            size_bytes: 0,
            category: FileCategory::TextCode,
            children: vec![FileNode {
                name: "Navbar.tsx".to_string(),
                relative_path: PathBuf::from("src/components/Navbar.tsx"),
                full_path: PathBuf::from("/app/src/components/Navbar.tsx"),
                is_dir: false,
                size_bytes: 200,
                category: FileCategory::TextCode,
                children: Vec::new(),
            }],
        };

        assert!(node_matches_query(&folder, "navbar"));
        assert!(!node_matches_query(&folder, "footer"));
    }

    #[test]
    fn switches_to_selected_code_mode_when_toggling_file_or_folder_in_ui() {
        // Arrange
        let file_a = FileNode {
            name: "a.rs".to_string(),
            relative_path: PathBuf::from("src/a.rs"),
            full_path: PathBuf::from("/app/src/a.rs"),
            is_dir: false,
            size_bytes: 100,
            category: FileCategory::TextCode,
            children: Vec::new(),
        };
        let src_dir = FileNode {
            name: "src".to_string(),
            relative_path: PathBuf::from("src"),
            full_path: PathBuf::from("/app/src"),
            is_dir: true,
            size_bytes: 0,
            category: FileCategory::TextCode,
            children: vec![file_a.clone()],
        };
        let root = FileNode {
            name: "app".to_string(),
            relative_path: PathBuf::new(),
            full_path: PathBuf::from("/app"),
            is_dir: true,
            size_bytes: 0,
            category: FileCategory::TextCode,
            children: vec![src_dir.clone()],
        };

        let mut selection = SelectionState::from_tree(&root);
        selection.export_mode = ExportMode::TreeOnly;

        // Act: user clicks the file checkbox while in TreeOnly mode
        apply_file_toggle_from_ui(&root, &file_a, &mut selection, true);

        // Assert: mode switches to SelectedCode and the file is included
        assert_eq!(selection.export_mode, ExportMode::SelectedCode);
        assert!(selection.is_file_included(&file_a));

        // Act: user unchecks the parent folder in UI
        apply_directory_toggle_from_ui(&root, &src_dir, &mut selection, false);

        // Assert
        assert!(!selection.is_file_included(&file_a));
    }
}
