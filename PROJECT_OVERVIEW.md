# Project Overview

> **Instruction for AI:** This file is the single source of truth for what this project is and what has been built.
> Read it at the start of every session. **Whenever you add, change, fix, or remove anything, update this file**
> (see "How to Keep This File Updated" at the bottom). Never leave it out of date.

---

## 1. What Is This Project?

**Name:** `root`

A cross-platform **desktop application built in pure Rust** that scans software project directories, detects the project framework (Next.js, Vite, Tauri, Electron, Rust/Cargo, etc.), displays an interactive file tree respecting `.gitignore` and `.ignore` rules, and exports the tree structure plus all or selected source files into a single `.md` file.

It is built for developers who want a fast, local-first way to package project structure and source code context into clean Markdown documentation or AI context files.

## 2. Tech Stack

| Area                 | Choice                                                      |
| -------------------- | ----------------------------------------------------------- |
| Language             | Rust (2021 Edition)                                         |
| Desktop UI           | `eframe` / `egui` (v0.29)                                   |
| Directory traversal  | `ignore` (v0.4)                                             |
| Native OS dialogs    | `rfd` (v0.15)                                               |
| Manifest parsing     | `serde`, `serde_json`, `toml`                               |
| Unit/integration tests | Cargo built-in test runner (`cargo test`) + `tempfile`    |
| Target platforms     | Windows, Linux, macOS                                       |

## 3. Core Concepts

- **FileNode:** A node in the scanned project hierarchy representing either a directory or a file, along with relative path, size, and binary/text classification.
- **ProjectKind:** The detected software project type (e.g., Next.js, Vite, Tauri, Electron, Cargo, Unknown) derived from root config files and manifest dependencies.
- **SelectionState:** Tracks whether each file or directory is included when exporting source code blocks into the Markdown document (Tree Only, All Code, or Selected Code).
- **MarkdownBundle:** The generated `.md` document containing the ASCII/Unicode directory tree and optional syntax-tagged fenced code blocks.

## 4. How to Run

```bash
cargo check         # fast compile check
cargo run           # run the desktop app in development
cargo test          # run all unit and integration tests
cargo build -r      # build optimized release binary

