//! Public API for loading and saving persistent application settings.

mod store;

#[cfg(test)]
mod tests;

pub use store::{
    clamp_zoom, default_settings_file_path, load_app_settings, load_settings_from_path,
    parse_settings_toml, save_app_settings, save_settings_to_path, serialize_settings_toml,
    AppSettings, DEFAULT_ZOOM_FACTOR, MAX_ZOOM_FACTOR, MIN_ZOOM_FACTOR, ZOOM_STEP,
};
