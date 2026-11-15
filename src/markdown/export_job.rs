//! Background thread runner and progress reporting for `.md` file exports.

use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};
use std::thread;

use crate::detector::ProjectKind;
use crate::markdown::code_bundler::format_code_section;
use crate::markdown::tree_formatter::format_ascii_tree;
use crate::markdown::writer::save_markdown_file;
use crate::scanner::FileNode;
use crate::selection::{ExportMode, SelectionState};

/// Snapshot of the current Markdown export progress sent from the worker thread to the UI.
#[derive(Debug, Clone, PartialEq)]
pub enum ExportState {
    /// Export is actively formatting files or writing the `.md` bundle.
    Running {
        /// Number of files processed so far.
        processed_files: usize,
        /// Total number of files to be embedded in the Markdown bundle.
        total_files: usize,
        /// Relative path of the file currently being processed.
        current_file: String,
        /// Normalized progress value in `0.0..=1.0`.
        fraction: f32,
    },
    /// Export finished and the `.md` file was written to disk.
    Completed {
        /// Destination path where the `.md` file was saved.
        saved_path: PathBuf,
        /// Total number of source code files embedded.
        total_files: usize,
    },
    /// Export failed due to an I/O error.
    Failed(String),
}

/// Handle held by the UI thread to poll non-blocking updates from a running export job.
pub struct ExportJobHandle {
    receiver: Receiver<ExportState>,
}

impl ExportJobHandle {
    /// Polls all pending progress messages from the worker thread and returns the latest state.
    pub fn poll_latest(&self) -> Option<ExportState> {
        let mut latest = None;
        while let Ok(state) = self.receiver.try_recv() {
            latest = Some(state);
        }
        latest
    }
}

/// Spawns a background thread that builds and writes the `.md` file while streaming progress.
pub fn spawn_export_job(
    root_node: FileNode,
    project_kind: ProjectKind,
    selection: SelectionState,
    max_file_size_bytes: u64,
    destination_path: PathBuf,
) -> ExportJobHandle {
    let (sender, receiver) = mpsc::channel();

    thread::spawn(move || {
        run_export_pipeline(
            &root_node,
            project_kind,
            &selection,
            max_file_size_bytes,
            &destination_path,
            &sender,
        );
    });

    ExportJobHandle { receiver }
}

/// Collects all file nodes that qualify for code inclusion under `selection` and `max_file_size_bytes`.
pub fn collect_exportable_files<'a>(
    root_node: &'a FileNode,
    selection: &SelectionState,
    max_file_size_bytes: u64,
) -> Vec<&'a FileNode> {
    let mut files = Vec::new();
    if selection.export_mode != ExportMode::TreeOnly {
        collect_files_recursively(root_node, selection, max_file_size_bytes, &mut files);
    }
    files
}

/// Calculates a normalized progress fraction in `0.0..=1.0` for `processed` out of `total` items.
pub fn calculate_progress_fraction(processed: usize, total: usize) -> f32 {
    if total == 0 {
        return 1.0;
    }
    (processed as f32 / total as f32).clamp(0.0, 1.0)
}

/// Executes the tree formatting, step-by-step file bundling, and disk write on the worker thread.
fn run_export_pipeline(
    root_node: &FileNode,
    project_kind: ProjectKind,
    selection: &SelectionState,
    max_file_size_bytes: u64,
    destination_path: &PathBuf,
    sender: &mpsc::Sender<ExportState>,
) {
    let files_to_bundle = collect_exportable_files(root_node, selection, max_file_size_bytes);
    let total_files = files_to_bundle.len();

    let _ = sender.send(ExportState::Running {
        processed_files: 0,
        total_files,
        current_file: "Building directory tree...".to_string(),
        fraction: 0.05,
    });

    let mut output = format_document_header(root_node, project_kind, selection);

    if !files_to_bundle.is_empty() {
        output.push_str("\n## Source Files\n\n");

        for (index, file_node) in files_to_bundle.iter().enumerate() {
            let processed = index + 1;
            let rel_display = file_node.relative_path.to_string_lossy().replace('\\', "/");

            let _ = sender.send(ExportState::Running {
                processed_files: processed,
                total_files,
                current_file: rel_display,
                fraction: calculate_progress_fraction(processed, total_files),
            });

            if let Ok(content) = fs::read_to_string(&file_node.full_path) {
                output.push_str(&format_code_section(&file_node.relative_path, &content));
                output.push('\n');
            }
        }
    }

    match save_markdown_file(destination_path, &output) {
        Ok(()) => {
            let _ = sender.send(ExportState::Completed {
                saved_path: destination_path.clone(),
                total_files,
            });
        }
        Err(err) => {
            let _ = sender.send(ExportState::Failed(err));
        }
    }
}

/// Formats the Markdown title, metadata bullets, and ASCII directory tree section.
fn format_document_header(
    root_node: &FileNode,
    project_kind: ProjectKind,
    selection: &SelectionState,
) -> String {
    let mut header = String::new();
    header.push_str(&format!("# Project: `{}`\n\n", root_node.name));
    header.push_str(&format!("- **Framework:** {}\n", project_kind.label()));
    header.push_str(&format!(
        "- **Export Mode:** {}\n\n",
        selection.export_mode.label()
    ));
    header.push_str("## Directory Tree\n\n```text\n");
    header.push_str(&format_ascii_tree(root_node));
    header.push_str("\n```\n");
    header
}

/// Recursively collects references to included files under `max_file_size_bytes`.
fn collect_files_recursively<'a>(
    node: &'a FileNode,
    selection: &SelectionState,
    max_file_size_bytes: u64,
    out: &mut Vec<&'a FileNode>,
) {
    for child in &node.children {
        if child.is_dir {
            collect_files_recursively(child, selection, max_file_size_bytes, out);
        } else if selection.is_file_included(child) && child.size_bytes <= max_file_size_bytes {
            out.push(child);
        }
    }
}
