//! Visual styling, spacing, and dark/light theme configuration for the `root` UI.

use eframe::egui::{self, Rounding, Vec2, Visuals};

/// Horizontal and vertical padding inside interactive buttons and inputs.
const BUTTON_PADDING: Vec2 = Vec2::new(10.0, 5.0);

/// Spacing between adjacent horizontal and vertical widgets.
const ITEM_SPACING: Vec2 = Vec2::new(8.0, 6.0);

/// Corner rounding radius in logical pixels for buttons, frames, and inputs.
const WIDGET_CORNER_RADIUS: f32 = 6.0;

/// Applies custom spacing, rounded widget corners, and dark or light color visuals to `ctx`.
pub fn apply_app_theme(ctx: &egui::Context, is_dark_mode: bool) {
    let mut visuals = if is_dark_mode {
        Visuals::dark()
    } else {
        Visuals::light()
    };

    // Configure uniform rounded corners across all widget states in egui 0.29
    let rounding = Rounding::same(WIDGET_CORNER_RADIUS);
    visuals.widgets.noninteractive.rounding = rounding;
    visuals.widgets.inactive.rounding = rounding;
    visuals.widgets.hovered.rounding = rounding;
    visuals.widgets.active.rounding = rounding;
    visuals.widgets.open.rounding = rounding;
    visuals.striped = true;

    ctx.set_visuals(visuals);

    ctx.style_mut(|style| {
        style.spacing.button_padding = BUTTON_PADDING;
        style.spacing.item_spacing = ITEM_SPACING;
    });
}
