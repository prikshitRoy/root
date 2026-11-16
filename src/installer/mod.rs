//! Cross-platform installer and uninstaller engine, OS shortcut generators, and Setup UI.

mod engine;
mod paths;
mod shortcuts;
mod types;
pub mod ui;

#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};

pub use engine::{
    detect_install_status, install_application, resolve_source_app_binary, uninstall_application,
};
pub use paths::{default_install_dir_for_os, resolve_install_paths, resolve_os_home_dir};
pub use shortcuts::{
    build_linux_desktop_entry, build_macos_info_plist, build_windows_shortcut_ps_script,
};
pub use types::{
    CliCommand, InstallOptions, InstallPaths, InstallReceipt, InstallStatus, InstallerError,
    TargetOs,
};

/// Base binary stem for the main application executable (`root` or `root.exe`).
pub const APP_BINARY_STEM: &str = "root";

/// Base binary stem for the installer executable (`root-installer` or `root-installer.exe`).
pub const INSTALLER_BINARY_STEM: &str = "root-installer";

/// Parses command-line arguments into a `CliCommand`.
pub fn parse_cli_args(args: &[String]) -> CliCommand {
    let mut index = 0;
    let mut custom_dir: Option<PathBuf> = None;
    let mut should_install = false;
    let mut should_uninstall = false;
    let mut should_open_gui = false;

    while index < args.len() {
        match args[index].as_str() {
            "--installer" => should_open_gui = true,
            "--install" | "-i" => should_install = true,
            "--uninstall" | "-u" => should_uninstall = true,
            "--dir" | "-d" => {
                if let Some(dir_val) = args.get(index + 1) {
                    custom_dir = Some(PathBuf::from(dir_val));
                    index += 1;
                }
            }
            _ => {}
        }
        index += 1;
    }

    if should_uninstall {
        CliCommand::Uninstall {
            install_dir: custom_dir,
        }
    } else if should_install {
        CliCommand::Install {
            install_dir: custom_dir,
        }
    } else if should_open_gui {
        CliCommand::OpenInstallerGui
    } else {
        CliCommand::RunDefault
    }
}

/// Returns `true` when the current binary is named `root` (`root.exe`) rather than `root-installer`.
pub fn is_invoked_as_installed_app(exe_path: Option<&Path>) -> bool {
    let Some(path_str) = exe_path.and_then(|p| p.to_str()) else {
        return false;
    };
    // Split on both '/' and '\' so Windows paths are parsed identically on Unix test runners
    let file_name = path_str.rsplit(['/', '\\']).next().unwrap_or(path_str);
    let stem = Path::new(file_name).file_stem().and_then(|s| s.to_str());

    stem.is_some_and(|s| s.eq_ignore_ascii_case(APP_BINARY_STEM))
}

/// Executes a headless CLI install or uninstall command on the current OS.
pub fn execute_cli_command(command: CliCommand) -> Result<(), InstallerError> {
    let target_os = TargetOs::current();
    let home_dir = resolve_os_home_dir();

    match command {
        CliCommand::Install { install_dir } => {
            let options = InstallOptions {
                install_dir: install_dir
                    .unwrap_or_else(|| default_install_dir_for_os(target_os, &home_dir)),
                ..InstallOptions::default()
            };
            let current_exe =
                std::env::current_exe().map_err(|e| InstallerError::IoError(e.to_string()))?;
            let source_bin = resolve_source_app_binary(&current_exe);
            let receipt = install_application(target_os, &home_dir, &source_bin, &options)?;
            println!(
                "Successfully installed root v{} to '{}'",
                receipt.version,
                receipt.installed_binary.display()
            );
            Ok(())
        }
        CliCommand::Uninstall { install_dir } => {
            let dir =
                install_dir.unwrap_or_else(|| default_install_dir_for_os(target_os, &home_dir));
            uninstall_application(target_os, &home_dir, &dir)?;
            println!("Successfully uninstalled root from '{}'", dir.display());
            Ok(())
        }
        CliCommand::RunDefault | CliCommand::OpenInstallerGui => Ok(()),
    }
}
