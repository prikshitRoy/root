//! Public branding constants and cross-platform icon generators for `root`.

mod icon;

#[cfg(test)]
mod tests;

pub use icon::{
    build_app_ico_bytes, build_logo_rgba, build_window_icon, encode_rgba_to_ico, APP_ICON_SVG,
    DEFAULT_ICON_SIZE,
};

/// Canonical lowercase application name.
pub const APP_NAME: &str = "root";

/// Branded heading and window title with the herb/sprout logo prefix.
pub const APP_DISPLAY_TITLE: &str = "🌿 root";

/// Branded onboarding card heading shown when no project is open.
pub const APP_WELCOME_TITLE: &str = "🌿 Welcome to root";

/// Branded window title for the cross-platform Setup Wizard.
pub const INSTALLER_WINDOW_TITLE: &str = "🌿 root Setup — Cross-Platform Installer";
