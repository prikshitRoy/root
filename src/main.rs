//! Entry point for the `root` desktop application.

pub mod detector;
pub mod scanner;
pub mod ui;

use eframe::egui;
use ui::RootApp;

/// Launches the native desktop window with default configuration.
fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 720.0])
            .with_min_inner_size([640.0, 480.0]),
        ..Default::default()
    };

    eframe::run_native(
        "root",
        native_options,
        Box::new(|_cc| Ok(Box::new(RootApp::default()))),
    )
}
