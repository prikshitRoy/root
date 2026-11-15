//! Top toolbar controls for folder picking, tree search filtering, theme switching, and `.md` export.

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
    /// User clicked "Select All Code" in `SelectedCode` mode.
    SelectAllFiles,
    /// User clicked "Clear All" in `SelectedCode` mode.
    ClearSelection,
    /// User clicked "Copy .md" to copy the generated Markdown to the system clipboard.
    CopyMarkdown,
    /// User clicked "Export .md..." to save the Markdown bundle to disk.
    ExportMarkdown,
    /// User toggled between dark and light UI themes.
    ToggleTheme,
    /// User clicked the Zoom In button (`+`).
    ZoomIn,
    /// User clicked the Zoom Out button (`-`).
    ZoomOut,
    /// User clicked the zoom percentage badge to reset zoom to 100%.
    ResetZoom,
}

/// Mutable UI settings edited directly by controls in the top toolbar.
pub struct ToolbarState<'a> {
    pub should_include_hidden: &'a mut bool,
    pub export_mode: &'a mut ExportMode,
    pub should_show_preview: &'a mut bool,
    pub search_query: &'a mut String,
    pub is_dark_mode: bool,
    pub zoom_factor: f32,
    pub has_loaded_project: bool,
    pub is_all_code_selected: bool,
    pub is_no_code_selected: bool,
    pub is_scan_running: bool,
    pub is_export_running: bool,
}

/// Renders the two-row top toolbar and returns any triggered `ToolbarAction`.
pub fn render_top_toolbar(ctx: &egui::Context, mut state: ToolbarState<'_>) -> ToolbarAction {
    let mut triggered_action = ToolbarAction::None;

    egui::TopBottomPanel::top("top_toolbar").show(ctx, |ui| {
        ui.add_space(6.0);
        render_primary_row(ui, &mut state, &mut triggered_action);

        if state.has_loaded_project && !state.is_scan_running {
            ui.separator();
            render_project_controls_row(ui, state, &mut triggered_action);
        }
        ui.add_space(6.0);
    });

    triggered_action
}

/// Renders the top row with app branding, folder picker, rescan, dotfile checkbox, and theme button.
fn render_primary_row(
    ui: &mut egui::Ui,
    state: &mut ToolbarState<'_>,
    triggered_action: &mut ToolbarAction,
) {
    ui.horizontal_wrapped(|ui| {
        ui.heading("🌿 root");
        ui.separator();

        let open_btn = egui::Button::new("📂 Open Project Folder...");
        if ui.add_enabled(!state.is_scan_running, open_btn).clicked() {
            *triggered_action = ToolbarAction::OpenFolder;
        }

        if state.has_loaded_project {
            let rescan_btn = egui::Button::new("🔄 Rescan");
            if ui.add_enabled(!state.is_scan_running, rescan_btn).clicked() {
                *triggered_action = ToolbarAction::Rescan;
            }
        }

        let dotfile_toggle = ui.checkbox(state.should_include_hidden, "Include dotfiles");
        if dotfile_toggle.changed() && state.has_loaded_project {
            *triggered_action = ToolbarAction::Rescan;
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let theme_label = if state.is_dark_mode {
                "☀ Light"
            } else {
                "🌙 Dark"
            };
            if ui.button(theme_label).clicked() {
                *triggered_action = ToolbarAction::ToggleTheme;
            }

            ui.separator();

            if ui
                .button("➕")
                .on_hover_text("Zoom In (Ctrl + Plus)")
                .clicked()
            {
                *triggered_action = ToolbarAction::ZoomIn;
            }

            let zoom_label = crate::ui::theme::format_zoom_percentage(state.zoom_factor);
            if ui
                .button(zoom_label)
                .on_hover_text("Reset Zoom to 100% (Ctrl + 0)")
                .clicked()
            {
                *triggered_action = ToolbarAction::ResetZoom;
            }

            if ui
                .button("➖")
                .on_hover_text("Zoom Out (Ctrl + Minus)")
                .clicked()
            {
                *triggered_action = ToolbarAction::ZoomOut;
            }
        });
    });
}

/// Renders the second row containing the file filter search box, export mode, and export buttons.
fn render_project_controls_row(
    ui: &mut egui::Ui,
    state: ToolbarState<'_>,
    triggered_action: &mut ToolbarAction,
) {
    ui.horizontal_wrapped(|ui| {
        ui.label("🔍");
        let search_box = egui::TextEdit::singleline(state.search_query)
            .hint_text("Filter files or folders...")
            .desired_width(180.0);
        ui.add(search_box);

        if !state.search_query.is_empty() && ui.small_button("✖").clicked() {
            state.search_query.clear();
        }

        ui.separator();
        ui.label("Bundle:");
        egui::ComboBox::from_id_salt("export_mode_selector")
            .selected_text(state.export_mode.label())
            .show_ui(ui, |ui| {
                for &mode in ExportMode::ALL {
                    ui.selectable_value(state.export_mode, mode, mode.label());
                }
            });

        if ui
            .button("☑ All Code")
            .on_hover_text("Check all non-binary source files in the tree")
            .clicked()
        {
            *triggered_action = ToolbarAction::SelectAllFiles;
        }

        // Dynamically switch between ☑ and ☐ and highlight whichever preset is active
        let all_code_label = format_toggle_button_label("All Code", state.is_all_code_selected);
        let all_code_btn = egui::Button::new(all_code_label).selected(state.is_all_code_selected);
        if ui
            .add(all_code_btn)
            .on_hover_text("Check all non-binary source files in the tree")
            .clicked()
        {
            *triggered_action = ToolbarAction::SelectAllFiles;
        }

        let none_label = format_toggle_button_label("None", state.is_no_code_selected);
        let none_btn = egui::Button::new(none_label).selected(state.is_no_code_selected);
        if ui
            .add(none_btn)
            .on_hover_text("Uncheck all files in the tree")
            .clicked()
        {
            *triggered_action = ToolbarAction::ClearSelection;
        }

        ui.separator();
        ui.checkbox(state.should_show_preview, "👁 Live Preview");

        if ui.button("📋 Copy .md").clicked() {
            *triggered_action = ToolbarAction::CopyMarkdown;
        }

        let export_btn = egui::Button::new("💾 Export .md...");
        if ui
            .add_enabled(!state.is_export_running, export_btn)
            .clicked()
        {
            *triggered_action = ToolbarAction::ExportMarkdown;
        }
    });
}

/// Formats a toolbar toggle button label with a checked (`☑`) or unchecked (`☐`) icon prefix.
pub fn format_toggle_button_label(title: &str, is_checked: bool) -> String {
    let icon = if is_checked { "☑" } else { "☐" };
    format!("{icon} {title}")
}

#[cfg(test)]
mod tests {
    use super::format_toggle_button_label;

    #[test]
    fn formats_toolbar_toggle_labels_with_matching_checkmark_icon() {
        // Arrange & Act & Assert
        assert_eq!(format_toggle_button_label("All Code", true), "☑ All Code");
        assert_eq!(format_toggle_button_label("All Code", false), "☐ All Code");
        assert_eq!(format_toggle_button_label("None", true), "☑ None");
        assert_eq!(format_toggle_button_label("None", false), "☐ None");
    }
}
