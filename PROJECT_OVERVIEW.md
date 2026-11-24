# Project Overview

> **Instruction for AI:** This file is the single source of truth for what this project is and what has been built.
> (see "How to Keep This File Updated" at the bottom).

## 1. What Is This Project?

**Name:** `root`

A cross-platform **desktop application built in pure Rust** that scans software project directories, detects the project framework (Next.js, Vite, WXT, Tauri, Electron, Node.js, Rust/Cargo, etc.), displays an interactive file tree respecting `.gitignore` and `.ignore` rules, and exports the tree structure plus all or selected source files into a single `.md` file.

It compiles into both a standalone portable desktop executable (`root`) that runs directly and a built-in graphical/CLI Setup Wizard (`root-installer`) that installs the application, generates native `🌿 root` icons (`.ico` and `.svg`), creates system shortcuts, and manages clean uninstallation.

## 2. Tech Stack

| Area                   | Choice                                                                      |
| ---------------------- | --------------------------------------------------------------------------- |
| Language               | Rust (2021 Edition)                                                         |
| Desktop & Installer UI | `eframe` / `egui` (v0.29)                                                   |
| Directory traversal    | `ignore` (v0.4)                                                             |
| Native OS dialogs      | `rfd` (v0.15)                                                               |
| Manifest & config I/O  | `serde` (v1.0), `serde_json` (v1.0), `toml` (v0.8)                          |
| Packaging metadata     | `cargo-packager` & `cargo-bundle` metadata in `Cargo.toml`                  |
| Unit/integration tests | Cargo built-in test runner (`cargo test`) + `tempfile` (v3.13)              |
| Target platforms       | Windows (active release binary target: `root.exe` & `root-installer.exe`), Linux, macOS |

## 3. Core Concepts

- **FileNode:** A node in the scanned project hierarchy representing either a directory or a file, along with its relative path, size in bytes, and content classification (`TextCode`, `Lockfile`, or `Binary`).
- **ProjectKind & ProjectProfile:** The auto-detected software project framework (Tauri, Electron, WXT, Next.js, Vite, Rust/Cargo, Node.js, or Unknown) paired with detection evidence and user cross-check/override state.
- **SelectionState & ExportMode:** Tracks how source files are bundled into the Markdown document (`TreeOnly`, `AllCode`, or `SelectedCode`) with cascading folder and file checkboxes.
- **ScanJobHandle & ExportJobHandle:** Non-blocking background worker handles using `std::thread` and `std::sync::mpsc` to stream live directory scan counts and `.md` file export progress to the UI.
- **AppSettings:** Persistent user preferences (`is_dark_mode` and `zoom_factor`) stored as TOML in the OS configuration directory (`settings.toml`).
- **Branding & Icon Generators:** Shared `🌿 root` title constants, pure-Rust 64x64 RGBA sprout-and-root icon rasterizer (`egui::IconData` for native window titlebars/taskbars), binary Windows `.ico` encoder (`encode_rgba_to_ico`), and scalable `APP_ICON_SVG`.
- **InstallerApp, InstallOptions & InstallReceipt:** Cross-platform installation engine and `egui` Setup Wizard that copies binaries, writes `root.ico` / `root.svg` icons, generates OS shortcuts/bundles, registers the uninstaller, and writes an `install-receipt.toml` manifest for clean uninstallation.

## 4. Features

Status key: **Done** | **In progress** | **Planned**

### 4.1 Workspace and app shell

| Feature                                                                                                        | Status |
| -------------------------------------------------------------------------------------------------------------- | ------ |
| Cargo project setup with `eframe` native window shell                                                          | Done   |
| Shared library (`src/lib.rs`) with dual binary targets: portable app (`root`) and Setup Wizard (`root-installer`) | Done |
| Feature-grouped module architecture (`branding`, `scanner`, `detector`, `selection`, `settings`, `markdown`, `installer`, `ui`) | Done |

### 4.2 Directory scanning and ignore handling

| Feature                                                                                         | Status |
| ----------------------------------------------------------------------------------------------- | ------ |
| Recursive directory scanning respecting `.gitignore`, `.ignore`, and `.git/info/exclude`        | Done   |
| Fast extension-first file classification and binary/lockfile filtering                          | Done   |
| Non-blocking background directory scan worker streaming live entry counts                       | Done   |

### 4.3 Framework detection

| Feature                                                                                         | Status |
| ----------------------------------------------------------------------------------------------- | ------ |
| Automatic detection for Next.js, Vite, WXT, Tauri, Electron, Node.js, and Rust/Cargo projects   | Done   |
| Interactive UI banner to cross-check, confirm, or override the detected project type            | Done   |

### 4.4 Tree selection and Markdown export

| Feature                                                                                          | Status |
| ------------------------------------------------------------------------------------------------ | ------ |
| ASCII/Unicode tree structure generator                                                           | Done   |
| Export modes: Tree Only, Tree + All Code, and Tree + Selected Code                               | Done   |
| Fenced code block bundler with file path headers, syntax tags, and dynamic backtick escaping     | Done   |
| Native folder picker and `.md` file save dialog via `rfd`                                        | Done   |
| Background `.md` export worker with real-time animated progress bar and one-click clipboard copy | Done   |

### 4.5 User interface and branding

| Feature                                                                                          | Status |
| ------------------------------------------------------------------------------------------------ | ------ |
| Unified `🌿 root` branding constants across window titles, top toolbar, onboarding card, and Setup Wizard | Done |
| Pure-Rust 64x64 RGBA `🌿 root` native window and taskbar icon (`ViewportBuilder::with_icon`)     | Done   |
| Interactive collapsible tree view with cascading folder/file checkboxes                          | Done   |
| Dynamic `All Code` and `None` preset buttons with live checkmark (`☑`/`☐`) and active states     | Done   |
| Animated project loading view displaying live scanned file counts and current path               | Done   |
| Live Markdown preview pane                                                                       | Done   |
| Live file/folder search filter, summary stats bar, and Dark/Light theme switcher                 | Done   |
| UI Zoom In / Zoom Out / Reset controls (70% to 200%) via toolbar buttons and keyboard shortcuts  | Done   |
| Persistent TOML app settings (`settings.toml`) for saving and restoring theme and zoom state     | Done   |
| Top toolbar `📦 Install...` button to launch the Setup Wizard directly from the portable app     | Done   |

### 4.6 Application installer and OS integration

| Feature                                                                                          | Status |
| ------------------------------------------------------------------------------------------------ | ------ |
| Graphical `eframe`/`egui` Setup Wizard (`root-installer`) to install, update, launch, or uninstall `root` | Done |
| Headless CLI installer & uninstaller flags (`--install`, `--uninstall`, `--dir <path>`, `--installer`) | Done |
| Pure-Rust Windows `root.ico` generator bound to Start Menu/Desktop `.lnk` `IconLocation` and Registry `DisplayIcon` | Done |
| Windows installer workflow (`%LOCALAPPDATA%\Programs\root`, Start Menu & Desktop `.lnk` shortcuts, `HKCU` Uninstall Registry entry) | Done |
| Cross-platform installer support for macOS (`~/Applications/root.app` bundle) and Linux (`.desktop` & SVG icon) | Done |
| Windows Settings → Installed apps entry: DisplayIcon (oot-leaf.ico,0), UninstallString with --dir, entry removed even without a receipt, uninstaller self-deletes after exit | Done |
| Linux launcher icon: .desktop Icon= plus XDG SVG (ssets/root.svg), icon/desktop cache refresh, .deb ships icon, window pp_id = oot | Done (CI-built; not yet verified on a real Linux desktop) |
| Single-file fallback allowing `root-installer.exe` to install itself as `root.exe` when distributed standalone | Done |
| TOML installation receipt (`install-receipt.toml`) tracking all created files and folders for clean removal | Done |
| Automated GitHub Actions release pipeline (`.github/workflows/release.yml`) building Windows (`.exe`, `.zip`), macOS arm64 & x64 (`.dmg`, `.app.zip`, `.tar.gz`), and Linux x64 (`.deb`, Setup Wizard, `.tar.gz`) with `SHA256SUMS.txt` | Done |

### 4.7 Testing

| Feature                                                                                          | Status |
| ------------------------------------------------------------------------------------------------ | ------ |
| Unit tests for pure branding/ICO encoding, detector, selection, settings, installer CLI/shortcuts, UI helpers, and Markdown | Done |
| Integration tests using `tempfile` for ignore-aware scanning, `.md` export, settings, and Windows/Linux/macOS install flows | Done |

## 5. Known Issues and Open Items

> Verify these against the current code; remove an item when it is fixed.

- Linux launcher-icon fix (explicit SVG size, cache refresh, .deb icon, pp_id) is covered by unit tests only; confirm on a real Ubuntu desktop after installing the next release.
- None other open. All cross-platform release targets (`windows-x64`, `macos-arm64`, `macos-x64`, `linux-x64`) build, pass tests, and publish via GitHub Actions.

## 6. Project Structure

```text
.github/workflows/
  release.yml            # CI/CD matrix workflow building and publishing multi-OS installers to GitHub Releases
assets/
  root.svg                # Shared SVG logo used by the installer (APP_ICON_SVG) and the Linux .deb.
Cargo.toml               # Rust package manifest, dual [[bin]] targets, and bundler metadata
README.md                # User and developer documentation
RELEASING.md             # Release & distribution guide, checksum verification, and maintainer checklist
PROJECT_OVERVIEW.md      # Single source of truth for project status and architecture
src/
  lib.rs                 # Shared library root exposing app and installer runners with native window icons
  main.rs                # Entry point for portable `root` binary (with CLI installer flag support)
  bin/
    root-installer.rs    # Entry point for dedicated `root-installer` Setup Wizard binary
  branding/              # Shared brand titles, RGBA sprout icon rasterizer, and .ico/.svg encoders
    mod.rs               # Public branding constants (`APP_DISPLAY_TITLE`, etc.) and icon re-exports
    icon.rs              # Pure-Rust 64x64 RGBA logo renderer, `egui::IconData`, and Windows `.ico` encoder
    tests.rs             # Unit tests for RGBA logo rendering, `.ico` header framing, and boundary validation
  installer/             # Cross-platform GUI/CLI installer, shortcut generators, and uninstaller
    mod.rs               # Public installer API and CLI argument parser (`--install`, `--uninstall`)
    types.rs             # TargetOs, InstallOptions, InstallPaths, InstallReceipt, and InstallerError
    os_refresh.rs        # Best-effort Linux icon/desktop cache refresh and Windows uninstaller self-deletion
    windows_uninstall_tests.rs # Regression tests for Windows uninstall command, display icon, and SVG size
    paths.rs             # Per-OS path resolution (Windows AppData/Start Menu/root.ico, macOS .app, Linux XDG)
    shortcuts.rs         # Generators for Windows `.lnk` PowerShell scripts, Registry keys, `.desktop`, and `Info.plist`
    engine.rs            # File/icon installation, permission handling, receipt writing, and uninstallation logic
    ui.rs                # Interactive `egui` Setup Wizard (`InstallerApp`)
    tests.rs             # Unit and `tempfile` integration tests for CLI parsing, `.ico` generation, and install/uninstall
  scanner/               # Ignore-aware directory traversal and binary/lockfile classification
    mod.rs               # Public scanner API re-exports
    types.rs             # FileNode, FileCategory, ScanOptions, and ScanError definitions
    binary_check.rs      # Pure functions for lockfile, known text, and binary file detection
    scan_job.rs          # Background thread project scanner streaming live progress over MPSC
    walker.rs            # Filesystem traversal using `ignore` and bottom-up tree assembly
    tests.rs             # Unit and `tempfile` integration tests for scanning and background jobs
  detector/              # Framework fingerprinting and user cross-check state
    mod.rs               # Public detector API and safe root manifest reader
    types.rs             # ProjectKind, ProjectSignals, and ProjectProfile definitions
    rules.rs             # Pure detection rules for Tauri, Electron, WXT, Next.js, Vite, Cargo, and Node.js
    tests.rs             # Unit tests for framework detection and user override state
  selection/             # Tree and file selection state for Markdown code inclusion
    mod.rs               # Public selection API re-exports
    state.rs             # ExportMode and SelectionState with cascading directory checkboxes
    tests.rs             # Unit tests for file selection, mode filtering, and preset flags
  settings/              # Persistent user preferences (theme and zoom factor)
    mod.rs               # Public settings API re-exports
    store.rs             # Pure TOML parsing/clamping and OS config directory persistence
    tests.rs             # Unit and `tempfile` tests for settings persistence and boundary clamping
  markdown/              # ASCII tree generation and fenced code block export
    mod.rs               # Public Markdown exporter API
    tree_formatter.rs    # Pure Unicode box-drawing directory tree formatter
    code_bundler.rs      # Pure syntax tag mapper and backtick-safe fenced code formatter
    export_job.rs        # Background thread `.md` exporter streaming per-file progress over MPSC
    writer.rs            # Full Markdown document builder and disk writer
    tests.rs             # Unit and `tempfile` tests for tree formatting, progress, and `.md` export
  ui/                    # Pure-Rust `egui` desktop interface
    mod.rs               # RootApp state container, background scan/export polling, and layout
    loading_view.rs      # Animated project loading screen with spinner and live entry counts
    preview_pane.rs      # Resizable side panel showing live generated Markdown output
    project_banner.rs    # Interactive banner to confirm or override the detected framework
    status_bar.rs        # Bottom status bar with animated `.md` export progress bar
    theme.rs             # Custom dark/light theme visuals, UI zoom scaling, and widget spacing
    toolbar.rs           # Top toolbar with search filter, zoom/theme toggles, install launcher, copy, and export
    tree_view.rs         # Scrollable project tree view with unique checkbox IDs and search filter
```

## 7. How to Run

```bash
cargo check                        # fast compile check
cargo run --bin root               # run the desktop app in development
cargo run --bin root-installer     # run the graphical Setup Wizard in development
cargo test                         # run all unit and integration tests
cargo build -r --bins              # build optimized Windows release binaries (root.exe & root-installer.exe)
```

### Headless Installer CLI Usage (Windows)

```bash
# Install to %LOCALAPPDATA%\Programs\root with root.ico, Start Menu & Desktop shortcuts
.\target\release\root-installer.exe --install

# Install to a custom directory
.\target\release\root-installer.exe --install --dir "C:\Tools\root"

# Uninstall cleanly and remove shortcuts + Registry entry
.\target\release\root-installer.exe --uninstall
```

## How to Keep This File Updated (Instructions for AI)

**How to deliver the update to me:**

- Keep track of each change and create a ".md" file at the end when I ask for "PROJECT_OVERVIEW.md".
