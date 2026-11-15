//! Scrollable collapsible tree view with summary statistics and live search filtering.

use eframe::egui;

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

/// Renders the project header card, summary statistics pills, and filtered directory tree.
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
                    render_tree_node(ui, child, selection, &normalized_query, 0);
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

/// Recursively renders a directory with a cascading checkbox or a file row.
fn render_tree_node(
    ui: &mut egui::Ui,
    node: &FileNode,
    selection: &mut SelectionState,
    normalized_query: &str,
    depth: usize,
) {
    if node.is_dir {
        // Auto-expand folders at the root or whenever a search query is active
        let is_default_open = depth == 0 || !normalized_query.is_empty();
        ui.horizontal(|ui| {
            if selection.export_mode == ExportMode::SelectedCode {
                let mut is_dir_checked = selection.is_directory_checked(node);
                if ui.checkbox(&mut is_dir_checked, "").changed() {
                    selection.set_directory_checked(node, is_dir_checked);
                }
            }

            let folder_label = format!("📁 {} ({})", node.name, node.children.len());
            egui::CollapsingHeader::new(folder_label)
                .id_salt(&node.relative_path)
                .default_open(is_default_open)
                .show(ui, |ui| {
                    for child in &node.children {
                        if node_matches_query(child, normalized_query) {
                            render_tree_node(ui, child, selection, normalized_query, depth + 1);
                        }
                    }
                });
        });
    } else {
        render_file_row(ui, node, selection);
    }
}

/// Renders a single file entry with its selection checkbox, size, and optional category badge.
fn render_file_row(ui: &mut egui::Ui, node: &FileNode, selection: &mut SelectionState) {
    ui.horizontal(|ui| {
        if selection.export_mode == ExportMode::SelectedCode
            && node.category != FileCategory::Binary
        {
            let mut is_checked = selection.is_path_checked(&node.relative_path);
            if ui.checkbox(&mut is_checked, "").changed() {
                selection.set_file_checked(node.relative_path.clone(), is_checked);
            }
        }

        ui.label(format!("📄 {}", node.name));
        ui.weak(format_byte_size(node.size_bytes));

        if let Some(badge) = category_badge(node.category) {
            ui.small(badge);
        }
    });
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

fn accumulate_summary(
    node: &FileNode,
    selection: &SelectionState,
    summary: &mut TreeSummary,
) {
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

    use super::{category_badge, format_byte_size, node_matches_query};
    use crate::scanner::{FileCategory, FileNode};

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
        // Arrange
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

        // Act & Assert
        assert!(node_matches_query(&folder, "navbar"));
        assert!(!node_matches_query(&folder, "footer"));
    }
}
