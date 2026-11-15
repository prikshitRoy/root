//! Entry point for the `root` desktop application.

use eframe::egui;

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

/// Top-level application state container.
#[derive(Default)]
struct RootApp;

impl eframe::App for RootApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("root");
            ui.label("Select a software project folder to inspect its tree and export Markdown.");
        });
    }
}
