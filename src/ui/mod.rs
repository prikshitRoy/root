//! Top-level UI state and panel orchestration for the `root` desktop app.

mod loading_view;
mod preview_pane;
mod project_banner;
mod status_bar;
mod theme;
mod toolbar;
mod tree_view;

use std::path::PathBuf;

use eframe::egui;

pub use theme::apply_app_theme;

use crate::detector::{detect_project_profile, ProjectKind, ProjectProfile};
use crate::markdown::{build_markdown_document, spawn_export_job, ExportJobHandle, ExportState};
use crate::scanner::{spawn_scan_job, FileNode, ScanJobHandle, ScanOptions, ScanState};
use crate::selection::{ExportMode, SelectionState};
use crate::settings::{load_app_settings, save_app_settings, AppSettings};
use loading_view::render_project_loading_view;
use preview_pane::render_markdown_preview_panel;
use project_banner::render_project_banner;
use status_bar::render_bottom_status_bar;
use theme::{step_zoom_in, step_zoom_out, DEFAULT_ZOOM_FACTOR};
use toolbar::{render_top_toolbar, ToolbarAction, ToolbarState};
use tree_view::render_project_tree;

/// Stateful container for the currently loaded project, background jobs, and UI preferences.
pub struct RootApp {
    /// Path to the currently selected software project folder.
    selected_root: Option<PathBuf>,
    /// Scanned directory tree respecting `.gitignore` and `.ignore` rules.
    scanned_tree: Option<FileNode>,
    /// Auto-detected and user-verified framework profile for the loaded project.
    project_profile: Option<ProjectProfile>,
    /// Tracks export mode and which files are checked for code bundling.
    selection: SelectionState,
    /// Case-insensitive filter string for narrowing down visible tree nodes.
    search_query: String,
    /// Whether the right-hand live Markdown preview panel is open.
    should_show_preview: bool,
    /// Whether dark theme visuals are active (`true` by default).
    is_dark_mode: bool,
    /// Current UI scale factor (`1.0` = 100%).
    zoom_factor: f32,
    /// Handle to an active background directory scan job, if a project is currently loading.
    active_scan_job: Option<ScanJobHandle>,
    /// Number of entries discovered so far by the active directory scan job.
    scan_entries_count: usize,
    /// Relative path currently being inspected by the active directory scan job.
    scan_current_path: String,
    /// Handle to an active background `.md` export job, if one is running.
    active_export_job: Option<ExportJobHandle>,
    /// Latest progress state received from the background export worker.
    export_state: Option<ExportState>,
    /// Status banner message shown in the bottom bar.
    status_message: Option<String>,
    /// Last error encountered when scanning or exporting, if any.
    error_message: Option<String>,
    /// Active scan options (such as whether hidden dotfiles are included).
    scan_options: ScanOptions,
}

impl Default for RootApp {
    fn default() -> Self {
        let saved_settings = load_app_settings();

        Self {
            selected_root: None,
            scanned_tree: None,
            project_profile: None,
            selection: SelectionState::default(),
            search_query: String::new(),
            should_show_preview: false,
            is_dark_mode: saved_settings.is_dark_mode,
            zoom_factor: saved_settings.zoom_factor,
            active_scan_job: None,
            scan_entries_count: 0,
            scan_current_path: String::new(),
            active_export_job: None,
            export_state: None,
            status_message: None,
            error_message: None,
            scan_options: ScanOptions::default(),
        }
    }
}

impl RootApp {
    /// Opens the native OS folder dialog and starts a background scan for the chosen directory.
    fn pick_and_scan_folder(&mut self) {
        if let Some(folder_path) = rfd::FileDialog::new().pick_folder() {
            self.selected_root = Some(folder_path);
            self.start_project_scan_job();
        }
    }

    /// Spawns a non-blocking background directory scan job for `self.selected_root`.
    fn start_project_scan_job(&mut self) {
        let Some(root_path) = &self.selected_root else {
            return;
        };

        self.scan_entries_count = 0;
        self.scan_current_path = "Starting scan...".to_string();
        self.error_message = None;
        self.export_state = None;
        self.status_message = Some("Scanning project directory...".to_string());

        self.active_scan_job = Some(spawn_scan_job(root_path.clone(), self.scan_options.clone()));
    }

    /// Polls the background directory scan worker and updates loading progress or finishes loading.
    fn poll_scan_progress(&mut self, ctx: &egui::Context) {
        let Some(job) = &self.active_scan_job else {
            return;
        };

        if let Some(state) = job.poll_latest() {
            match state {
                ScanState::Running {
                    entries_scanned,
                    current_path,
                } => {
                    self.scan_entries_count = entries_scanned;
                    self.scan_current_path = current_path;
                    ctx.request_repaint();
                }
                ScanState::Completed(tree) => {
                    self.project_profile = Some(detect_project_profile(&tree));
                    self.selection = SelectionState::from_tree(&tree);
                    self.scanned_tree = Some(tree);
                    self.active_scan_job = None;
                    self.status_message = Some("Project loaded.".to_string());
                }
                ScanState::Failed(err) => {
                    self.scanned_tree = None;
                    self.project_profile = None;
                    self.active_scan_job = None;
                    self.error_message = Some(err.to_string());
                }
            }
        } else {
            ctx.request_repaint();
        }
    }

    /// Starts a non-blocking background `.md` export job after the user picks a destination file.
    fn start_export_markdown_job(&mut self) {
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

        let active_kind = self.active_project_kind();
        self.error_message = None;
        self.status_message = None;

        self.active_export_job = Some(spawn_export_job(
            tree.clone(),
            active_kind,
            self.selection.clone(),
            self.scan_options.max_file_size_bytes,
            save_path,
        ));
    }

    /// Copies the generated Markdown bundle directly to the system clipboard.
    fn copy_markdown_to_clipboard(&mut self, ctx: &egui::Context) {
        let Some(tree) = &self.scanned_tree else {
            return;
        };
        let markdown = build_markdown_document(
            tree,
            self.active_project_kind(),
            &self.selection,
            self.scan_options.max_file_size_bytes,
        );
        ctx.copy_text(markdown);
        self.status_message = Some("Copied Markdown bundle to clipboard.".to_string());
        self.error_message = None;
    }

    /// Polls the background export channel and requests continuous UI repaints while running.
    fn poll_export_progress(&mut self, ctx: &egui::Context) {
        let Some(job) = &self.active_export_job else {
            return;
        };

        if let Some(latest_state) = job.poll_latest() {
            match &latest_state {
                ExportState::Running { .. } => {
                    ctx.request_repaint();
                }
                ExportState::Completed { saved_path, .. } => {
                    self.status_message =
                        Some(format!("Saved Markdown to '{}'", saved_path.display()));
                    self.active_export_job = None;
                }
                ExportState::Failed(err) => {
                    self.error_message = Some(err.clone());
                    self.active_export_job = None;
                }
            }
            self.export_state = Some(latest_state);
        } else {
            ctx.request_repaint();
        }
    }

    /// Returns the currently active `ProjectKind` or `ProjectKind::Unknown`.
    fn active_project_kind(&self) -> ProjectKind {
        self.project_profile
            .as_ref()
            .map_or(ProjectKind::Unknown, |p| p.active_kind)
    }
}

impl eframe::App for RootApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_zoom_shortcuts(ctx);
        apply_app_theme(ctx, self.is_dark_mode, self.zoom_factor);
        self.poll_scan_progress(ctx);
        self.poll_export_progress(ctx);
        let (is_all_code_selected, is_no_code_selected) =
            self.scanned_tree.as_ref().map_or((false, false), |tree| {
                (
                    self.selection.is_all_code_selected(tree),
                    self.selection.is_no_code_selected(tree),
                )
            });

        let action = render_top_toolbar(
            ctx,
            ToolbarState {
                should_include_hidden: &mut self.scan_options.should_include_hidden,
                export_mode: &mut self.selection.export_mode,
                should_show_preview: &mut self.should_show_preview,
                search_query: &mut self.search_query,
                is_dark_mode: self.is_dark_mode,
                zoom_factor: self.zoom_factor,
                has_loaded_project: self.selected_root.is_some(),
                is_all_code_selected,
                is_no_code_selected,
                is_scan_running: self.active_scan_job.is_some(),
                is_export_running: self.active_export_job.is_some(),
            },
        );

        match action {
            ToolbarAction::OpenFolder => self.pick_and_scan_folder(),
            ToolbarAction::Rescan => self.start_project_scan_job(),
            ToolbarAction::SelectAllFiles => {
                if let Some(tree) = &self.scanned_tree {
                    self.selection.export_mode = ExportMode::SelectedCode;
                    self.selection.select_all_text_files(tree);
                }
            }
            ToolbarAction::ClearSelection => {
                self.selection.export_mode = ExportMode::SelectedCode;
                self.selection.clear_all();
            }
            ToolbarAction::CopyMarkdown => self.copy_markdown_to_clipboard(ctx),
            ToolbarAction::ExportMarkdown => self.start_export_markdown_job(),
            ToolbarAction::ToggleTheme => {
                self.is_dark_mode = !self.is_dark_mode;
                self.persist_ui_settings();
            }
            ToolbarAction::ZoomIn => {
                self.zoom_factor = step_zoom_in(self.zoom_factor);
                self.persist_ui_settings();
            }
            ToolbarAction::ZoomOut => {
                self.zoom_factor = step_zoom_out(self.zoom_factor);
                self.persist_ui_settings();
            }
            ToolbarAction::ResetZoom => {
                self.zoom_factor = DEFAULT_ZOOM_FACTOR;
                self.persist_ui_settings();
            }
            ToolbarAction::None => {}
        }

        render_bottom_status_bar(
            ctx,
            &self.export_state,
            &self.status_message,
            &self.error_message,
        );

        if self.should_show_preview && self.active_scan_job.is_none() {
            if let Some(tree) = &self.scanned_tree {
                render_markdown_preview_panel(
                    ctx,
                    tree,
                    self.active_project_kind(),
                    &self.selection,
                    self.scan_options.max_file_size_bytes,
                );
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.active_scan_job.is_some() {
                render_project_loading_view(
                    ui,
                    self.selected_root.as_deref(),
                    self.scan_entries_count,
                    &self.scan_current_path,
                );
                return;
            }

            if let Some(profile) = &mut self.project_profile {
                render_project_banner(ui, profile);
                ui.add_space(6.0);
            }

            if let Some(tree) = &self.scanned_tree {
                render_project_tree(ui, tree, &mut self.selection, &self.search_query);
            } else if self.render_onboarding_card(ui) {
                self.pick_and_scan_folder();
            }
        });
    }
}

impl RootApp {
    /// Saves the current `is_dark_mode` and `zoom_factor` preferences to disk.
    fn persist_ui_settings(&mut self) {
        let current = AppSettings {
            is_dark_mode: self.is_dark_mode,
            zoom_factor: self.zoom_factor,
        };
        if let Err(err) = save_app_settings(&current) {
            self.error_message = Some(err);
        }
    }

    /// Checks for `Ctrl + Plus/Equals`, `Ctrl + Minus`, and `Ctrl + 0` keyboard shortcuts.
    fn handle_zoom_shortcuts(&mut self, ctx: &egui::Context) {
        let mut did_zoom_change = false;
        ctx.input(|input| {
            if !input.modifiers.command {
                return;
            }
            if input.key_pressed(egui::Key::Plus) || input.key_pressed(egui::Key::Equals) {
                self.zoom_factor = step_zoom_in(self.zoom_factor);
                did_zoom_change = true;
            } else if input.key_pressed(egui::Key::Minus) {
                self.zoom_factor = step_zoom_out(self.zoom_factor);
                did_zoom_change = true;
            } else if input.key_pressed(egui::Key::Num0) {
                self.zoom_factor = DEFAULT_ZOOM_FACTOR;
                did_zoom_change = true;
            }
        });

        if did_zoom_change {
            self.persist_ui_settings();
        }
    }

    /// Renders the centered onboarding card and returns `true` if the user clicked the open button.
    fn render_onboarding_card(&self, ui: &mut egui::Ui) -> bool {
        let mut should_open = false;
        ui.vertical_centered(|ui| {
            ui.add_space(100.0);
            egui::Frame::group(ui.style())
                .inner_margin(24.0)
                .show(ui, |ui| {
                    ui.heading("🌿 Welcome to root");
                    ui.add_space(6.0);
                    ui.label(
                        "Inspect software project trees (Next.js, Vite, WXT, Tauri, Electron, Rust)\n\
                         respecting .gitignore & .ignore rules, and bundle source code into Markdown.",
                    );
                    ui.add_space(12.0);
                    if ui.button("📂 Choose a Project Folder...").clicked() {
                        should_open = true;
                    }
                });
        });
        should_open
    }
}
