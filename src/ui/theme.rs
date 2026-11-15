//! Visual styling, UI zoom scaling, and dark/light theme configuration for the `root` UI.

use eframe::egui::{self, Rounding, Vec2, Visuals};

pub use crate::settings::{
    clamp_zoom, DEFAULT_ZOOM_FACTOR, MAX_ZOOM_FACTOR, MIN_ZOOM_FACTOR, ZOOM_STEP,
};

/// Multiplier to convert a normalized zoom factor into a display percentage.
const PERCENT_MULTIPLIER: f32 = 100.0;

/// Horizontal and vertical padding inside interactive buttons and inputs.
const BUTTON_PADDING: Vec2 = Vec2::new(10.0, 5.0);

/// Spacing between adjacent horizontal and vertical widgets.
const ITEM_SPACING: Vec2 = Vec2::new(8.0, 6.0);

/// Corner rounding radius in logical pixels for buttons, frames, and inputs.
const WIDGET_CORNER_RADIUS: f32 = 6.0;

/// Applies UI zoom scaling, custom spacing, rounded corners, and dark/light visuals to `ctx`.
pub fn apply_app_theme(ctx: &egui::Context, is_dark_mode: bool, zoom_factor: f32) {
    ctx.set_zoom_factor(clamp_zoom(zoom_factor));

    let mut visuals = if is_dark_mode {
        Visuals::dark()
    } else {
        Visuals::light()
    };

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

/// Increases `current_zoom` by `ZOOM_STEP` up to `MAX_ZOOM_FACTOR`.
pub fn step_zoom_in(current_zoom: f32) -> f32 {
    clamp_zoom(current_zoom + ZOOM_STEP)
}

/// Decreases `current_zoom` by `ZOOM_STEP` down to `MIN_ZOOM_FACTOR`.
pub fn step_zoom_out(current_zoom: f32) -> f32 {
    clamp_zoom(current_zoom - ZOOM_STEP)
}

/// Formats `zoom_factor` as a whole-number percentage string (e.g., `"110%"`).
pub fn format_zoom_percentage(zoom_factor: f32) -> String {
    let percent = (clamp_zoom(zoom_factor) * PERCENT_MULTIPLIER).round() as u32;
    format!("{percent}%")
}

#[cfg(test)]
mod tests {
    use super::{
        clamp_zoom, format_zoom_percentage, step_zoom_in, step_zoom_out, DEFAULT_ZOOM_FACTOR,
        MAX_ZOOM_FACTOR, MIN_ZOOM_FACTOR,
    };

    #[test]
    fn steps_zoom_in_and_out_by_ten_percent() {
        // Arrange
        let initial = DEFAULT_ZOOM_FACTOR;

        // Act
        let zoomed_in = step_zoom_in(initial);
        let zoomed_out = step_zoom_out(initial);

        // Assert
        assert_eq!(zoomed_in, 1.1);
        assert_eq!(zoomed_out, 0.9);
    }

    #[test]
    fn clamps_zoom_factor_at_minimum_and_maximum_bounds() {
        // Arrange & Act
        let below_min = step_zoom_out(MIN_ZOOM_FACTOR);
        let above_max = step_zoom_in(MAX_ZOOM_FACTOR);
        let extreme_high = clamp_zoom(5.0);

        // Assert
        assert_eq!(below_min, MIN_ZOOM_FACTOR);
        assert_eq!(above_max, MAX_ZOOM_FACTOR);
        assert_eq!(extreme_high, MAX_ZOOM_FACTOR);
    }

    #[test]
    fn formats_zoom_factor_as_rounded_percentage() {
        // Arrange & Act & Assert
        assert_eq!(format_zoom_percentage(1.0), "100%");
        assert_eq!(format_zoom_percentage(1.2), "120%");
        assert_eq!(format_zoom_percentage(0.8), "80%");
    }
}
