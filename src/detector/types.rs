//! Data types representing detected software frameworks and user verification state.

/// Supported software project frameworks recognized by `root`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectKind {
    /// Tauri desktop/mobile application (`src-tauri`, `tauri.conf.json`, or `@tauri-apps/*`).
    Tauri,
    /// Electron desktop application (`electron` dependency or builder config).
    Electron,
    /// WXT universal web browser extension (`wxt.config.*` or `wxt` dependency).
    Wxt,
    /// Next.js web application (`next.config.*` or `next` dependency).
    NextJs,
    /// Vite frontend application (`vite.config.*` or `vite` dependency).
    Vite,
    /// Pure Rust / Cargo crate or workspace (`Cargo.toml`).
    RustCargo,
    /// Generic Node.js / TypeScript / JavaScript project (`package.json`).
    NodeJs,
    /// Unrecognized or custom project structure.
    Unknown,
}

impl ProjectKind {
    /// Ordered list of all selectable project kinds for the UI cross-check dropdown.
    pub const ALL: &'static [ProjectKind] = &[
        ProjectKind::Tauri,
        ProjectKind::Electron,
        ProjectKind::Wxt,
        ProjectKind::NextJs,
        ProjectKind::Vite,
        ProjectKind::RustCargo,
        ProjectKind::NodeJs,
        ProjectKind::Unknown,
    ];

    /// Human-readable display name for UI labels and dropdowns.
    pub fn label(self) -> &'static str {
        match self {
            Self::Tauri => "Tauri",
            Self::Electron => "Electron",
            Self::Wxt => "WXT (Browser Extension)",
            Self::NextJs => "Next.js",
            Self::Vite => "Vite",
            Self::RustCargo => "Rust (Cargo)",
            Self::NodeJs => "Node.js",
            Self::Unknown => "Other / Unknown",
        }
    }
}

/// Raw filesystem signals collected from a project root to run pure detection rules.
#[derive(Debug, Clone, Default)]
pub struct ProjectSignals {
    /// File and folder names directly inside the project root.
    pub root_entry_names: Vec<String>,
    /// Raw text contents of `package.json` if present at the root.
    pub package_json_content: Option<String>,
    /// Raw text contents of `Cargo.toml` if present at the root.
    pub cargo_toml_content: Option<String>,
}

/// Holds both the auto-detected framework and the user's cross-checked selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectProfile {
    /// Framework automatically inferred by `detector::rules`.
    pub detected_kind: ProjectKind,
    /// Framework currently active (starts as `detected_kind`, updated if the user overrides it).
    pub active_kind: ProjectKind,
    /// Short explanation of which file or dependency triggered the detection.
    pub evidence: String,
    /// Whether the user has explicitly confirmed or overridden the project framework.
    pub is_user_confirmed: bool,
}

impl ProjectProfile {
    /// Creates a new unconfirmed profile from an automatic detection result.
    pub fn new(detected_kind: ProjectKind, evidence: impl Into<String>) -> Self {
        Self {
            detected_kind,
            active_kind: detected_kind,
            evidence: evidence.into(),
            is_user_confirmed: false,
        }
    }

    /// Marks the currently active framework as verified by the user.
    pub fn confirm(&mut self) {
        self.is_user_confirmed = true;
    }

    /// Overrides the active framework with a user-chosen `ProjectKind` and marks it confirmed.
    pub fn select_kind(&mut self, chosen_kind: ProjectKind) {
        self.active_kind = chosen_kind;
        self.is_user_confirmed = true;
    }
}
