//! Regression tests for Windows "Installed apps" registration and icon assets.

use std::path::Path;

use super::os_refresh::build_windows_self_delete_args;
use super::{build_windows_display_icon, build_windows_uninstall_command};
use crate::branding::APP_ICON_SVG;

#[test]
fn uninstall_command_targets_the_custom_install_dir() {
    // Arrange: a non-default install location (Settings would otherwise uninstall the default one)
    let uninstaller = Path::new(r"D:\Tools\root\root-installer.exe");
    let install_dir = Path::new(r"D:\Tools\root");

    // Act
    let command = build_windows_uninstall_command(uninstaller, install_dir);

    // Assert
    assert_eq!(
        command,
        r#""D:\Tools\root\root-installer.exe" --uninstall --dir "D:\Tools\root""#
    );
}

#[test]
fn display_icon_uses_path_comma_index_form() {
    // Arrange & Act
    let value = build_windows_display_icon(Path::new(r"C:\Apps\root\root-leaf.ico"));

    // Assert
    assert_eq!(value, r"C:\Apps\root\root-leaf.ico,0");
}

#[test]
fn self_delete_args_remove_uninstaller_then_folder() {
    // Arrange & Act
    let args = build_windows_self_delete_args(
        Path::new(r"C:\Apps\root\root-installer.exe"),
        Path::new(r"C:\Apps\root"),
    );

    // Assert: file is deleted before the folder so rmdir succeeds on an empty directory
    assert_eq!(
        args,
        r#"/c ping -n 3 127.0.0.1 >nul & del /f /q "C:\Apps\root\root-installer.exe" & rmdir "C:\Apps\root""#
    );
}

#[test]
fn svg_icon_declares_explicit_size_for_launchers() {
    // Launchers that cannot infer a size from viewBox alone render a blank icon
    assert!(APP_ICON_SVG.contains(r#"width="128" height="128""#));
}
