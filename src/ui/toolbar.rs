//! Top toolbar controls for opening a project folder, configuring selection, and exporting `.md`.

use eframe::egui;

use crate::selection::ExportMode;

/// User-initiated actions emitted by the top toolbar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolbarAction {
    /// No button was clicked this frame.
    None,
    /// User clicked the button to pick a new project directory.
    OpenFolder,
    /// User toggled a scan setting or clicked Rescan.
    Rescan,
    /// User clicked "Select All Text Files" in `SelectedCode` mode.
    SelectAllFiles,
    /// User clicked "Clear Selection" in `SelectedCode` mode.
    ClearSelection,
    /// User clicked "Export .md..." to save the Markdown bundle to disk.
    ExportMarkdown,
}

/// Renders the top panel with folder picker, export mode selector, and `.md` export button.
pub fn render_top_toolbar(
    ctx: &egui::Context,
    should_include_hidden: &mut bool,
    export_mode: &mut ExportMode,
    should_show_preview: &mut bool,
    has_loaded_project: bool,
) -> ToolbarAction {
    let mut triggered_action = ToolbarAction::None;

    egui::TopBottomPanel::top("top_toolbar").show(ctx, |ui| {
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            if ui.button("📂 Open Project Folder...").clicked() {
                triggered_action = ToolbarAction::OpenFolder;
            }

            if has_loaded_project && ui.button("🔄 Rescan").clicked() {
                triggered_action = ToolbarAction::Rescan;
            }

            let dotfile_toggle = ui.checkbox(should_include_hidden, "Include dotfiles");
            if dotfile_toggle.changed() && has_loaded_project {
                triggered_action = ToolbarAction::Rescan;
            }

            if has_loaded_project {
                ui.separator();
                render_export_controls(
                    ui,
                    export_mode,
                    should_show_preview,
                    &mut triggered_action,
                );
            }
        });
        ui.add_space(4.0);
    });

    triggered_action
}

/// Renders the export mode dropdown, batch selection buttons, preview toggle, and save button.
fn render_export_controls(
    ui: &mut egui::Ui,
    export_mode: &mut ExportMode,
    should_show_preview: &mut bool,
    triggered_action: &mut ToolbarAction,
) {
    ui.label("Mode:");
    egui::ComboBox::from_id_salt("export_mode_selector")
        .selected_text(export_mode.label())
        .show_ui(ui, |ui| {
            for &mode in ExportMode::ALL {
                ui.selectable_value(export_mode, mode, mode.label());
            }
        });

    if *export_mode == ExportMode::SelectedCode {
        if ui.button("Select All Code").clicked() {
            *triggered_action = ToolbarAction::SelectAllFiles;
        }
        if ui.button("Clear All").clicked() {
            *triggered_action = ToolbarAction::ClearSelection;
        }
    }

    ui.separator();
    ui.checkbox(should_show_preview, "Live .md Preview");

    if ui.button("💾 Export .md...").clicked() {
        *triggered_action = ToolbarAction::ExportMarkdown;
    }
}
