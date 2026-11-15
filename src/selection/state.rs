//! Pure state management for choosing which source files are bundled into Markdown.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::scanner::{FileCategory, FileNode};

/// Controls how much source code is appended below the directory tree in the exported `.md` file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExportMode {
    /// Exports only the ASCII directory tree structure with no source code blocks.
    TreeOnly,
    /// Exports the directory tree plus all non-binary, non-lockfile source code files.
    AllCode,
    /// Exports the directory tree plus only the files checked by the user in the UI tree.
    #[default]
    SelectedCode,
}

impl ExportMode {
    /// All available export modes for UI toolbar selection.
    pub const ALL: &'static [ExportMode] = &[
        ExportMode::TreeOnly,
        ExportMode::AllCode,
        ExportMode::SelectedCode,
    ];

    /// Human-readable label for toolbar buttons and dropdowns.
    pub fn label(self) -> &'static str {
        match self {
            Self::TreeOnly => "Tree Only",
            Self::AllCode => "Tree + All Code",
            Self::SelectedCode => "Tree + Selected Code",
        }
    }
}

/// Tracks the active `ExportMode` and the set of user-checked file paths.
#[derive(Debug, Clone, Default)]
pub struct SelectionState {
    /// Active code inclusion mode.
    pub export_mode: ExportMode,
    /// Relative paths of files explicitly checked for inclusion when in `SelectedCode` mode.
    selected_files: BTreeSet<PathBuf>,
}

impl SelectionState {
    /// Creates a new selection state pre-populated with all standard text code files in `root`.
    pub fn from_tree(root: &FileNode) -> Self {
        let mut state = Self {
            export_mode: ExportMode::SelectedCode,
            selected_files: BTreeSet::new(),
        };
        state.select_all_text_files(root);
        state
    }

    /// Returns `true` if `node` is a non-binary file that should be embedded in the Markdown output.
    pub fn is_file_included(&self, node: &FileNode) -> bool {
        if node.is_dir || node.category == FileCategory::Binary {
            return false;
        }

        match self.export_mode {
            ExportMode::TreeOnly => false,
            ExportMode::AllCode => node.category == FileCategory::TextCode,
            ExportMode::SelectedCode => self.selected_files.contains(&node.relative_path),
        }
    }

    /// Returns `true` if a specific relative file path is checked in `SelectedCode` mode.
    pub fn is_path_checked(&self, relative_path: &Path) -> bool {
        self.selected_files.contains(relative_path)
    }

    /// Checks or unchecks a single file path.
    pub fn set_file_checked(&mut self, relative_path: PathBuf, is_checked: bool) {
        if is_checked {
            self.selected_files.insert(relative_path);
        } else {
            self.selected_files.remove(&relative_path);
        }
    }

    /// Returns `true` if `dir_node` contains at least one selectable file and all of them are checked.
    pub fn is_directory_checked(&self, dir_node: &FileNode) -> bool {
        let mut has_selectable_files = false;
        let all_checked = self.are_all_descendants_checked(dir_node, &mut has_selectable_files);
        has_selectable_files && all_checked
    }

    /// Recursively checks or unchecks all standard text files inside `dir_node`.
    pub fn set_directory_checked(&mut self, dir_node: &FileNode, should_check: bool) {
        for child in &dir_node.children {
            if child.is_dir {
                self.set_directory_checked(child, should_check);
            } else if child.category != FileCategory::Binary {
                self.set_file_checked(child.relative_path.clone(), should_check);
            }
        }
    }

    /// Selects every `FileCategory::TextCode` file in `root` while skipping binaries and lockfiles.
    pub fn select_all_text_files(&mut self, root: &FileNode) {
        self.selected_files.clear();
        self.collect_text_files_recursively(root);
    }

    /// Unchecks all files.
    pub fn clear_all(&mut self) {
        self.selected_files.clear();
    }

    /// Counts how many files in `root` will be embedded under the current `ExportMode`.
    pub fn count_included_files(&self, root: &FileNode) -> usize {
        if !root.is_dir {
            return usize::from(self.is_file_included(root));
        }

        root.children
            .iter()
            .map(|child| self.count_included_files(child))
            .sum()
    }

    /// Helper that walks `node` and inserts all `TextCode` relative paths.
    fn collect_text_files_recursively(&mut self, node: &FileNode) {
        for child in &node.children {
            if child.is_dir {
                self.collect_text_files_recursively(child);
            } else if child.category == FileCategory::TextCode {
                self.selected_files.insert(child.relative_path.clone());
            }
        }
    }

    /// Helper that checks whether all non-binary descendants of `dir_node` are checked.
    fn are_all_descendants_checked(
        &self,
        dir_node: &FileNode,
        has_selectable_files: &mut bool,
    ) -> bool {
        for child in &dir_node.children {
            if child.is_dir {
                if !self.are_all_descendants_checked(child, has_selectable_files) {
                    return false;
                }
            } else if child.category == FileCategory::TextCode {
                *has_selectable_files = true;
                if !self.selected_files.contains(&child.relative_path) {
                    return false;
                }
            }
        }
        true
    }
}
