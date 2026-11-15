//! Unit tests for pure framework detection rules and user verification state.

use crate::detector::{detect_from_signals, ProjectKind, ProjectSignals};

#[test]
fn detects_tauri_project_and_prioritizes_it_over_vite_config() {
    // Arrange: hybrid Tauri + Vite project has both `src-tauri` and `vite.config.ts`
    let signals = ProjectSignals {
        root_entry_names: vec![
            "src-tauri".to_string(),
            "vite.config.ts".to_string(),
            "package.json".to_string(),
        ],
        package_json_content: Some(r#"{"devDependencies": {"vite": "^5.0.0"}}"#.to_string()),
        cargo_toml_content: None,
    };

    // Act
    let profile = detect_from_signals(&signals);

    // Assert
    assert_eq!(profile.detected_kind, ProjectKind::Tauri);
    assert_eq!(profile.active_kind, ProjectKind::Tauri);
    assert!(!profile.is_user_confirmed);
}

#[test]
fn detects_electron_from_package_json_dev_dependencies() {
    // Arrange
    let signals = ProjectSignals {
        root_entry_names: vec!["package.json".to_string()],
        package_json_content: Some(
            r#"{"name": "desktop-app", "devDependencies": {"electron": "^31.0.0"}}"#.to_string(),
        ),
        cargo_toml_content: None,
    };

    // Act
    let profile = detect_from_signals(&signals);

    // Assert
    assert_eq!(profile.detected_kind, ProjectKind::Electron);
    assert_eq!(profile.evidence, "Found 'electron' in package.json");
}

#[test]
fn detects_nextjs_from_next_config_file() {
    // Arrange
    let signals = ProjectSignals {
        root_entry_names: vec!["next.config.ts".to_string(), "package.json".to_string()],
        package_json_content: None,
        cargo_toml_content: None,
    };

    // Act
    let profile = detect_from_signals(&signals);

    // Assert
    assert_eq!(profile.detected_kind, ProjectKind::NextJs);
    assert_eq!(profile.evidence, "Found 'next.config.ts' in project root");
}

#[test]
fn detects_vite_from_package_json_dependencies() {
    // Arrange
    let signals = ProjectSignals {
        root_entry_names: vec!["package.json".to_string()],
        package_json_content: Some(r#"{"devDependencies": {"vite": "^5.4.0"}}"#.to_string()),
        cargo_toml_content: None,
    };

    // Act
    let profile = detect_from_signals(&signals);

    // Assert
    assert_eq!(profile.detected_kind, ProjectKind::Vite);
}

#[test]
fn detects_rust_cargo_when_cargo_toml_exists() {
    // Arrange
    let signals = ProjectSignals {
        root_entry_names: vec!["Cargo.toml".to_string(), "src".to_string()],
        package_json_content: None,
        cargo_toml_content: Some("[package]\nname = \"root\"".to_string()),
    };

    // Act
    let profile = detect_from_signals(&signals);

    // Assert
    assert_eq!(profile.detected_kind, ProjectKind::RustCargo);
}

#[test]
fn falls_back_to_nodejs_when_package_json_has_invalid_json() {
    // Arrange
    let signals = ProjectSignals {
        root_entry_names: vec!["package.json".to_string()],
        package_json_content: Some("{broken json".to_string()),
        cargo_toml_content: None,
    };

    // Act
    let profile = detect_from_signals(&signals);

    // Assert
    assert_eq!(profile.detected_kind, ProjectKind::NodeJs);
}

#[test]
fn returns_unknown_when_no_known_files_exist() {
    // Arrange
    let signals = ProjectSignals {
        root_entry_names: vec!["README.md".to_string()],
        package_json_content: None,
        cargo_toml_content: None,
    };

    // Act
    let profile = detect_from_signals(&signals);

    // Assert
    assert_eq!(profile.detected_kind, ProjectKind::Unknown);
}

#[test]
fn updates_active_kind_and_confirmation_flag_when_user_overrides_framework() {
    // Arrange: auto-detected as Tauri, but user wants to treat it as Next.js
    let signals = ProjectSignals {
        root_entry_names: vec!["src-tauri".to_string(), "next.config.mjs".to_string()],
        package_json_content: None,
        cargo_toml_content: None,
    };
    let mut profile = detect_from_signals(&signals);

    // Act
    profile.select_kind(ProjectKind::NextJs);

    // Assert
    assert_eq!(profile.detected_kind, ProjectKind::Tauri);
    assert_eq!(profile.active_kind, ProjectKind::NextJs);
    assert!(profile.is_user_confirmed);
}
