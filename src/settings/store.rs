//! Pure TOML serialization, boundary validation, and disk persistence for app settings.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Default UI zoom scale (100%).
pub const DEFAULT_ZOOM_FACTOR: f32 = 1.0;

/// Minimum allowed UI zoom scale (70%).
pub const MIN_ZOOM_FACTOR: f32 = 0.7;

/// Maximum allowed UI zoom scale (200%).
pub const MAX_ZOOM_FACTOR: f32 = 2.0;

/// Increment/decrement step per zoom action (10%).
pub const ZOOM_STEP: f32 = 0.1;

/// Subdirectory inside the OS user configuration folder used to store app settings.
const APP_CONFIG_DIR_NAME: &str = "root";

/// Filename for the persisted TOML configuration file.
const SETTINGS_FILE_NAME: &str = "settings.toml";

/// Clamps `zoom_factor` to `[MIN_ZOOM_FACTOR, MAX_ZOOM_FACTOR]` rounded to the nearest tenth.
pub fn clamp_zoom(zoom_factor: f32) -> f32 {
    let clamped = zoom_factor.clamp(MIN_ZOOM_FACTOR, MAX_ZOOM_FACTOR);
    (clamped * 10.0).round() / 10.0
}

/// Persisted user preferences for visual theme and UI scaling.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AppSettings {
    /// Whether the dark color theme is enabled (`true` by default).
    pub is_dark_mode: bool,
    /// UI zoom scale factor (`1.0` = 100%, clamped to `0.7..=2.0`).
    pub zoom_factor: f32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            is_dark_mode: true,
            zoom_factor: DEFAULT_ZOOM_FACTOR,
        }
    }
}

impl AppSettings {
    /// Returns a copy of these settings with `zoom_factor` clamped to valid bounds.
    pub fn sanitized(self) -> Self {
        Self {
            is_dark_mode: self.is_dark_mode,
            zoom_factor: clamp_zoom(self.zoom_factor),
        }
    }
}

/// Parses a raw TOML string into `AppSettings` and clamps `zoom_factor` at the boundary.
pub fn parse_settings_toml(raw_toml: &str) -> Result<AppSettings, String> {
    let parsed: AppSettings =
        toml::from_str(raw_toml).map_err(|err| format!("Failed to parse settings TOML: {err}"))?;
    Ok(parsed.sanitized())
}

/// Serializes `AppSettings` into a formatted TOML string after sanitizing bounds.
pub fn serialize_settings_toml(settings: &AppSettings) -> Result<String, String> {
    let sanitized = settings.sanitized();
    toml::to_string_pretty(&sanitized)
        .map_err(|err| format!("Failed to serialize settings to TOML: {err}"))
}

/// Loads `AppSettings` from `file_path`, falling back to defaults if the file is missing or invalid.
pub fn load_settings_from_path(file_path: &Path) -> AppSettings {
    let Ok(raw_toml) = fs::read_to_string(file_path) else {
        return AppSettings::default();
    };

    parse_settings_toml(&raw_toml).unwrap_or_default()
}

/// Writes `AppSettings` to `file_path`, creating any missing parent directories first.
pub fn save_settings_to_path(file_path: &Path, settings: &AppSettings) -> Result<(), String> {
    if let Some(parent_dir) = file_path.parent() {
        fs::create_dir_all(parent_dir).map_err(|err| {
            format!(
                "Failed to create settings directory '{}': {err}",
                parent_dir.display()
            )
        })?;
    }

    let serialized = serialize_settings_toml(settings)?;
    fs::write(file_path, serialized).map_err(|err| {
        format!(
            "Failed to write settings file '{}': {err}",
            file_path.display()
        )
    })
}

/// Resolves the cross-platform user config path (`%APPDATA%/root/settings.toml` or `~/.config/root/settings.toml`).
pub fn default_settings_file_path() -> PathBuf {
    let base_config_dir = env::var_os("APPDATA")
        .or_else(|| env::var_os("XDG_CONFIG_HOME"))
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .unwrap_or_else(|| PathBuf::from("."));

    base_config_dir
        .join(APP_CONFIG_DIR_NAME)
        .join(SETTINGS_FILE_NAME)
}

/// Loads `AppSettings` from the default OS configuration path.
pub fn load_app_settings() -> AppSettings {
    load_settings_from_path(&default_settings_file_path())
}

/// Saves `AppSettings` to the default OS configuration path.
pub fn save_app_settings(settings: &AppSettings) -> Result<(), String> {
    save_settings_to_path(&default_settings_file_path(), settings)
}
