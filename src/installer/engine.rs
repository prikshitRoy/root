//! Core installation, binary resolution, receipt persistence, and uninstallation engine.

use std::fs;
use std::path::{Path, PathBuf};

use crate::installer::paths::resolve_install_paths;
use crate::installer::shortcuts::{
    build_linux_desktop_entry, build_macos_desktop_command, build_macos_info_plist,
    create_windows_lnk_shortcut, update_windows_uninstall_registry, APP_ICON_SVG,
};
use crate::installer::types::{
    InstallOptions, InstallReceipt, InstallStatus, InstallerError, TargetOs,
};

/// Current package version embedded from `Cargo.toml`.
const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Resolves the `root` binary to install: prefers a sibling `root` (`root.exe`) next to `installer_exe`,
/// falling back to `installer_exe` itself for single-file standalone installer distribution.
pub fn resolve_source_app_binary(installer_exe: &Path) -> PathBuf {
    if let Some(parent_dir) = installer_exe.parent() {
        let sibling_app = parent_dir.join(TargetOs::current().app_binary_name());
        if sibling_app.is_file() {
            return sibling_app;
        }
    }
    installer_exe.to_path_buf()
}

/// Checks whether `root` is currently installed at `install_dir` for `target_os`.
pub fn detect_install_status(
    target_os: TargetOs,
    home_dir: &Path,
    install_dir: &Path,
) -> InstallStatus {
    let options = InstallOptions {
        install_dir: install_dir.to_path_buf(),
        ..InstallOptions::default()
    };
    let paths = resolve_install_paths(target_os, home_dir, &options);

    if let Ok(raw_receipt) = fs::read_to_string(&paths.receipt_file) {
        if let Ok(receipt) = toml::from_str::<InstallReceipt>(&raw_receipt) {
            if receipt.installed_binary.is_file() {
                return InstallStatus::Installed {
                    version: receipt.version,
                    binary_path: receipt.installed_binary,
                };
            }
        }
    }

    if paths.installed_binary.is_file() {
        return InstallStatus::Installed {
            version: PACKAGE_VERSION.to_string(),
            binary_path: paths.installed_binary,
        };
    }

    InstallStatus::NotInstalled
}

/// Installs `root` and its OS shortcuts/bundles according to `options`, returning an `InstallReceipt`.
pub fn install_application(
    target_os: TargetOs,
    home_dir: &Path,
    source_binary: &Path,
    options: &InstallOptions,
) -> Result<InstallReceipt, InstallerError> {
    if !source_binary.is_file() {
        return Err(InstallerError::SourceBinaryNotFound(
            source_binary.to_path_buf(),
        ));
    }

    let paths = resolve_install_paths(target_os, home_dir, options);
    let mut created_files = Vec::new();
    let mut created_dirs = Vec::new();

    if let Some(bin_parent) = paths.installed_binary.parent() {
        create_dir_checked(bin_parent)?;
        created_dirs.push(bin_parent.to_path_buf());
    }

    copy_binary_with_permissions(source_binary, &paths.installed_binary)?;
    created_files.push(paths.installed_binary.clone());

    // Copy uninstaller binary alongside main binary so user can run GUI/CLI uninstaller anytime
    let uninstaller_src = std::env::current_exe().unwrap_or_else(|_| source_binary.to_path_buf());
    if uninstaller_src.is_file() {
        copy_binary_with_permissions(&uninstaller_src, &paths.installed_uninstaller)?;
        created_files.push(paths.installed_uninstaller.clone());
    }

    if let Some(icon_path) = &paths.icon_path {
        write_text_file(icon_path, APP_ICON_SVG)?;
        created_files.push(icon_path.clone());
    }

    match target_os {
        TargetOs::Windows => {
            install_windows_shortcuts(&paths, &mut created_files)?;
        }
        TargetOs::MacOs => {
            install_macos_bundle(&paths, &mut created_files, &mut created_dirs)?;
        }
        TargetOs::Linux => {
            install_linux_shortcuts(&paths, &mut created_files)?;
        }
    }

    let registered_windows_uninstall = if target_os == TargetOs::Windows {
        update_windows_uninstall_registry(
            &paths.install_dir,
            &paths.installed_binary,
            &paths.installed_uninstaller,
            PACKAGE_VERSION,
            true,
        )
    } else {
        false
    };

    let receipt = InstallReceipt {
        version: PACKAGE_VERSION.to_string(),
        target_os,
        installed_binary: paths.installed_binary.clone(),
        created_files,
        created_dirs,
        registered_windows_uninstall,
    };

    let receipt_toml = toml::to_string_pretty(&receipt)
        .map_err(|err| InstallerError::ReceiptError(err.to_string()))?;
    write_text_file(&paths.receipt_file, &receipt_toml)?;

    Ok(receipt)
}

/// Removes an existing `root` installation, its shortcuts, and its receipt from `install_dir`.
pub fn uninstall_application(
    target_os: TargetOs,
    home_dir: &Path,
    install_dir: &Path,
) -> Result<(), InstallerError> {
    let options = InstallOptions {
        install_dir: install_dir.to_path_buf(),
        create_menu_shortcut: true,
        create_desktop_shortcut: true,
        install_cli_symlink: true,
    };
    let paths = resolve_install_paths(target_os, home_dir, &options);

    if let Ok(raw_receipt) = fs::read_to_string(&paths.receipt_file) {
        if let Ok(receipt) = toml::from_str::<InstallReceipt>(&raw_receipt) {
            for file in &receipt.created_files {
                let _ = fs::remove_file(file);
            }
            let _ = fs::remove_file(&paths.receipt_file);

            if receipt.registered_windows_uninstall {
                update_windows_uninstall_registry(
                    &paths.install_dir,
                    &paths.installed_binary,
                    &paths.installed_uninstaller,
                    &receipt.version,
                    false,
                );
            }

            for dir in receipt.created_dirs.iter().rev() {
                let _ = fs::remove_dir_all(dir);
            }
            let _ = fs::remove_dir(&paths.install_dir);
            return Ok(());
        }
    }

    // Fallback cleanup if receipt was manually deleted
    for candidate in [
        Some(&paths.installed_binary),
        Some(&paths.installed_uninstaller),
        Some(&paths.receipt_file),
        paths.menu_entry_path.as_ref(),
        paths.desktop_entry_path.as_ref(),
        paths.cli_link_path.as_ref(),
        paths.icon_path.as_ref(),
    ]
    .into_iter()
    .flatten()
    {
        let _ = fs::remove_file(candidate);
    }

    if target_os == TargetOs::MacOs {
        let app_bundle = install_dir.join("root.app");
        let _ = fs::remove_dir_all(app_bundle);
    } else {
        let _ = fs::remove_dir(install_dir);
    }

    Ok(())
}

fn install_windows_shortcuts(
    paths: &crate::installer::types::InstallPaths,
    created_files: &mut Vec<PathBuf>,
) -> Result<(), InstallerError> {
    for shortcut_path in [&paths.menu_entry_path, &paths.desktop_entry_path]
        .into_iter()
        .flatten()
    {
        if create_windows_lnk_shortcut(
            shortcut_path,
            &paths.installed_binary,
            &paths.install_dir,
            "Inspect software project trees and export code to Markdown",
        ) {
            created_files.push(shortcut_path.clone());
        } else {
            // Fallback `.cmd` launcher when running in environments without PowerShell WScript.Shell
            let cmd_fallback = shortcut_path.with_extension("cmd");
            let cmd_content = format!(
                "@echo off\r\nstart \"\" \"{}\"\r\n",
                paths.installed_binary.display()
            );
            write_text_file(&cmd_fallback, &cmd_content)?;
            created_files.push(cmd_fallback);
        }
    }
    Ok(())
}

fn install_macos_bundle(
    paths: &crate::installer::types::InstallPaths,
    created_files: &mut Vec<PathBuf>,
    created_dirs: &mut Vec<PathBuf>,
) -> Result<(), InstallerError> {
    if let Some(plist_path) = &paths.menu_entry_path {
        let plist_xml = build_macos_info_plist(PACKAGE_VERSION);
        write_text_file(plist_path, &plist_xml)?;
        created_files.push(plist_path.clone());
        if let Some(bundle_dir) = plist_path.parent().and_then(|p| p.parent()) {
            created_dirs.push(bundle_dir.to_path_buf());
        }
    }

    if let Some(desktop_cmd_path) = &paths.desktop_entry_path {
        let script = build_macos_desktop_command(&paths.installed_binary);
        write_text_file(desktop_cmd_path, &script)?;
        set_executable_permissions(desktop_cmd_path)?;
        created_files.push(desktop_cmd_path.clone());
    }

    if let Some(cli_path) = &paths.cli_link_path {
        copy_binary_with_permissions(&paths.installed_binary, cli_path)?;
        created_files.push(cli_path.clone());
    }

    Ok(())
}

fn install_linux_shortcuts(
    paths: &crate::installer::types::InstallPaths,
    created_files: &mut Vec<PathBuf>,
) -> Result<(), InstallerError> {
    let exec_target = paths
        .cli_link_path
        .as_ref()
        .unwrap_or(&paths.installed_binary);

    if let Some(cli_path) = &paths.cli_link_path {
        copy_binary_with_permissions(&paths.installed_binary, cli_path)?;
        created_files.push(cli_path.clone());
    }

    let desktop_content = build_linux_desktop_entry(exec_target, paths.icon_path.as_deref());

    if let Some(menu_desktop) = &paths.menu_entry_path {
        write_text_file(menu_desktop, &desktop_content)?;
        set_executable_permissions(menu_desktop)?;
        created_files.push(menu_desktop.clone());
    }

    if let Some(desktop_shortcut) = &paths.desktop_entry_path {
        write_text_file(desktop_shortcut, &desktop_content)?;
        set_executable_permissions(desktop_shortcut)?;
        created_files.push(desktop_shortcut.clone());
    }

    Ok(())
}

fn create_dir_checked(dir: &Path) -> Result<(), InstallerError> {
    fs::create_dir_all(dir).map_err(|err| {
        InstallerError::IoError(format!(
            "Failed to create directory '{}': {err}",
            dir.display()
        ))
    })
}

fn write_text_file(path: &Path, content: &str) -> Result<(), InstallerError> {
    if let Some(parent) = path.parent() {
        create_dir_checked(parent)?;
    }
    fs::write(path, content).map_err(|err| {
        InstallerError::IoError(format!("Failed to write '{}': {err}", path.display()))
    })
}

fn copy_binary_with_permissions(src: &Path, dst: &Path) -> Result<(), InstallerError> {
    if let Some(parent) = dst.parent() {
        create_dir_checked(parent)?;
    }
    if src != dst {
        fs::copy(src, dst).map_err(|err| {
            InstallerError::IoError(format!(
                "Failed to copy '{}' to '{}': {err}",
                src.display(),
                dst.display()
            ))
        })?;
    }
    set_executable_permissions(dst)
}

fn set_executable_permissions(path: &Path) -> Result<(), InstallerError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o755);
        fs::set_permissions(path, perms).map_err(|err| {
            InstallerError::IoError(format!(
                "Failed to set executable permissions on '{}': {err}",
                path.display()
            ))
        })?;
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}
