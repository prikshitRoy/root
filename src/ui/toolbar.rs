//! Top toolbar controls for opening a project folder and adjusting scan flags.

use eframe::egui;

/// User-initiated actions emitted by the top toolbar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolbarAction {
    /// No button was clicked this frame.
    None,
    /// User clicked the button to pick a new project directory.
    OpenFolder,
    /// User toggled a scan setting or clicked Rescan.
    Rescan,
}

/// Renders the top panel with folder picker and scan toggles, returning any triggered action.
pub fn render_top_toolbar(
    ctx: &egui::Context,
    should_include_hidden: &mut bool,
    has_loaded_project: bool,
) -> ToolbarAction {
    let mut triggered_action = ToolbarAction::None;

    egui::TopBottomPanel::top("top_toolbar").show(ctx, |ui| {
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            if ui.button("Open Project Folder...").clicked() {
                triggered_action = ToolbarAction::OpenFolder;
            }

            if has_loaded_project && ui.button("Rescan").clicked() {
                triggered_action = ToolbarAction::Rescan;
            }

            ui.separator();

            // Re-scan automatically if the user toggles hidden dotfile visibility
            let checkbox_response = ui.checkbox(should_include_hidden, "Include hidden dotfiles");
            if checkbox_response.changed() && has_loaded_project {
                triggered_action = ToolbarAction::Rescan;
            }
        });
        ui.add_space(4.0);
    });

    triggered_action
}
