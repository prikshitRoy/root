//! Data structures and error types for cross-platform application installation.

use std::error::Error;
use std::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Supported operating system families for native installation layouts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetOs {
    /// Microsoft Windows (`%LOCALAPPDATA%\Programs\root`, `.lnk` shortcuts, Registry entry).
    Windows,
    /// Apple macOS (`~/Applications/root.app` bundle, `Info.plist`, CLI symlink).
    MacOs,
    /// Linux / FreeDesktop (`~/.local/bin/root`, `.desktop` file, XDG scalable SVG icon).
    Linux,
}

impl TargetOs {
    /// Detects the active host operating system at compile time.
    pub const fn current() -> Self {
        if cfg!(target_os = "windows") {
            Self::Windows
        } else if cfg!(target_os = "macos") {
            Self::MacOs
        } else {
            Self::Linux
        }
    }

    /// Human-readable operating system label for the Setup Wizard UI.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Windows => "Windows",
            Self::MacOs => "macOS",
            Self::Linux => "Linux",
        }
    }

    /// Standard executable filename for `root` on this OS.
    pub const fn app_binary_name(self) -> &'static str {
        match self {
            Self::Windows => "root.exe",
            Self::MacOs | Self::Linux => "root",
        }
    }

    /// Standard executable filename for `root-installer` on this OS.
    pub const fn installer_binary_name(self) -> &'static str {
        match self {
            Self::Windows => "root-installer.exe",
            Self::MacOs | Self::Linux => "root-installer",
        }
    }
}

/// User-configurable options in the Setup Wizard or CLI installer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallOptions {
    /// Base directory where `root` (or `root.app`) will be installed.
    pub install_dir: PathBuf,
    /// Whether to create a Start Menu (Windows), Applications (macOS), or `.desktop` launcher (Linux).
    pub create_menu_shortcut: bool,
    /// Whether to place a shortcut on the user's Desktop.
    pub create_desktop_shortcut: bool,
    /// Whether to expose the `root` binary in `~/.local/bin` (Linux/macOS).
    pub install_cli_symlink: bool,
}

impl Default for InstallOptions {
    fn default() -> Self {
        let target_os = TargetOs::current();
        let home = crate::installer::paths::resolve_os_home_dir();
        Self {
            install_dir: crate::installer::paths::default_install_dir_for_os(target_os, &home),
            create_menu_shortcut: true,
            create_desktop_shortcut: true,
            install_cli_symlink: true,
        }
    }
}

/// Concrete target paths computed for an installation run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallPaths {
    /// Root folder for the installation (or parent of `root.app` on macOS).
    pub install_dir: PathBuf,
    /// Full path to the installed main `root` executable.
    pub installed_binary: PathBuf,
    /// Full path to the installed `root-installer` executable used for future uninstallation.
    pub installed_uninstaller: PathBuf,
    /// Path to `install-receipt.toml` storing the manifest of created files.
    pub receipt_file: PathBuf,
    /// Optional Start Menu `.lnk`, `.desktop` launcher, or `.app` bundle path.
    pub menu_entry_path: Option<PathBuf>,
    /// Optional Desktop shortcut path.
    pub desktop_entry_path: Option<PathBuf>,
    /// Optional CLI binary link in `~/.local/bin/root`.
    pub cli_link_path: Option<PathBuf>,
    /// Optional icon asset path (`root.svg`).
    pub icon_path: Option<PathBuf>,
}

/// Persisted TOML receipt written during installation so uninstallation removes all artifacts cleanly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstallReceipt {
    /// Installed application version (e.g., `"0.1.0"`).
    pub version: String,
    /// Target OS under which the installation was performed.
    pub target_os: TargetOs,
    /// Main installed executable path.
    pub installed_binary: PathBuf,
    /// List of all auxiliary files, icons, and shortcuts created by the installer.
    pub created_files: Vec<PathBuf>,
    /// List of bundle or app directories created by the installer.
    pub created_dirs: Vec<PathBuf>,
    /// Whether a Windows Uninstall Registry key was written.
    pub registered_windows_uninstall: bool,
}

/// Current installation state detected on the host system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallStatus {
    /// `root` is not currently installed at the target directory.
    NotInstalled,
    /// `root` is installed and has a valid receipt or binary at the target path.
    Installed {
        version: String,
        binary_path: PathBuf,
    },
}

/// Parsed command-line mode for `root` and `root-installer`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliCommand {
    /// Run default binary behavior (`RootApp` for `root`, `InstallerApp` for `root-installer`).
    RunDefault,
    /// Explicitly open the graphical Setup Wizard (`--installer`).
    OpenInstallerGui,
    /// Perform a headless installation (`--install [--dir <path>]`).
    Install { install_dir: Option<PathBuf> },
    /// Perform a headless uninstallation (`--uninstall [--dir <path>]`).
    Uninstall { install_dir: Option<PathBuf> },
}

/// Errors that can occur during installation or uninstallation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallerError {
    /// Source binary to install does not exist on disk.
    SourceBinaryNotFound(PathBuf),
    /// Filesystem or permission error while copying files or creating directories.
    IoError(String),
    /// Failed to serialize or parse `install-receipt.toml`.
    ReceiptError(String),
}

impl fmt::Display for InstallerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceBinaryNotFound(path) => {
                write!(
                    f,
                    "Source application binary not found at '{}'",
                    path.display()
                )
            }
            Self::IoError(reason) => write!(f, "Filesystem error: {reason}"),
            Self::ReceiptError(reason) => write!(f, "Install receipt error: {reason}"),
        }
    }
}

impl Error for InstallerError {}
