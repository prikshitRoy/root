//! Background thread runner and progress streaming for scanning large project directories.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::thread;

use crate::scanner::types::{FileNode, ScanError, ScanOptions};
use crate::scanner::walker::scan_directory_with_progress;

/// Number of entries between progress messages sent to the UI thread to avoid channel contention.
const PROGRESS_BATCH_INTERVAL: usize = 20;

/// Snapshot of the background directory scan state sent to the UI thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScanState {
    /// Directory traversal is actively running.
    Running {
        /// Number of files and folders discovered so far.
        entries_scanned: usize,
        /// Relative path of the entry currently being scanned.
        current_path: String,
    },
    /// Directory scan finished and built the hierarchical `FileNode` tree.
    Completed(FileNode),
    /// Directory scan failed with an error.
    Failed(ScanError),
}

/// Handle held by the UI thread to poll non-blocking updates from a running directory scan.
pub struct ScanJobHandle {
    receiver: Receiver<ScanState>,
}

impl ScanJobHandle {
    /// Drains pending messages from the worker thread and returns the most recent `ScanState`.
    pub fn poll_latest(&self) -> Option<ScanState> {
        let mut latest = None;
        while let Ok(state) = self.receiver.try_recv() {
            latest = Some(state);
        }
        latest
    }
}

/// Spawns a background thread that scans `root_path` and streams progress updates to the UI.
pub fn spawn_scan_job(root_path: PathBuf, options: ScanOptions) -> ScanJobHandle {
    let (sender, receiver) = mpsc::channel();

    thread::spawn(move || {
        let _ = sender.send(ScanState::Running {
            entries_scanned: 0,
            current_path: "Initializing project scan...".to_string(),
        });

        let result =
            scan_directory_with_progress(&root_path, &options, |count, rel_path: &Path| {
                if count == 1 || count % PROGRESS_BATCH_INTERVAL == 0 {
                    let display_path = if rel_path.as_os_str().is_empty() {
                        ".".to_string()
                    } else {
                        rel_path.to_string_lossy().replace('\\', "/")
                    };

                    let _ = sender.send(ScanState::Running {
                        entries_scanned: count,
                        current_path: display_path,
                    });
                }
            });

        match result {
            Ok(tree) => {
                let _ = sender.send(ScanState::Completed(tree));
            }
            Err(err) => {
                let _ = sender.send(ScanState::Failed(err));
            }
        }
    });

    ScanJobHandle { receiver }
}
