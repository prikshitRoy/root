//! Bottom status bar and animated `.md` export progress bar.

use eframe::egui;

use crate::markdown::ExportState;

/// Renders the bottom status panel containing the `.md` export progress bar and feedback messages.
pub fn render_bottom_status_bar(
    ctx: &egui::Context,
    export_state: &Option<ExportState>,
    status_message: &Option<String>,
    error_message: &Option<String>,
) {
    egui::TopBottomPanel::bottom("bottom_status_bar").show(ctx, |ui| {
        ui.add_space(4.0);

        if let Some(state) = export_state {
            render_export_progress_row(ui, state);
        }

        ui.horizontal_wrapped(|ui| {
            if let Some(error) = error_message {
                ui.colored_label(egui::Color32::LIGHT_RED, format!("⚠ {error}"));
            } else if let Some(status) = status_message {
                ui.colored_label(egui::Color32::LIGHT_GREEN, format!("✔ {status}"));
            } else if export_state.is_none() {
                ui.weak("Ready — open a project folder to inspect its tree or export Markdown.");
            }
        });

        ui.add_space(4.0);
    });
}

/// Renders the progress bar and file counter corresponding to the current `ExportState`.
fn render_export_progress_row(ui: &mut egui::Ui, state: &ExportState) {
    match state {
        ExportState::Running {
            processed_files,
            total_files,
            current_file,
            fraction,
        } => {
            let progress_text = format!(
                "Exporting .md ({processed_files}/{total_files} files) — {current_file}"
            );
            ui.add(
                egui::ProgressBar::new(*fraction)
                    .text(progress_text)
                    .animate(true),
            );
            ui.add_space(2.0);
        }
        ExportState::Completed {
            saved_path,
            total_files,
        } => {
            let done_text = format!(
                "100% — Exported {total_files} code files to '{}'",
                saved_path.display()
            );
            ui.add(egui::ProgressBar::new(1.0).text(done_text));
            ui.add_space(2.0);
        }
        ExportState::Failed(reason) => {
            ui.colored_label(
                egui::Color32::LIGHT_RED,
                format!("✖ Export failed: {reason}"),
            );
        }
    }
}
