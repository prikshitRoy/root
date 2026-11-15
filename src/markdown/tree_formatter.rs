//! Pure functions for rendering a `FileNode` hierarchy as a box-drawing ASCII/Unicode tree.

use crate::scanner::FileNode;

/// Connector prefix for a tree node that has siblings below it.
const BRANCH_MIDDLE: &str = "├── ";

/// Connector prefix for the final child node in a directory.
const BRANCH_LAST: &str = "└── ";

/// Vertical continuation prefix when an ancestor has remaining siblings.
const INDENT_ACTIVE: &str = "│   ";

/// Blank indentation prefix when an ancestor was the last child.
const INDENT_EMPTY: &str = "    ";

/// Formats `root_node` and all of its children into a Unicode box-drawing directory tree.
pub fn format_ascii_tree(root_node: &FileNode) -> String {
    let mut lines = Vec::new();
    lines.push(format!("{}/", root_node.name));

    let child_count = root_node.children.len();
    for (index, child) in root_node.children.iter().enumerate() {
        let is_last = index + 1 == child_count;
        append_tree_lines(child, "", is_last, &mut lines);
    }

    lines.join("\n")
}

/// Recursively appends formatted lines for `node` and its descendants.
fn append_tree_lines(
    node: &FileNode,
    prefix: &str,
    is_last: bool,
    lines: &mut Vec<String>,
) {
    let connector = if is_last { BRANCH_LAST } else { BRANCH_MIDDLE };
    let trailing_slash = if node.is_dir { "/" } else { "" };
    lines.push(format!("{prefix}{connector}{}{trailing_slash}", node.name));

    if !node.is_dir {
        return;
    }

    let child_prefix = format!(
        "{prefix}{}",
        if is_last { INDENT_EMPTY } else { INDENT_ACTIVE }
    );
    let child_count = node.children.len();
    for (index, child) in node.children.iter().enumerate() {
        let is_last_child = index + 1 == child_count;
        append_tree_lines(child, &child_prefix, is_last_child, lines);
    }
}