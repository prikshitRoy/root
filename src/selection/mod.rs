//! Public API for managing tree and file selection modes before Markdown export.

mod state;

#[cfg(test)]
mod tests;

pub use state::{ExportMode, SelectionState};
