//! UI banner displaying the detected framework and letting the user confirm or override it.

use eframe::egui;

use crate::detector::{ProjectKind, ProjectProfile};

/// Renders the framework detection and user cross-check controls above the project tree.
pub fn render_project_banner(ui: &mut egui::Ui, profile: &mut ProjectProfile) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.strong("Project Type:");

            // Dropdown allowing the user to cross-check and override the detected framework
            let mut selected_kind = profile.active_kind;
            egui::ComboBox::from_id_salt("framework_selector")
                .selected_text(selected_kind.label())
                .show_ui(ui, |ui| {
                    for &kind in ProjectKind::ALL {
                        if ui
                            .selectable_value(&mut selected_kind, kind, kind.label())
                            .clicked()
                        {
                            profile.select_kind(selected_kind);
                        }
                    }
                });

            render_verification_status(ui, profile);
        });

        ui.small(format!(
            "Auto-detected: {} ({})",
            profile.detected_kind.label(),
            profile.evidence
        ));
    });
}

/// Renders either the confirmation prompt button or the verified status badge.
fn render_verification_status(ui: &mut egui::Ui, profile: &mut ProjectProfile) {
    if !profile.is_user_confirmed {
        ui.separator();
        ui.label(format!(
            "Detected as {}. Is this correct?",
            profile.active_kind.label()
        ));
        if ui.button("✔ Confirm").clicked() {
            profile.confirm();
        }
    } else if profile.active_kind == profile.detected_kind {
        ui.colored_label(egui::Color32::LIGHT_GREEN, "✔ Verified");
    } else {
        ui.colored_label(
            egui::Color32::LIGHT_YELLOW,
            format!(
                "✔ User overridden (auto-detected: {})",
                profile.detected_kind.label()
            ),
        );
    }
}
