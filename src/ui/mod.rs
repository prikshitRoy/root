//! Top-level UI state and panel orchestration for the `root` desktop app.

mod preview_pane;
mod project_banner;
mod toolbar;
mod tree_view;

use std::path::PathBuf;

use eframe::egui;

use crate::detector::{detect_project_profile, ProjectKind, ProjectProfile};
use crate::markdown::{build_markdown_document, save_markdown_file};
use crate::scanner::{scan_directory, FileNode, ScanOptions};
use crate::selection::SelectionState;
use preview_pane::render_markdown_preview_panel;
use project_banner::render_project_banner;
use toolbar::{render_top_toolbar, ToolbarAction};
use tree_view::render_project_tree;

/// Stateful container for the currently loaded project, selection state, and scan configuration.
#[derive(Default)]
pub struct RootApp {
    /// Path to the currently selected software project folder.
    selected_root: Option<PathBuf>,
    /// Scanned directory tree respecting `.gitignore` and `.ignore` rules.
    scanned_tree: Option<FileNode>,
    /// Auto-detected and user-verified framework profile for the loaded project.
    project_profile: Option<ProjectProfile>,
    /// Tracks export mode and which files are checked for code bundling.
    selection: SelectionState,
    /// Whether the right-hand live Markdown preview panel is open.
    should_show_preview: bool,
    /// Status banner message shown after exporting a `.md` file.
    status_message: Option<String>,
    /// Last error encountered when scanning or exporting, if any.
    error_message: Option<String>,
    /// Active scan options (such as whether hidden dotfiles are included).
    scan_options: ScanOptions,
}

impl RootApp {
    /// Opens the native OS folder dialog and scans the chosen directory.
    fn pick_and_scan_folder(&mut self) {
        if let Some(folder_path) = rfd::FileDialog::new().pick_folder() {
            self.selected_root = Some(folder_path);
            self.rescan_current_folder();
        }
    }

    /// Re-runs the ignore-aware directory scanner and framework detector on the current root path.
    fn rescan_current_folder(&mut self) {
        let Some(root_path) = &self.selected_root else {
            return;
        };

        match scan_directory(root_path, &self.scan_options) {
            Ok(tree) => {
                self.project_profile = Some(detect_project_profile(&tree));
                self.selection = SelectionState::from_tree(&tree);
                self.scanned_tree = Some(tree);
                self.error_message = None;
                self.status_message = None;
            }
            Err(err) => {
                self.scanned_tree = None;
                self.project_profile = None;
                self.error_message = Some(err.to_string());
            }
        }
    }

    /// Opens a native save dialog and writes the generated Markdown bundle to the chosen `.md` file.
    fn export_markdown_dialog(&mut self) {
        let Some(tree) = &self.scanned_tree else {
            return;
        };

        let default_filename = format!("{}-context.md", tree.name);
        let Some(save_path) = rfd::FileDialog::new()
            .add_filter("Markdown", &["md"])
            .set_file_name(&default_filename)
            .save_file()
        else {
            return;
        };

        let active_kind = self
            .project_profile
            .as_ref()
            .map_or(ProjectKind::Unknown, |p| p.active_kind);

        let markdown = build_markdown_document(
            tree,
            active_kind,
            &self.selection,
            self.scan_options.max_file_size_bytes,
        );

        match save_markdown_file(&save_path, &markdown) {
            Ok(()) => {
                self.status_message =
                    Some(format!("Saved Markdown to '{}'", save_path.display()));
                self.error_message = None;
            }
            Err(err) => {
                self.error_message = Some(err);
            }
        }
    }
}

impl eframe::App for RootApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let has_loaded_project = self.selected_root.is_some();

        let action = render_top_toolbar(
            ctx,
            &mut self.scan_options.should_include_hidden,
            &mut self.selection.export_mode,
            &mut self.should_show_preview,
            has_loaded_project,
        );

        match action {
            ToolbarAction::OpenFolder => self.pick_and_scan_folder(),
            ToolbarAction::Rescan => self.rescan_current_folder(),
            ToolbarAction::SelectAllFiles => {
                if let Some(tree) = &self.scanned_tree {
                    self.selection.select_all_text_files(tree);
                }
            }
            ToolbarAction::ClearSelection => self.selection.clear_all(),
            ToolbarAction::ExportMarkdown => self.export_markdown_dialog(),
            ToolbarAction::None => {}
        }

        if self.should_show_preview {
            if let Some(tree) = &self.scanned_tree {
                let kind = self
                    .project_profile
                    .as_ref()
                    .map_or(ProjectKind::Unknown, |p| p.active_kind);
                render_markdown_preview_panel(
                    ctx,
                    tree,
                    kind,
                    &self.selection,
                    self.scan_options.max_file_size_bytes,
                );
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            render_Feedback_banners(ui, &self.error_message, &self.status_message);

            if let Some(profile) = &mut self.project_profile {
                render_project_banner(ui, profile);
                ui.add_space(6.0);
            }

            if let Some(tree) = &self.scanned_tree {
                render_project_tree(ui, tree, &mut self.selection);
            } else {
                render_empty_state(ui);
            }
        });
    }
}

/// Renders error or export success messages at the top of the central panel.
fn render_Feedback_banners(
    ui: &mut egui::Ui,
    error_message: &Option<String>,
    status_message: &Option<String>,
) {
    if let Some(error) = error_message {
        ui.colored_label(egui::Color32::LIGHT_RED, error);
        ui.separator();
    }
    if let Some(status) = status_message {
        ui.colored_label(egui::Color32::LIGHT_GREEN, status);
        ui.separator();
    }
}

/// Renders the centered onboarding prompt when no project folder is loaded yet.
fn render_empty_state(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(120.0);
        ui.heading("No Project Loaded");
        ui.label("Click 'Open Project Folder...' in the top toolbar to scan a software repository.");
    });
}
