//! Unit and `tempfile` integration tests for `settings`.

use std::fs;

use tempfile::TempDir;

use crate::settings::{
    load_settings_from_path, parse_settings_toml, save_settings_to_path, AppSettings,
    DEFAULT_ZOOM_FACTOR, MAX_ZOOM_FACTOR, MIN_ZOOM_FACTOR,
};

#[test]
fn saves_and_reloads_theme_and_zoom_settings_from_disk() {
    // Arrange
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("nested").join("settings.toml");
    let custom_settings = AppSettings {
        is_dark_mode: false,
        zoom_factor: 1.4,
    };

    // Act
    save_settings_to_path(&file_path, &custom_settings).unwrap();
    let loaded = load_settings_from_path(&file_path);

    // Assert
    assert_eq!(loaded, custom_settings);
}

#[test]
fn clamps_out_of_bounds_zoom_values_when_parsing_toml() {
    // Arrange
    let too_high_toml = "is_dark_mode = true\nzoom_factor = 9.5\n";
    let too_low_toml = "is_dark_mode = false\nzoom_factor = 0.2\n";

    // Act
    let high_settings = parse_settings_toml(too_high_toml).unwrap();
    let low_settings = parse_settings_toml(too_low_toml).unwrap();

    // Assert
    assert_eq!(high_settings.zoom_factor, MAX_ZOOM_FACTOR);
    assert_eq!(low_settings.zoom_factor, MIN_ZOOM_FACTOR);
    assert!(!low_settings.is_dark_mode);
}

#[test]
fn returns_default_settings_when_file_does_not_exist() {
    // Arrange
    let temp = TempDir::new().unwrap();
    let missing_path = temp.path().join("does_not_exist.toml");

    // Act
    let loaded = load_settings_from_path(&missing_path);

    // Assert
    assert!(loaded.is_dark_mode);
    assert_eq!(loaded.zoom_factor, DEFAULT_ZOOM_FACTOR);
}

#[test]
fn returns_default_settings_when_toml_file_is_corrupted() {
    // Arrange
    let temp = TempDir::new().unwrap();
    let broken_path = temp.path().join("settings.toml");
    fs::write(&broken_path, "not valid toml = = =").unwrap();

    // Act
    let loaded = load_settings_from_path(&broken_path);

    // Assert
    assert_eq!(loaded, AppSettings::default());
}
