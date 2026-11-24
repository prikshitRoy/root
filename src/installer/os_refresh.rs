//! Best-effort OS integration steps run after install/uninstall: Linux icon and
//! launcher cache refresh, and deferred self-deletion of the Windows uninstaller.

use std::path::Path;

use crate::installer::types::{InstallPaths, TargetOs};

/// Refreshes the FreeDesktop icon/launcher caches so the new `root` icon shows up in the app launcher.
///
/// Failures are ignored on purpose: the tools are optional and the `.desktop` file
/// already points at the SVG by absolute path, so launchers still work without a cache.
pub fn refresh_launcher_caches(target_os: TargetOs, paths: &InstallPaths) {
    if target_os != TargetOs::Linux {
        return;
    }

    if let Some(icon_dir) = paths
        .icon_path
        .as_deref()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .and_then(Path::parent)
    {
        run_quietly("gtk-update-icon-cache", &["-f", "-t"], Some(icon_dir));
    }

    if let Some(apps_dir) = paths.menu_entry_path.as_deref().and_then(Path::parent) {
        run_quietly("update-desktop-database", &[], Some(apps_dir));
    }

    // GNOME only runs launchers on the Desktop that are marked as trusted.
    if let Some(desktop_file) = paths.desktop_entry_path.as_deref() {
        if desktop_file.is_file() {
            let file_arg = desktop_file.to_string_lossy().into_owned();
            run_quietly(
                "gio",
                &["set", &file_arg, "metadata::trusted", "true"],
                None,
            );
        }
    }
}

/// Runs `program args [extra_path]`, discarding output and errors (missing tool is fine).
fn run_quietly(program: &str, args: &[&str], extra_path: Option<&Path>) {
    let mut command = std::process::Command::new(program);
    command.args(args);
    if let Some(path) = extra_path {
        command.arg(path);
    }
    let _ = command
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
}

/// Builds the `cmd.exe` argument string that waits for the uninstaller to exit, then deletes it.
pub fn build_windows_self_delete_args(uninstaller: &Path, install_dir: &Path) -> String {
    format!(
        "/c ping -n 3 127.0.0.1 >nul & del /f /q \"{}\" & rmdir \"{}\"",
        uninstaller.display(),
        install_dir.display()
    )
}

/// On Windows, deletes the running uninstaller (and the then-empty install folder) after it exits.
///
/// A running `.exe` cannot delete itself, so without this `root-installer.exe` and the
/// install folder are left behind. Does nothing unless the running exe is the installed copy.
pub fn schedule_windows_self_delete(paths: &InstallPaths) {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;

        let running_installed_copy =
            std::env::current_exe().is_ok_and(|exe| exe == paths.installed_uninstaller);
        if !running_installed_copy {
            return;
        }

        let args = build_windows_self_delete_args(&paths.installed_uninstaller, &paths.install_dir);
        let _ = std::process::Command::new("cmd")
            .raw_arg(args)
            .creation_flags(CREATE_NO_WINDOW)
            .spawn();
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = paths;
    }
}
