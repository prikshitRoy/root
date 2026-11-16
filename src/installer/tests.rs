//! Unit and `tempfile` integration tests for cross-platform installation and uninstallation.

use std::fs;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

use crate::installer::{
    build_linux_desktop_entry, build_macos_info_plist, build_windows_shortcut_ps_script,
    default_install_dir_for_os, detect_install_status, install_application,
    is_invoked_as_installed_app, parse_cli_args, uninstall_application, CliCommand, InstallOptions,
    InstallStatus, TargetOs,
};

/// Creates a fake application binary inside `dir` for testing installation flows.
fn create_fake_source_binary(dir: &Path) -> PathBuf {
    let bin_path = dir.join("fake-root-bin");
    fs::write(&bin_path, b"#!/bin/sh\necho root\n").expect("failed to write fake binary");
    bin_path
}

#[test]
fn detects_whether_executable_is_named_root_or_root_installer() {
    assert!(is_invoked_as_installed_app(Some(Path::new(
        "/usr/bin/root"
    ))));
    assert!(is_invoked_as_installed_app(Some(Path::new(
        r"C:\Users\Dev\AppData\Local\Programs\root\root.exe"
    ))));
    assert!(!is_invoked_as_installed_app(Some(Path::new(
        "/usr/bin/root-installer"
    ))));
    assert!(!is_invoked_as_installed_app(Some(Path::new(
        r"C:\target\release\root-installer.exe"
    ))));
}

#[test]
fn parses_cli_install_and_uninstall_flags_with_custom_directory() {
    let install_args = vec![
        "--install".to_string(),
        "--dir".to_string(),
        "/opt/root".to_string(),
    ];
    assert_eq!(
        parse_cli_args(&install_args),
        CliCommand::Install {
            install_dir: Some(PathBuf::from("/opt/root"))
        }
    );

    let uninstall_args = vec!["--uninstall".to_string()];
    assert_eq!(
        parse_cli_args(&uninstall_args),
        CliCommand::Uninstall { install_dir: None }
    );

    let gui_args = vec!["--installer".to_string()];
    assert_eq!(parse_cli_args(&gui_args), CliCommand::OpenInstallerGui);
}

#[test]
fn installs_and_uninstalls_linux_layout_cleanly_in_temp_dir() {
    let temp = TempDir::new().unwrap();
    let home_dir = temp.path().join("home");
    let source_bin = create_fake_source_binary(temp.path());
    let install_dir = default_install_dir_for_os(TargetOs::Linux, &home_dir);

    let options = InstallOptions {
        install_dir: install_dir.clone(),
        create_menu_shortcut: true,
        create_desktop_shortcut: true,
        install_cli_symlink: true,
    };

    // Act: Install
    let receipt = install_application(TargetOs::Linux, &home_dir, &source_bin, &options).unwrap();

    // Assert: Binary, CLI link, .desktop entry, and SVG icon exist
    assert!(receipt.installed_binary.is_file());
    assert!(home_dir.join(".local/bin/root").is_file());
    assert!(home_dir
        .join(".local/share/applications/root.desktop")
        .is_file());
    assert!(home_dir
        .join(".local/share/icons/hicolor/scalable/apps/root.svg")
        .is_file());
    assert_eq!(
        detect_install_status(TargetOs::Linux, &home_dir, &install_dir),
        InstallStatus::Installed {
            version: receipt.version,
            binary_path: receipt.installed_binary,
        }
    );

    // Act: Uninstall
    uninstall_application(TargetOs::Linux, &home_dir, &install_dir).unwrap();

    // Assert: Everything is removed
    assert_eq!(
        detect_install_status(TargetOs::Linux, &home_dir, &install_dir),
        InstallStatus::NotInstalled
    );
    assert!(!home_dir
        .join(".local/share/applications/root.desktop")
        .exists());
}

#[test]
fn installs_and_uninstalls_macos_app_bundle_cleanly_in_temp_dir() {
    let temp = TempDir::new().unwrap();
    let home_dir = temp.path().join("Users/dev");
    let source_bin = create_fake_source_binary(temp.path());
    let install_dir = default_install_dir_for_os(TargetOs::MacOs, &home_dir);

    let options = InstallOptions {
        install_dir: install_dir.clone(),
        create_menu_shortcut: true,
        create_desktop_shortcut: true,
        install_cli_symlink: true,
    };

    // Act: Install macOS `.app` bundle
    let receipt = install_application(TargetOs::MacOs, &home_dir, &source_bin, &options).unwrap();

    let app_bundle = install_dir.join("root.app");
    assert!(app_bundle.join("Contents/Info.plist").is_file());
    assert!(app_bundle.join("Contents/MacOS/root").is_file());
    assert!(receipt.installed_binary.is_file());

    // Act: Uninstall
    uninstall_application(TargetOs::MacOs, &home_dir, &install_dir).unwrap();
    assert!(!app_bundle.exists());
}

#[test]
fn installs_windows_layout_with_generated_ico_file_in_temp_dir() {
    // Arrange
    let temp = TempDir::new().unwrap();
    let home_dir = temp.path().join("Users/Dev");
    let source_bin = create_fake_source_binary(temp.path());
    let install_dir = temp.path().join("Programs/root");

    let options = InstallOptions {
        install_dir: install_dir.clone(),
        create_menu_shortcut: false,
        create_desktop_shortcut: false,
        install_cli_symlink: false,
    };

    // Act
    let receipt = install_application(TargetOs::Windows, &home_dir, &source_bin, &options).unwrap();
    let ico_path = install_dir.join("root.ico");
    let ico_bytes = fs::read(&ico_path).unwrap();

    // Assert: root.exe and root.ico exist, and root.ico begins with ICONDIR header [0, 0, 1, 0, 1, 0]
    assert!(receipt.installed_binary.is_file());
    assert!(ico_path.is_file());
    assert_eq!(&ico_bytes[0..6], &[0, 0, 1, 0, 1, 0]);
}

#[test]
fn formats_linux_desktop_macos_plist_and_windows_ps_script_correctly() {
    // Arrange & Act
    let desktop = build_linux_desktop_entry(
        Path::new("/home/user/.local/bin/root"),
        Some(Path::new("/home/user/root.svg")),
    );
    let plist = build_macos_info_plist("0.1.0");
    let ps = build_windows_shortcut_ps_script(
        Path::new(r"C:\Users\O'Connor\Desktop\root.lnk"),
        Path::new(r"C:\Program Files\root\root.exe"),
        Path::new(r"C:\Program Files\root"),
        Some(Path::new(r"C:\Program Files\root\root.ico")),
        "root app",
    );

    // Assert
    assert!(desktop.contains("Exec=\"/home/user/.local/bin/root\""));
    assert!(desktop.contains("Icon=/home/user/root.svg"));
    assert!(plist.contains("<string>com.root.desktop</string>"));
    assert!(plist.contains("<string>0.1.0</string>"));
    assert!(ps.contains("O''Connor"));
    assert!(ps.contains(r"root.ico,0"));
}
