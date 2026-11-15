# 🌿 root

**root** is a fast, cross-platform desktop application built in **100% pure Rust** (`eframe` / `egui`) for inspecting software repository trees and bundling project structure and source code into a single, clean Markdown (`.md`) file.

---

## ✨ Key Features

- **Ignore-Aware Directory Scanning:** Automatically respects `.gitignore`, `.ignore`, `.git/info/exclude`, and global gitignore rules using the `ignore` crate—even in repositories where `git init` has not been run yet.
- **Non-Blocking Background Scanner & Loading UI:** Scans large projects on a background thread while displaying a live spinner and discovered file count so the desktop UI never freezes.
- **Automatic Framework Detection & User Cross-Check:** Detects **Tauri**, **Electron**, **Next.js**, **Vite**, **Rust (Cargo)**, and **Node.js** projects from root config files and manifests, and lets you confirm or override the detected framework in the UI.
- **Flexible Markdown Code Bundling:**
  - **Tree Only:** Exports only the Unicode box-drawing directory structure (`├──`, `└──`, `│`).
  - **Tree + All Code:** Exports the directory tree plus all non-binary, non-lockfile source code files.
  - **Tree + Selected Code:** Interactive cascading checkboxes on folders and individual files, plus quick **☑ All Code** and **☐ None** preset buttons.
- **Smart Binary & Lockfile Filtering:** Keeps heavy lockfiles (`Cargo.lock`, `yarn.lock`, `package-lock.json`, `pnpm-lock.yaml`, etc.) and binary assets visible in the directory tree while excluding them from bloating code dumps by default.
- **Backtick-Safe Fenced Code Blocks:** Automatically tags fenced code blocks with syntax identifiers (`rust`, `tsx`, `ts`, `json`, `toml`, etc.) and dynamically expands fence lengths (` ```` `) when a source file itself contains triple backticks.
- **Real-Time Export Progress & Clipboard Copy:** Streams per-file export progress to an animated bottom progress bar when saving `.md` files, or copies the generated Markdown directly to your clipboard in one click.
- **Persistent UI Theme & Zoom:** Switch between **Dark** and **Light** mode and scale the interface from **70% to 200%** (`settings.toml` is automatically saved in your OS config folder).

---

## 🚀 Getting Started

### Prerequisites

- [Rust toolchain](https://rustup.rs/) (`rustc` and `cargo`, 2021 edition or newer)

### Run in Development

```bash
cargo run
```

### Build Optimized Release Binary

```bash
cargo build --release
```

The compiled desktop binary will be placed in `target/release/root` (or `target\release\root.exe` on Windows).

### Run Unit & Integration Tests

```bash
cargo test
```

---

## ⌨️ Keyboard Shortcuts & Controls

| Shortcut / Control             | Action                                           |
| ------------------------------ | ------------------------------------------------ |
| `Ctrl + +` / `Ctrl + =` (`➕`) | Zoom In UI (+10%, up to 200%)                    |
| `Ctrl + -` (`➖`)              | Zoom Out UI (-10%, down to 70%)                  |
| `Ctrl + 0` (`100%` button)     | Reset UI Zoom to 100%                            |
| `🌙 Dark` / `☀ Light`          | Toggle between Dark and Light themes             |
| `🔍 Filter files or folders`   | Live case-insensitive filter across the tree     |
| `☑ All Code` / `☐ None`        | Select or deselect all non-binary source files   |
| `👁 Live Preview`               | Toggle the resizable live Markdown preview panel |
| `📋 Copy .md`                  | Copy the full Markdown bundle to clipboard       |
| `💾 Export .md...`             | Save the Markdown bundle to disk with progress   |

---

## ⚙️ Persistent Settings

Your theme (`is_dark_mode`) and UI zoom level (`zoom_factor`) are automatically saved whenever changed:

- **Windows:** `%APPDATA%\root\settings.toml`
- **Linux / macOS:** `~/.config/root/settings.toml` (or `$XDG_CONFIG_HOME/root/settings.toml`)

---

## 🗂️ Project Architecture

The codebase is organized by feature domain with strict one-way dependencies (`ui -> core modules -> filesystem`):

```text
src/
  main.rs                # Entry point and native window initialization
  scanner/               # Ignore-aware directory traversal, binary detection, and background scan worker
  detector/              # Framework fingerprinting rules (Tauri, Electron, Next.js, Vite, Cargo, Node.js)
  selection/             # Tree selection state, ExportMode, and cascading directory checkbox logic
  settings/              # Persistent TOML settings serialization, boundary clamping, and disk I/O
  markdown/              # Unicode box-drawing tree formatter, fenced code bundler, and export worker
  ui/                    # Pure-Rust egui panels, toolbar, tree view, loading screen, and status bar
```