//! Shared library root for the `root` desktop application and its cross-platform installer.

pub mod branding;
pub mod detector;
pub mod installer;
pub mod markdown;
pub mod scanner;
pub mod selection;
pub mod settings;
pub mod ui;

use std::sync::Arc;

use branding::{build_window_icon, APP_DISPLAY_TITLE, APP_NAME, INSTALLER_WINDOW_TITLE};
use eframe::egui;
use installer::ui::InstallerApp;
use ui::RootApp;

/// Launches the main `root` desktop application window with the `🌿 root` window and taskbar icon.
pub fn run_desktop_app() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_app_id(APP_NAME)
            .with_title(APP_DISPLAY_TITLE)
            .with_inner_size([1024.0, 720.0])
            .with_min_inner_size([640.0, 480.0])
            .with_icon(Arc::new(build_window_icon())),
        ..Default::default()
    };

    eframe::run_native(
        APP_DISPLAY_TITLE,
        native_options,
        Box::new(|_cc| Ok(Box::new(RootApp::default()))),
    )
}

/// Launches the interactive cross-platform `root Setup` window with the `🌿 root` window and taskbar icon.
pub fn run_installer_app() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_app_id(APP_NAME)
            .with_title(INSTALLER_WINDOW_TITLE)
            .with_inner_size([620.0, 540.0])
            .with_min_inner_size([520.0, 460.0])
            .with_resizable(true)
            .with_icon(Arc::new(build_window_icon())),
        ..Default::default()
    };

    eframe::run_native(
        INSTALLER_WINDOW_TITLE,
        native_options,
        Box::new(|_cc| Ok(Box::new(InstallerApp::default()))),
    )
}
