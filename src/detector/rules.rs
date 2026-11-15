//! Pure rules for fingerprinting software projects from root filenames and manifest contents.

use serde_json::Value;

use crate::detector::types::{ProjectKind, ProjectProfile, ProjectSignals};

/// Config filenames that identify a Tauri project at the root.
const TAURI_ROOT_MARKERS: &[&str] = &["src-tauri", "tauri.conf.json", "tauri.conf.json5"];

/// Config filenames that identify an Electron project at the root.
const ELECTRON_CONFIG_FILES: &[&str] = &[
    "electron-builder.yml",
    "electron-builder.yaml",
    "electron-builder.json",
    "forge.config.js",
    "forge.config.ts",
    "electron.vite.config.ts",
    "electron.vite.config.js",
];

/// Config filenames that identify a Next.js project at the root.
const NEXTJS_CONFIG_FILES: &[&str] = &[
    "next.config.js",
    "next.config.mjs",
    "next.config.ts",
    "next.config.cjs",
];

/// Config filenames that identify a Vite project at the root.
const VITE_CONFIG_FILES: &[&str] = &[
    "vite.config.ts",
    "vite.config.js",
    "vite.config.mjs",
    "vite.config.mts",
];

/// Evaluates `ProjectSignals` in priority order and returns an unconfirmed `ProjectProfile`.
pub fn detect_from_signals(signals: &ProjectSignals) -> ProjectProfile {
    let parsed_pkg = signals
        .package_json_content
        .as_deref()
        .and_then(|raw| serde_json::from_str::<Value>(raw).ok());

    if let Some(evidence) = match_tauri(&signals.root_entry_names, parsed_pkg.as_ref()) {
        return ProjectProfile::new(ProjectKind::Tauri, evidence);
    }

    if let Some(evidence) = match_electron(&signals.root_entry_names, parsed_pkg.as_ref()) {
        return ProjectProfile::new(ProjectKind::Electron, evidence);
    }

    if let Some(evidence) = match_nextjs(&signals.root_entry_names, parsed_pkg.as_ref()) {
        return ProjectProfile::new(ProjectKind::NextJs, evidence);
    }

    if let Some(evidence) = match_vite(&signals.root_entry_names, parsed_pkg.as_ref()) {
        return ProjectProfile::new(ProjectKind::Vite, evidence);
    }

    if signals.cargo_toml_content.is_some() || has_entry(&signals.root_entry_names, "Cargo.toml") {
        return ProjectProfile::new(ProjectKind::RustCargo, "Found Cargo.toml in project root");
    }

    if signals.package_json_content.is_some()
        || has_entry(&signals.root_entry_names, "package.json")
    {
        return ProjectProfile::new(ProjectKind::NodeJs, "Found package.json in project root");
    }

    ProjectProfile::new(
        ProjectKind::Unknown,
        "No known framework config or manifest found",
    )
}

/// Checks for Tauri directory/config markers or `@tauri-apps/*` packages.
fn match_tauri(root_entries: &[String], pkg_json: Option<&Value>) -> Option<String> {
    if let Some(marker) = find_matching_entry(root_entries, TAURI_ROOT_MARKERS) {
        return Some(format!("Found '{marker}' in project root"));
    }

    for dep in ["@tauri-apps/cli", "@tauri-apps/api"] {
        if has_npm_dependency(pkg_json, dep) {
            return Some(format!("Found '{dep}' in package.json"));
        }
    }

    None
}

/// Checks for Electron builder/forge configs or the `electron` package.
fn match_electron(root_entries: &[String], pkg_json: Option<&Value>) -> Option<String> {
    if let Some(config) = find_matching_entry(root_entries, ELECTRON_CONFIG_FILES) {
        return Some(format!("Found '{config}' in project root"));
    }

    if has_npm_dependency(pkg_json, "electron") {
        return Some("Found 'electron' in package.json".to_string());
    }

    None
}

/// Checks for `next.config.*` files or the `next` dependency in `package.json`.
fn match_nextjs(root_entries: &[String], pkg_json: Option<&Value>) -> Option<String> {
    if let Some(config) = find_matching_entry(root_entries, NEXTJS_CONFIG_FILES) {
        return Some(format!("Found '{config}' in project root"));
    }

    if has_npm_dependency(pkg_json, "next") {
        return Some("Found 'next' in package.json".to_string());
    }

    None
}

/// Checks for `vite.config.*` files or the `vite` dependency in `package.json`.
fn match_vite(root_entries: &[String], pkg_json: Option<&Value>) -> Option<String> {
    if let Some(config) = find_matching_entry(root_entries, VITE_CONFIG_FILES) {
        return Some(format!("Found '{config}' in project root"));
    }

    if has_npm_dependency(pkg_json, "vite") {
        return Some("Found 'vite' in package.json".to_string());
    }

    None
}

/// Returns the first candidate name that matches an entry in `root_entries` (case-insensitive).
fn find_matching_entry<'a>(root_entries: &[String], candidates: &[&'a str]) -> Option<&'a str> {
    candidates
        .iter()
        .copied()
        .find(|&candidate| has_entry(root_entries, candidate))
}

/// Returns `true` if `target` exists in `root_entries` (case-insensitive).
fn has_entry(root_entries: &[String], target: &str) -> bool {
    root_entries
        .iter()
        .any(|entry| entry.eq_ignore_ascii_case(target))
}

/// Checks `dependencies` and `devDependencies` inside a parsed `package.json` for `pkg_name`.
fn has_npm_dependency(pkg_json: Option<&Value>, pkg_name: &str) -> bool {
    let Some(root_obj) = pkg_json.and_then(Value::as_object) else {
        return false;
    };

    ["dependencies", "devDependencies"].iter().any(|section| {
        root_obj
            .get(*section)
            .and_then(Value::as_object)
            .is_some_and(|deps| deps.contains_key(pkg_name))
    })
}
