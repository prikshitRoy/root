# 🌿 root

**root** is a fast, cross-platform desktop application built in **100% pure Rust** (`eframe` / `egui`) for inspecting software repository trees and bundling project structure and source code into a single, clean Markdown (`.md`) file.

It compiles into both a standalone portable desktop executable (`root`) that runs directly and a built-in graphical/CLI Setup Wizard (`root-installer`) for installing the application, icons, and system shortcuts.

---

## ✨ Key Features

- **Ignore-Aware Directory Scanning:** Automatically respects `.gitignore`, `.ignore`, `.git/info/exclude`, and global gitignore rules using the `ignore` crate—even in repositories where `git init` has not been run yet.
- **Non-Blocking Background Scanner & Loading UI:** Scans large projects on a background thread while displaying a live spinner and discovered file count so the desktop UI never freezes.
- **Automatic Framework Detection & User Cross-Check:** Detects **Tauri**, **Electron**, **WXT (Browser Extension)**, **Next.js**, **Vite**, **Rust (Cargo)**, and **Node.js** projects from root config files and manifests, and lets you confirm or override the detected framework in the UI.
- **Flexible Markdown Code Bundling:**
  - **Tree Only:** Exports only the Unicode box-drawing directory structure (`├──`, `└──`, `│`).
  - **Tree + All Code:** Exports the directory tree plus all non-binary, non-lockfile source code files.
  - **Tree + Selected Code:** Interactive cascading checkboxes on folders and individual files, plus quick **☑ All Code** and **☐ None** preset buttons.
- **Smart Binary & Lockfile Filtering:** Keeps heavy lockfiles (`Cargo.lock`, `yarn.lock`, `package-lock.json`, `pnpm-lock.yaml`, etc.) and binary assets visible in the directory tree while excluding them from bloating code dumps by default.
- **Backtick-Safe Fenced Code Blocks:** Automatically tags fenced code blocks with syntax identifiers (`rust`, `tsx`, `ts`, `json`, `toml`, etc.) and dynamically expands fence lengths (` ```` `) when a source file itself contains triple backticks.
- **Real-Time Export Progress & Clipboard Copy:** Streams per-file export progress to an animated bottom progress bar when saving `.md` files, or copies the generated Markdown directly to your clipboard in one click.
- **Persistent UI Theme & Zoom:** Switch between **Dark** and **Light** mode and scale the interface from **70% to 200%** (`settings.toml` is automatically saved in your OS config folder).
- **Built-In Setup Wizard & Uninstaller (`root-installer`):**
  - **Graphical & CLI Modes:** Launch via `root-installer`, the `📦 Install...` button in the app toolbar, or headless CLI flags (`--install`, `--uninstall`, `--dir`).
  - **Pure-Rust `🌿 root` Branding & Icons:** Automatically renders the `🌿 root` window/taskbar icon (`egui::IconData`), generates a 32-bit Windows `root.ico` file for `.lnk` shortcuts and the Windows Installed Apps registry, and writes `root.svg` on Linux/macOS.

---
## 🚀 Getting Started

### Download Prebuilt Installers & Executables

Download the latest release for your OS from the [**GitHub Releases**](../../releases/latest) page (see [`RELEASING.md`](./RELEASING.md) for checksum verification and full details):

| OS | Recommended Installer | Portable Executable / Archive |
| :--- | :--- | :--- |
| **Windows (x64)** | `root-setup-windows-x64.exe` (GUI/CLI Setup Wizard) | `root-windows-x64.exe` or `root-windows-x64.zip` |
| **macOS (Apple Silicon)** | `root-macos-arm64.dmg` (Drag-and-drop Disk Image) | `root-macos-arm64.app.zip` or `root-macos-arm64.tar.gz` |
| **macOS (Intel)** | `root-macos-x64.dmg` (Drag-and-drop Disk Image) | `root-macos-x64.app.zip` or `root-macos-x64.tar.gz` |
| **Linux (x64)** | `root-linux-amd64.deb` (Ubuntu/Debian) or `root-installer-linux-x64` | `root-linux-x64.tar.gz` |

> **Platform Testing Status:**
> - **Windows (x64) & Linux (Ubuntu x64):** Manually tested and verified on real systems, in addition to automated CI tests.
> - **macOS (Apple Silicon & Intel):** Built and verified via automated unit/integration tests on GitHub Actions macOS runners, but **not yet manually tested on physical Mac hardware**. If you use macOS and encounter any issues, please [open an issue](../../issues).

### Prerequisites (Building from Source)

- [Rust toolchain](https://rustup.rs/) (`rustc` and `cargo`, 2021 edition or newer)

### Run in Development

```bash
# Run the main desktop application
cargo run --bin root

# Run the graphical Setup Wizard
cargo run --bin root-installer
```

### Build Optimized Release Binaries (Windows)

```bash
cargo build --release --bins
```

This produces two executables in `target\release\`:

- `target\release\root.exe` — Portable desktop app that runs directly.
- `target\release\root-installer.exe` — Graphical Setup Wizard and CLI installer/uninstaller.

### Headless CLI Install & Uninstall

```bash
# Install to %LOCALAPPDATA%\Programs\root with Start Menu & Desktop shortcuts
.\target\release\root-installer.exe --install

# Install to a custom directory
.\target\release\root-installer.exe --install --dir "C:\Tools\root"

# Uninstall cleanly and remove shortcuts + Registry entry
.\target\release\root-installer.exe --uninstall
```

### Run Unit & Integration Tests

```bash
cargo test
```

---

## ⌨️ Keyboard Shortcuts & Controls

| Shortcut / Control             | Action                                                |
| ------------------------------ | ----------------------------------------------------- |
| `Ctrl + +` / `Ctrl + =` (`➕`) | Zoom In UI (+10%, up to 200%)                         |
| `Ctrl + -` (`➖`)              | Zoom Out UI (-10%, down to 70%)                       |
| `Ctrl + 0` (`100%` button)     | Reset UI Zoom to 100%                                 |
| `🌙 Dark` / `☀ Light`          | Toggle between Dark and Light themes                  |
| `📦 Install...`                | Open the `root Setup` installer wizard                |
| `🔍 Filter files or folders`   | Live case-insensitive filter across the tree          |
| `☑ All Code` / `☐ None`        | Select or deselect all non-binary source files        |
| `👁 Live Preview`               | Toggle the resizable live Markdown preview panel      |
| `📋 Copy .md`                  | Copy the full Markdown bundle to clipboard            |
| `💾 Export .md...`             | Save the Markdown bundle to disk with progress        |

---

## ⚙️ Persistent Settings

Your theme (`is_dark_mode`) and UI zoom level (`zoom_factor`) are automatically saved whenever changed:

- **Windows:** `%APPDATA%\root\settings.toml`
- **Linux / macOS:** `~/.config/root/settings.toml` (or `$XDG_CONFIG_HOME/root/settings.toml`)

---

## 🗂️ Project Architecture

The codebase is organized by feature domain with strict one-way dependencies (`ui / installer -> core modules -> filesystem`):

```text
src/
  lib.rs                 # Shared library root exposing desktop app and installer runners
  main.rs                # Entry point for portable `root` binary (with CLI installer flags)
  bin/
    root-installer.rs    # Entry point for dedicated `root-installer` Setup Wizard binary
  branding/              # Brand constants, RGBA sprout icon rasterizer, and pure-Rust .ico/.svg encoders
  installer/             # Cross-platform GUI/CLI installer, OS shortcut/Registry generators, and uninstaller
  scanner/               # Ignore-aware directory traversal, binary detection, and background scan worker
  detector/              # Framework fingerprinting rules (Tauri, Electron, WXT, Next.js, Vite, Cargo, Node.js)
  selection/             # Tree selection state, ExportMode, and cascading directory checkbox logic
  settings/              # Persistent TOML settings serialization, boundary clamping, and disk I/O
  markdown/              # Unicode box-drawing tree formatter, fenced code bundler, and export worker
  ui/                    # Pure-Rust egui panels, toolbar, tree view, loading screen, and status bar
```
