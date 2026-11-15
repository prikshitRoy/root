//! Pure functions for wrapping source files in syntax-tagged Markdown code blocks.

use std::path::Path;

/// Minimum number of backticks used for a fenced Markdown code block.
const MIN_FENCE_BACKTICKS: usize = 3;

/// Formats a single file's relative path and UTF-8 content into a Markdown section.
pub fn format_code_section(relative_path: &Path, content: &str) -> String {
    // Normalize Windows backslashes to forward slashes for clean cross-platform Markdown headers
    let normalized_path = relative_path.to_string_lossy().replace('\\', "/");
    let language_tag = language_tag_for_path(relative_path);
    let fence = code_fence_for_content(content);
    let trimmed_content = content.trim_end_matches(['\r', '\n']);

    format!(
        "### `{normalized_path}`\n\n{fence}{language_tag}\n{trimmed_content}\n{fence}\n"
    )
}

/// Computes a backtick fence strictly longer than any backtick run inside `content`.
pub fn code_fence_for_content(content: &str) -> String {
    let mut max_run = 0;
    let mut current_run = 0;

    for ch in content.chars() {
        if ch == '`' {
            current_run += 1;
            if current_run > max_run {
                max_run = current_run;
            }
        } else {
            current_run = 0;
        }
    }

    let fence_len = MIN_FENCE_BACKTICKS.max(max_run + 1);
    "`".repeat(fence_len)
}

/// Maps a file path's extension or filename to a Markdown syntax-highlighting identifier.
pub fn language_tag_for_path(path: &Path) -> &'static str {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    match ext.as_str() {
        "rs" => "rust",
        "ts" | "mts" | "cts" => "ts",
        "tsx" => "tsx",
        "js" | "mjs" | "cjs" => "js",
        "jsx" => "jsx",
        "json" | "jsonc" => "json",
        "toml" => "toml",
        "yaml" | "yml" => "yaml",
        "md" | "mdx" => "markdown",
        "html" | "htm" => "html",
        "css" | "scss" => "css",
        "py" => "python",
        "sh" | "bash" | "zsh" => "bash",
        "sql" => "sql",
        "c" | "h" => "c",
        "cpp" | "hpp" | "cc" => "cpp",
        "go" => "go",
        _ => "text",
    }
}
