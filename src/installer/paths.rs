//! Pure cross-platform path calculation for Windows, macOS, and Linux installations.

use std::env;
use std::path::{Path, PathBuf};

use crate::installer::types::{InstallOptions, InstallPaths, TargetOs};

/// Filename used to record installed artifacts for clean uninstallation.
const RECEIPT_FILE_NAME: &str = "install-receipt.toml";

/// Resolves the current user's home directory across Windows, macOS, and Linux.
pub fn resolve_os_home_dir() -> PathBuf {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Computes the default per-user installation directory for `target_os`.
pub fn default_install_dir_for_os(target_os: TargetOs, home_dir: &Path) -> PathBuf {
    match target_os {
        TargetOs::Windows => {
            let local_app_data = env::var_os("LOCALAPPDATA")
                .map(PathBuf::from)
                .unwrap_or_else(|| home_dir.join("AppData").join("Local"));
            local_app_data.join("Programs").join("root")
        }
        TargetOs::MacOs => home_dir.join("Applications"),
        TargetOs::Linux => home_dir.join(".local").join("share").join("root"),
    }
}

/// Resolves all target binary, shortcut, icon, and receipt paths for a given OS and options.
pub fn resolve_install_paths(
    target_os: TargetOs,
    home_dir: &Path,
    options: &InstallOptions,
) -> InstallPaths {
    match target_os {
        TargetOs::Windows => resolve_windows_paths(home_dir, options),
        TargetOs::MacOs => resolve_macos_paths(home_dir, options),
        TargetOs::Linux => resolve_linux_paths(home_dir, options),
    }
}

fn resolve_windows_paths(home_dir: &Path, options: &InstallOptions) -> InstallPaths {
    let install_dir = options.install_dir.clone();
    let installed_binary = install_dir.join(TargetOs::Windows.app_binary_name());
    let installed_uninstaller = install_dir.join(TargetOs::Windows.installer_binary_name());
    let receipt_file = install_dir.join(RECEIPT_FILE_NAME);

    let app_data = env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir.join("AppData").join("Roaming"));

    let menu_entry_path = options.create_menu_shortcut.then(|| {
        app_data
            .join("Microsoft")
            .join("Windows")
            .join("Start Menu")
            .join("Programs")
            .join("root.lnk")
    });

    let desktop_entry_path = options
        .create_desktop_shortcut
        .then(|| home_dir.join("Desktop").join("root.lnk"));

    InstallPaths {
        install_dir,
        installed_binary,
        installed_uninstaller,
        receipt_file,
        menu_entry_path,
        desktop_entry_path,
        cli_link_path: None,
        icon_path: None,
    }
}

fn resolve_macos_paths(home_dir: &Path, options: &InstallOptions) -> InstallPaths {
    let app_bundle_dir = if options.install_dir.extension().is_some_and(|e| e == "app") {
        options.install_dir.clone()
    } else {
        options.install_dir.join("root.app")
    };

    let contents_dir = app_bundle_dir.join("Contents");
    let macos_bin_dir = contents_dir.join("MacOS");
    let resources_dir = contents_dir.join("Resources");

    let installed_binary = macos_bin_dir.join(TargetOs::MacOs.app_binary_name());
    let installed_uninstaller = macos_bin_dir.join(TargetOs::MacOs.installer_binary_name());
    let receipt_file = resources_dir.join(RECEIPT_FILE_NAME);
    let icon_path = Some(resources_dir.join("root.svg"));

    let menu_entry_path = Some(contents_dir.join("Info.plist"));
    let desktop_entry_path = options
        .create_desktop_shortcut
        .then(|| home_dir.join("Desktop").join("root.command"));
    let cli_link_path = options
        .install_cli_symlink
        .then(|| home_dir.join(".local").join("bin").join("root"));

    InstallPaths {
        install_dir: options.install_dir.clone(),
        installed_binary,
        installed_uninstaller,
        receipt_file,
        menu_entry_path,
        desktop_entry_path,
        cli_link_path,
        icon_path,
    }
}

fn resolve_linux_paths(home_dir: &Path, options: &InstallOptions) -> InstallPaths {
    let install_dir = options.install_dir.clone();
    let installed_binary = install_dir.join(TargetOs::Linux.app_binary_name());
    let installed_uninstaller = install_dir.join(TargetOs::Linux.installer_binary_name());
    let receipt_file = install_dir.join(RECEIPT_FILE_NAME);

    let data_home = home_dir.join(".local").join("share");
    let icon_path = Some(
        data_home
            .join("icons")
            .join("hicolor")
            .join("scalable")
            .join("apps")
            .join("root.svg"),
    );

    let menu_entry_path = options
        .create_menu_shortcut
        .then(|| data_home.join("applications").join("root.desktop"));

    let desktop_entry_path = options
        .create_desktop_shortcut
        .then(|| home_dir.join("Desktop").join("root.desktop"));

    let cli_link_path = options
        .install_cli_symlink
        .then(|| home_dir.join(".local").join("bin").join("root"));

    InstallPaths {
        install_dir,
        installed_binary,
        installed_uninstaller,
        receipt_file,
        menu_entry_path,
        desktop_entry_path,
        cli_link_path,
        icon_path,
    }
}
