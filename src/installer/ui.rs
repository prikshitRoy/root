//! Interactive `eframe`/`egui` Setup Wizard UI for installing, updating, and uninstalling `root`.

use std::path::PathBuf;
use std::process::Command;

use eframe::egui;

use crate::installer::{
    default_install_dir_for_os, detect_install_status, install_application, resolve_install_paths,
    resolve_os_home_dir, resolve_source_app_binary, uninstall_application, InstallOptions,
    InstallStatus, TargetOs,
};
use crate::settings::load_app_settings;
use crate::ui::apply_app_theme;

/// Stateful model for the cross-platform `root Setup` GUI wizard.
pub struct InstallerApp {
    target_os: TargetOs,
    home_dir: PathBuf,
    options: InstallOptions,
    install_status: InstallStatus,
    is_dark_mode: bool,
    zoom_factor: f32,
    status_message: Option<String>,
    error_message: Option<String>,
}

impl Default for InstallerApp {
    fn default() -> Self {
        let saved_settings = load_app_settings();
        let target_os = TargetOs::current();
        let home_dir = resolve_os_home_dir();
        let options = InstallOptions::default();
        let install_status = detect_install_status(target_os, &home_dir, &options.install_dir);

        Self {
            target_os,
            home_dir,
            options,
            install_status,
            is_dark_mode: saved_settings.is_dark_mode,
            zoom_factor: saved_settings.zoom_factor,
            status_message: None,
            error_message: None,
        }
    }
}

impl InstallerApp {
    fn refresh_status(&mut self) {
        self.install_status =
            detect_install_status(self.target_os, &self.home_dir, &self.options.install_dir);
    }

    fn run_install(&mut self) {
        self.error_message = None;
        self.status_message = None;

        let Ok(current_exe) = std::env::current_exe() else {
            self.error_message = Some("Unable to determine current installer path.".to_string());
            return;
        };

        let source_binary = resolve_source_app_binary(&current_exe);
        match install_application(
            self.target_os,
            &self.home_dir,
            &source_binary,
            &self.options,
        ) {
            Ok(receipt) => {
                self.status_message = Some(format!(
                    "Installed root v{} to '{}'",
                    receipt.version,
                    receipt.installed_binary.display()
                ));
                self.refresh_status();
            }
            Err(err) => {
                self.error_message = Some(err.to_string());
            }
        }
    }

    fn run_uninstall(&mut self) {
        self.error_message = None;
        self.status_message = None;

        match uninstall_application(self.target_os, &self.home_dir, &self.options.install_dir) {
            Ok(()) => {
                self.status_message = Some("Uninstalled root and removed shortcuts.".to_string());
                self.refresh_status();
            }
            Err(err) => {
                self.error_message = Some(err.to_string());
            }
        }
    }

    fn launch_installed_app(&mut self) {
        if let InstallStatus::Installed { binary_path, .. } = &self.install_status {
            if let Err(err) = Command::new(binary_path).spawn() {
                self.error_message = Some(format!("Failed to launch root: {err}"));
            } else {
                self.status_message = Some("Launched root.".to_string());
            }
        }
    }
}

impl eframe::App for InstallerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        apply_app_theme(ctx, self.is_dark_mode, self.zoom_factor);

        egui::TopBottomPanel::top("installer_header").show(ctx, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.heading("🌿 root Setup Wizard");
                ui.separator();
                ui.label(format!(
                    "v{} • {} Installer",
                    env!("CARGO_PKG_VERSION"),
                    self.target_os.label()
                ));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let theme_btn = if self.is_dark_mode {
                        "☀ Light"
                    } else {
                        "🌙 Dark"
                    };
                    if ui.button(theme_btn).clicked() {
                        self.is_dark_mode = !self.is_dark_mode;
                    }
                });
            });
            ui.add_space(6.0);
        });

        egui::TopBottomPanel::bottom("installer_footer").show(ctx, |ui| {
            ui.add_space(6.0);
            if let Some(err) = &self.error_message {
                ui.colored_label(egui::Color32::LIGHT_RED, format!("⚠ {err}"));
            } else if let Some(msg) = &self.status_message {
                ui.colored_label(egui::Color32::LIGHT_GREEN, format!("✔ {msg}"));
            } else {
                ui.weak("Ready to configure your installation.");
            }
            ui.add_space(6.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(6.0);
            self.render_status_banner(ui);
            ui.add_space(10.0);
            self.render_location_section(ui);
            ui.add_space(10.0);
            self.render_options_section(ui);
            ui.add_space(10.0);
            self.render_paths_preview(ui);
            ui.add_space(16.0);
            self.render_action_buttons(ui);
        });
    }
}

impl InstallerApp {
    fn render_status_banner(&self, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.strong("Current Status:");
                match &self.install_status {
                    InstallStatus::NotInstalled => {
                        ui.colored_label(
                            egui::Color32::LIGHT_YELLOW,
                            "Not Installed in selected directory",
                        );
                    }
                    InstallStatus::Installed {
                        version,
                        binary_path,
                    } => {
                        ui.colored_label(
                            egui::Color32::LIGHT_GREEN,
                            format!("✔ Installed (v{version})"),
                        );
                        ui.weak(format!("({})", binary_path.display()));
                    }
                }
            });
        });
    }

    fn render_location_section(&mut self, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.strong("1. Installation Directory");
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let mut dir_text = self.options.install_dir.to_string_lossy().into_owned();
                if ui
                    .add(egui::TextEdit::singleline(&mut dir_text).desired_width(340.0))
                    .changed()
                {
                    self.options.install_dir = PathBuf::from(dir_text);
                    self.refresh_status();
                }

                if ui.button("📂 Browse...").clicked() {
                    if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                        self.options.install_dir = folder;
                        self.refresh_status();
                    }
                }

                if ui.button("↺ Default").clicked() {
                    self.options.install_dir =
                        default_install_dir_for_os(self.target_os, &self.home_dir);
                    self.refresh_status();
                }
            });
        });
    }

    fn render_options_section(&mut self, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.strong("2. OS Integration & Shortcuts");
            ui.add_space(4.0);

            let menu_label = match self.target_os {
                TargetOs::Windows => "Create Windows Start Menu shortcut & Installed Apps entry",
                TargetOs::MacOs => "Create macOS Application Bundle (root.app)",
                TargetOs::Linux => "Create Linux Desktop Application Launcher (root.desktop)",
            };
            ui.checkbox(&mut self.options.create_menu_shortcut, menu_label);
            ui.checkbox(
                &mut self.options.create_desktop_shortcut,
                "Create Desktop shortcut",
            );

            if self.target_os != TargetOs::Windows {
                ui.checkbox(
                    &mut self.options.install_cli_symlink,
                    "Install CLI binary in ~/.local/bin/root",
                );
            }
        });
    }

    fn render_paths_preview(&self, ui: &mut egui::Ui) {
        let preview = resolve_install_paths(self.target_os, &self.home_dir, &self.options);
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.strong("Target Paths Preview");
            ui.small(format!(
                "• App Binary: {}",
                preview.installed_binary.display()
            ));
            if let Some(menu) = &preview.menu_entry_path {
                ui.small(format!("• System Launcher: {}", menu.display()));
            }
            if let Some(desktop) = &preview.desktop_entry_path {
                ui.small(format!("• Desktop Shortcut: {}", desktop.display()));
            }
            if let Some(cli) = &preview.cli_link_path {
                ui.small(format!("• CLI Command: {}", cli.display()));
            }
        });
    }

    fn render_action_buttons(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            let install_label = match self.install_status {
                InstallStatus::NotInstalled => "🚀 Install root",
                InstallStatus::Installed { .. } => "🔄 Reinstall / Update root",
            };

            if ui.button(install_label).clicked() {
                self.run_install();
            }

            if matches!(self.install_status, InstallStatus::Installed { .. }) {
                if ui.button("▶ Launch root Now").clicked() {
                    self.launch_installed_app();
                }

                if ui.button("🗑 Uninstall").clicked() {
                    self.run_uninstall();
                }
            }
        });
    }
}
