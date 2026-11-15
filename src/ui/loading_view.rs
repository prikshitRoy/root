//! Centered loading UI displayed while a software project directory is being scanned.

use std::path::Path;

use eframe::egui;

/// Renders an animated loading card showing live entry counts and the current path being scanned.
pub fn render_project_loading_view(
    ui: &mut egui::Ui,
    project_path: Option<&Path>,
    entries_scanned: usize,
    current_path: &str,
) {
    ui.vertical_centered(|ui| {
        ui.add_space(100.0);

        egui::Frame::group(ui.style())
            .inner_margin(28.0)
            .show(ui, |ui| {
                ui.set_max_width(480.0);
                ui.vertical_centered(|ui| {
                    ui.add(egui::Spinner::new().size(32.0));
                    ui.add_space(12.0);

                    ui.heading("Scanning Project Structure...");
                    if let Some(root) = project_path {
                        ui.weak(format!("{}", root.display()));
                    }

                    ui.add_space(12.0);
                    let status_label = format!("{entries_scanned} files & folders discovered");
                    ui.add(
                        egui::ProgressBar::new(0.99)
                            .text(status_label)
                            .animate(true),
                    );

                    ui.add_space(8.0);
                    ui.small(format!("Inspecting: {current_path}"));
                });
            });
    });
}
