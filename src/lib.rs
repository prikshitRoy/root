//! Shared library root for the `root` desktop application and its cross-platform installer.

pub mod detector;
pub mod installer;
pub mod markdown;
pub mod scanner;
pub mod selection;
pub mod settings;
pub mod ui;

use eframe::egui;
use installer::ui::InstallerApp;
use ui::RootApp;

/// Launches the main `root` desktop application window.
pub fn run_desktop_app() -> eframe::Result<()> {
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

/// Launches the interactive cross-platform `root Setup` installer window.
pub fn run_installer_app() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("root Setup — Cross-Platform Installer")
            .with_inner_size([620.0, 540.0])
            .with_min_inner_size([520.0, 460.0])
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "root Setup",
        native_options,
        Box::new(|_cc| Ok(Box::new(InstallerApp::default()))),
    )
}
