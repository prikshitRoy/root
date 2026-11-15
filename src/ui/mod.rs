//! Top-level UI state and panel orchestration for the `root` desktop app.

mod toolbar;
mod tree_view;

use std::path::PathBuf;

use eframe::egui;

use crate::scanner::{scan_directory, FileNode, ScanOptions};
use toolbar::{render_top_toolbar, ToolbarAction};
use tree_view::render_project_tree;

/// Stateful container for the currently loaded project and scan configuration.
#[derive(Default)]
pub struct RootApp {
    /// Path to the currently selected software project folder.
    selected_root: Option<PathBuf>,
    /// Scanned directory tree respecting `.gitignore` and `.ignore` rules.
    scanned_tree: Option<FileNode>,
    /// Last error encountered when scanning a folder, if any.
    error_message: Option<String>,
    /// Active scan options (such as whether hidden dotfiles are included).
    scan_options: ScanOptions,
}

impl RootApp {
    /// Opens the native OS folder dialog and scans the chosen directory.
    fn pick_and_scan_folder(&mut self) {
        // Trigger native OS directory picker without blocking if the user cancels
        if let Some(folder_path) = rfd::FileDialog::new().pick_folder() {
            self.selected_root = Some(folder_path);
            self.rescan_current_folder();
        }
    }

    /// Re-runs the ignore-aware directory scanner on the currently selected root path.
    fn rescan_current_folder(&mut self) {
        let Some(root_path) = &self.selected_root else {
            return;
        };

        match scan_directory(root_path, &self.scan_options) {
            Ok(tree) => {
                self.scanned_tree = Some(tree);
                self.error_message = None;
            }
            Err(err) => {
                self.scanned_tree = None;
                self.error_message = Some(err.to_string());
            }
        }
    }
}

impl eframe::App for RootApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let has_loaded_project = self.selected_root.is_some();

        // Render top action bar and handle user button clicks
        let action = render_top_toolbar(
            ctx,
            &mut self.scan_options.should_include_hidden,
            has_loaded_project,
        );

        match action {
            ToolbarAction::OpenFolder => self.pick_and_scan_folder(),
            ToolbarAction::Rescan => self.rescan_current_folder(),
            ToolbarAction::None => {}
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(error) = &self.error_message {
                ui.colored_label(egui::Color32::LIGHT_RED, error);
                ui.separator();
            }

            if let Some(tree) = &self.scanned_tree {
                render_project_tree(ui, tree);
            } else {
                render_empty_state(ui);
            }
        });
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
