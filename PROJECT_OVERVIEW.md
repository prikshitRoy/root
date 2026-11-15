# Project Overview

> **Instruction for AI:** This file is the single source of truth for what this project is and what has been built.
> (see "How to Keep This File Updated" at the bottom).

---

## 1. What Is This Project?

**Name:** `root`

A cross-platform **desktop application built in pure Rust** that scans software project directories, detects the project framework (Next.js, Vite, Tauri, Electron, Node.js, Rust/Cargo, etc.), displays an interactive file tree respecting `.gitignore` and `.ignore` rules, and exports the tree structure plus all or selected source files into a single `.md` file.

It is built for developers who want a fast, local-first way to package project structure and source code context into clean Markdown documentation or AI context files.

## 2. Tech Stack

| Area                   | Choice                                                      |
| ---------------------- | ----------------------------------------------------------- |
| Language               | Rust (2021 Edition)                                         |
| Desktop UI             | `eframe` / `egui` (v0.29)                                   |
| Directory traversal    | `ignore` (v0.4)                                             |
| Native OS dialogs      | `rfd` (v0.15)                                               |
| Manifest & config I/O  | `serde` (v1.0), `serde_json` (v1.0), `toml` (v0.8)          |
| Unit/integration tests | Cargo built-in test runner (`cargo test`) + `tempfile` (v3.13) |
| Target platforms       | Windows, Linux, macOS                                       |

## 3. Core Concepts

- **FileNode:** A node in the scanned project hierarchy representing either a directory or a file, along with its relative path, size in bytes, and content classification (`TextCode`, `Lockfile`, or `Binary`).
- **ProjectKind & ProjectProfile:** The auto-detected software project framework (Tauri, Electron, Next.js, Vite, Rust/Cargo, Node.js, or Unknown) paired with detection evidence and user cross-check/override state.
- **SelectionState & ExportMode:** Tracks how source files are bundled into the Markdown document (`TreeOnly`, `AllCode`, or `SelectedCode`) with cascading folder and file checkboxes.
- **ScanJobHandle & ExportJobHandle:** Non-blocking background worker handles using `std::thread` and `std::sync::mpsc` to stream live directory scan counts and `.md` file export progress to the UI.
- **AppSettings:** Persistent user preferences (`is_dark_mode` and `zoom_factor`) stored as TOML in the OS configuration directory (`settings.toml`).

## 4. Features

Status key: **Done** | **In progress** | **Planned**

### 4.1 Workspace and app shell

| Feature                                                                                         | Status  |
| ----------------------------------------------------------------------------------------------- | ------- |
| Cargo project setup with `eframe` native window shell                                           | Done    |
| Feature-grouped module architecture (`scanner`, `detector`, `selection`, `settings`, `markdown`, `ui`) | Done |

### 4.2 Directory scanning and ignore handling

| Feature                                                                                         | Status  |
| ----------------------------------------------------------------------------------------------- | ------- |
| Recursive directory scanning respecting `.gitignore`, `.ignore`, and `.git/info/exclude`        | Done    |
| Fast extension-first file classification and binary/lockfile filtering                          | Done    |
| Non-blocking background directory scan worker streaming live entry counts                       | Done    |

### 4.3 Framework detection

| Feature                                                                                         | Status  |
| ----------------------------------------------------------------------------------------------- | ------- |
| Automatic detection for Next.js, Vite, Tauri, Electron, Node.js, and Rust/Cargo projects        | Done    |
| Interactive UI banner to cross-check, confirm, or override the detected project type            | Done    |

### 4.4 Tree selection and Markdown export

| Feature                                                                                         | Status  |
| ----------------------------------------------------------------------------------------------- | ------- |
| ASCII/Unicode tree structure generator                                                          | Done    |
| Export modes: Tree Only, Tree + All Code, and Tree + Selected Code                              | Done    |
| Fenced code block bundler with file path headers, syntax tags, and dynamic backtick escaping    | Done    |
| Native folder picker and `.md` file save dialog via `rfd`                                       | Done    |
| Background `.md` export worker with real-time animated progress bar and one-click clipboard copy | Done   |

### 4.5 User interface

| Feature                                                                                         | Status  |
| ----------------------------------------------------------------------------------------------- | ------- |
| Interactive collapsible tree view with cascading folder/file checkboxes                         | Done    |
| Dynamic `All Code` and `None` preset buttons with live checkmark (`☑`/`☐`) and active states    | Done    |
| Animated project loading view displaying live scanned file counts and current path              | Done    |
| Live Markdown preview pane                                                                      | Done    |
| Live file/folder search filter, summary stats bar, and Dark/Light theme switcher                | Done    |
| UI Zoom In / Zoom Out / Reset controls (`70%`–`200%`) via toolbar buttons and keyboard shortcuts | Done    |
| Persistent TOML app settings (`settings.toml`) for saving and restoring theme and zoom state    | Done    |

### 4.6 Testing

| Feature                                                                                         | Status  |
| ----------------------------------------------------------------------------------------------- | ------- |
| Unit tests for pure detector, selection, settings, UI helpers, and Markdown formatting logic    | Done    |
| Integration tests using `tempfile` for ignore-aware directory scanning, export, and settings    | Done    |

## 5. Known Issues and Open Items

> Verify these against the current code; remove an item when it is fixed.

- None recorded.

## 6. Project Structure

```text
Cargo.toml               # Rust package manifest and dependencies
README.md                # User and developer documentation
PROJECT_OVERVIEW.md      # Single source of truth for project status and architecture
src/
  main.rs                # Application entry point and native window initialization
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
    rules.rs             # Pure detection rules for Tauri, Electron, Next.js, Vite, Cargo, and Node.js
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
    toolbar.rs           # Top toolbar with search filter, zoom/theme toggles, copy, and export
    tree_view.rs         # Scrollable project tree view with unique checkbox IDs and search filter
```

## 7. How to Run

```bash
cargo check         # fast compile check
cargo run           # run the desktop app in development
cargo test          # run all unit and integration tests
cargo build -r      # build optimized release binary
```


## How to Keep This File Updated (Instructions for AI)

**How to deliver the update to me:**

- Keep track of each change and create a ".md" file at the end when I ask for "PROJECT_OVERVIEW.md".
