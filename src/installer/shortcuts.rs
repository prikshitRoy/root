//! Pure generators and OS helpers for `.desktop` entries, macOS `Info.plist`, icons, and Windows `.lnk` shortcuts.

use std::path::Path;

pub use crate::branding::APP_ICON_SVG;

/// Builds a FreeDesktop-compliant `.desktop` file for Linux application launchers.
pub fn build_linux_desktop_entry(binary_path: &Path, icon_path: Option<&Path>) -> String {
    let exec_str = binary_path.to_string_lossy();
    let icon_str = icon_path
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| "utilities-terminal".to_string());

    format!(
        "[Desktop Entry]\n\
         Version=1.0\n\
         Type=Application\n\
         Name=root\n\
         GenericName=Project Tree & Markdown Bundler\n\
         Comment=Inspect software project trees and export source code to Markdown\n\
         Exec=\"{exec_str}\"\n\
         Icon={icon_str}\n\
         Terminal=false\n\
         Categories=Development;Utility;\n\
         StartupWMClass=root\n"
    )
}

/// Builds a macOS `Contents/Info.plist` XML manifest for `root.app`.
pub fn build_macos_info_plist(version: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>root</string>
    <key>CFBundleDisplayName</key>
    <string>root</string>
    <key>CFBundleIdentifier</key>
    <string>com.root.desktop</string>
    <key>CFBundleVersion</key>
    <string>{version}</string>
    <key>CFBundleShortVersionString</key>
    <string>{version}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleExecutable</key>
    <string>root</string>
    <key>CFBundleIconFile</key>
    <string>root.svg</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
"#
    )
}

/// Builds a macOS `.command` launcher script for optional Desktop shortcut creation.
pub fn build_macos_desktop_command(binary_path: &Path) -> String {
    format!("#!/bin/sh\n\"{}\" &\n", binary_path.to_string_lossy())
}

/// Builds a PowerShell script that creates a native Windows `.lnk` shortcut with an `IconLocation`.
pub fn build_windows_shortcut_ps_script(
    shortcut_lnk_path: &Path,
    target_exe_path: &Path,
    working_dir: &Path,
    icon_path: Option<&Path>,
    description: &str,
) -> String {
    let lnk = escape_ps_single_quoted(&shortcut_lnk_path.to_string_lossy());
    let target = escape_ps_single_quoted(&target_exe_path.to_string_lossy());
    let work = escape_ps_single_quoted(&working_dir.to_string_lossy());
    let icon_source = icon_path.unwrap_or(target_exe_path);
    let icon = escape_ps_single_quoted(&icon_source.to_string_lossy());
    let desc = escape_ps_single_quoted(description);

    format!(
        "$ws = New-Object -ComObject WScript.Shell; \
         $s = $ws.CreateShortcut('{lnk}'); \
         $s.TargetPath = '{target}'; \
         $s.WorkingDirectory = '{work}'; \
         $s.IconLocation = '{icon},0'; \
         $s.Description = '{desc}'; \
         $s.Save()"
    )
}

/// Escapes single quotes inside a PowerShell single-quoted string literal (`'` -> `''`).
fn escape_ps_single_quoted(input: &str) -> String {
    input.replace('\'', "''")
}

/// Creates a Windows `.lnk` shortcut on Windows hosts, returning `true` if creation succeeded.
pub fn create_windows_lnk_shortcut(
    shortcut_lnk_path: &Path,
    target_exe_path: &Path,
    working_dir: &Path,
    icon_path: Option<&Path>,
    description: &str,
) -> bool {
    let script = build_windows_shortcut_ps_script(
        shortcut_lnk_path,
        target_exe_path,
        working_dir,
        icon_path,
        description,
    );

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;

        if let Some(parent) = shortcut_lnk_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        // Remove existing .lnk first so Windows Explorer does not keep stale shortcut metadata
        let _ = std::fs::remove_file(shortcut_lnk_path);

        let status = std::process::Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .creation_flags(CREATE_NO_WINDOW)
            .status();

        // Notify Windows Shell to refresh cached shortcut icons immediately
        let _ = std::process::Command::new("ie4uinit.exe")
            .arg("-show")
            .creation_flags(CREATE_NO_WINDOW)
            .status();

        return status.is_ok_and(|s| s.success()) && shortcut_lnk_path.exists();
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = script;
        false
    }
}

/// Builds the `UninstallString` Windows runs from Settings → Installed apps.
/// Passes `--dir` so a custom install location is uninstalled instead of the default one.
pub fn build_windows_uninstall_command(installed_uninstaller: &Path, install_dir: &Path) -> String {
    format!(
        "\"{}\" --uninstall --dir \"{}\"",
        installed_uninstaller.display(),
        install_dir.display()
    )
}

/// Builds the `DisplayIcon` value (`<path>,0`), the documented `path,index` form Windows Settings reads.
pub fn build_windows_display_icon(icon_source: &Path) -> String {
    format!("{},0", icon_source.display())
}

/// Registers or removes the per-user Windows Uninstall Registry entry (`HKCU\...\Uninstall\root`).
pub fn update_windows_uninstall_registry(
    install_dir: &Path,
    installed_binary: &Path,
    installed_uninstaller: &Path,
    icon_path: Option<&Path>,
    version: &str,
    should_register: bool,
) -> bool {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let reg_key = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\root";

        if !should_register {
            return std::process::Command::new("reg")
                .args(["delete", reg_key, "/f"])
                .creation_flags(CREATE_NO_WINDOW)
                .status()
                .is_ok_and(|s| s.success());
        }

        let display_icon = build_windows_display_icon(icon_path.unwrap_or(installed_binary));
        let uninstall_cmd = build_windows_uninstall_command(installed_uninstaller, install_dir);
        let entries: &[(&str, &str, &str)] = &[
            ("DisplayName", "REG_SZ", "root"),
            ("DisplayVersion", "REG_SZ", version),
            ("Publisher", "REG_SZ", "root"),
            ("InstallLocation", "REG_SZ", &install_dir.to_string_lossy()),
            ("DisplayIcon", "REG_SZ", &display_icon),
            ("UninstallString", "REG_SZ", &uninstall_cmd),
            ("NoModify", "REG_DWORD", "1"),
            ("NoRepair", "REG_DWORD", "1"),
        ];

        for &(name, reg_type, value) in entries {
            let ok = std::process::Command::new("reg")
                .args([
                    "add", reg_key, "/v", name, "/t", reg_type, "/d", value, "/f",
                ])
                .creation_flags(CREATE_NO_WINDOW)
                .status()
                .is_ok_and(|s| s.success());
            if !ok {
                return false;
            }
        }
        true
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = (
            install_dir,
            installed_binary,
            installed_uninstaller,
            icon_path,
            version,
            should_register,
        );
        false
    }
}
