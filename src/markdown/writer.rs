//! Document assembly and filesystem export for generated Markdown bundles.

use std::fs;
use std::path::Path;

use crate::detector::ProjectKind;
use crate::markdown::code_bundler::format_code_section;
use crate::markdown::tree_formatter::format_ascii_tree;
use crate::scanner::FileNode;
use crate::selection::{ExportMode, SelectionState};

/// Builds the complete `.md` document for `root_node` based on `selection` and `max_file_size_bytes`.
pub fn build_markdown_document(
    root_node: &FileNode,
    project_kind: ProjectKind,
    selection: &SelectionState,
    max_file_size_bytes: u64,
) -> String {
    let mut output = String::new();

    // 1. Document header and project metadata
    output.push_str(&format!("# Project: `{}`\n\n", root_node.name));
    output.push_str(&format!("- **Framework:** {}\n", project_kind.label()));
    output.push_str(&format!(
        "- **Export Mode:** {}\n\n",
        selection.export_mode.label()
    ));

    // 2. Directory tree section
    output.push_str("## Directory Tree\n\n```text\n");
    output.push_str(&format_ascii_tree(root_node));
    output.push_str("\n```\n");

    // 3. Optional source code sections
    if selection.export_mode != ExportMode::TreeOnly {
        let mut code_sections = Vec::new();
        collect_code_sections(
            root_node,
            selection,
            max_file_size_bytes,
            &mut code_sections,
        );

        if !code_sections.is_empty() {
            output.push_str("\n## Source Files\n\n");
            for section in code_sections {
                output.push_str(&section);
                output.push('\n');
            }
        }
    }

    output
}

/// Writes the generated Markdown string to `destination_path` on disk.
pub fn save_markdown_file(destination_path: &Path, markdown_content: &str) -> Result<(), String> {
    fs::write(destination_path, markdown_content).map_err(|err| {
        format!(
            "Failed to write Markdown file to '{}': {}",
            destination_path.display(),
            err
        )
    })
}

/// Recursively reads included files under `max_file_size_bytes` and formats their code blocks.
fn collect_code_sections(
    node: &FileNode,
    selection: &SelectionState,
    max_file_size_bytes: u64,
    sections: &mut Vec<String>,
) {
    for child in &node.children {
        if child.is_dir {
            collect_code_sections(child, selection, max_file_size_bytes, sections);
        } else if selection.is_file_included(child) && child.size_bytes <= max_file_size_bytes {
            if let Ok(content) = fs::read_to_string(&child.full_path) {
                sections.push(format_code_section(&child.relative_path, &content));
            }
        }
    }
}
