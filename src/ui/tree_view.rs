//! Scrollable collapsible tree view for displaying a scanned `FileNode` hierarchy.

use eframe::egui;

use crate::scanner::{FileCategory, FileNode};
use crate::selection::{ExportMode, SelectionState};

/// Byte threshold for displaying file sizes in kilobytes.
const BYTES_PER_KB: u64 = 1024;

/// Byte threshold for displaying file sizes in megabytes.
const BYTES_PER_MB: u64 = 1024 * 1024;

/// Renders the project header path, included file count, and scrollable recursive directory tree.
pub fn render_project_tree(
    ui: &mut egui::Ui,
    root_node: &FileNode,
    selection: &mut SelectionState,
) {
    ui.horizontal_wrapped(|ui| {
        ui.heading(&root_node.name);
        ui.weak(format!("({})", root_node.full_path.display()));
        ui.separator();
        let included_count = selection.count_included_files(root_node);
        ui.label(format!("Files included in .md code bundle: {included_count}"));
    });
    ui.separator();

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for child in &root_node.children {
                render_tree_node(ui, child, selection, 0);
            }
        });
}

/// Recursively renders a directory with a cascading checkbox or a file row.
fn render_tree_node(
    ui: &mut egui::Ui,
    node: &FileNode,
    selection: &mut SelectionState,
    depth: usize,
) {
    if node.is_dir {
        let is_default_open = depth == 0;
        ui.horizontal(|ui| {
            if selection.export_mode == ExportMode::SelectedCode {
                let mut is_dir_checked = selection.is_directory_checked(node);
                if ui.checkbox(&mut is_dir_checked, "").changed() {
                    selection.set_directory_checked(node, is_dir_checked);
                }
            }

            let folder_label = format!("📁 {}", node.name);
            egui::CollapsingHeader::new(folder_label)
                .id_salt(&node.relative_path)
                .default_open(is_default_open)
                .show(ui, |ui| {
                    for child in &node.children {
                        render_tree_node(ui, child, selection, depth + 1);
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
        if selection.export_mode == ExportMode::SelectedCode && node.category != FileCategory::Binary
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
    use super::{category_badge, format_byte_size};
    use crate::scanner::FileCategory;

    #[test]
    fn formats_bytes_below_one_kilobyte_as_raw_bytes() {
        // Arrange
        let size = 512;

        // Act
        let formatted = format_byte_size(size);

        // Assert
        assert_eq!(formatted, "512 B");
    }

    #[test]
    fn formats_kilobyte_and_megabyte_boundaries_with_one_decimal() {
        // Arrange
        let one_kb = 1024;
        let two_and_half_mb = 2 * 1024 * 1024 + 512 * 1024;

        // Act
        let kb_text = format_byte_size(one_kb);
        let mb_text = format_byte_size(two_and_half_mb);

        // Assert
        assert_eq!(kb_text, "1.0 KB");
        assert_eq!(mb_text, "2.5 MB");
    }

    #[test]
    fn returns_badge_only_for_lockfile_and_binary_categories() {
        // Arrange & Act & Assert
        assert_eq!(category_badge(FileCategory::TextCode), None);
        assert_eq!(category_badge(FileCategory::Lockfile), Some("[lockfile]"));
        assert_eq!(category_badge(FileCategory::Binary), Some("[binary]"));
    }
}
