//! Public API for generating ASCII project trees and exporting `.md` code bundles.

mod code_bundler;
mod tree_formatter;
mod writer;

#[cfg(test)]
mod tests;

pub use code_bundler::{code_fence_for_content, format_code_section, language_tag_for_path};
pub use tree_formatter::format_ascii_tree;
pub use writer::{build_markdown_document, save_markdown_file};
