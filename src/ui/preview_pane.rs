//! Right-hand side panel for previewing the generated Markdown document before saving.

use eframe::egui;

use crate::detector::ProjectKind;
use crate::markdown::build_markdown_document;
use crate::scanner::FileNode;
use crate::selection::SelectionState;

/// Default width in logical pixels for the Markdown preview side panel.
const DEFAULT_PREVIEW_WIDTH: f32 = 420.0;

/// Renders a resizable right-hand panel showing the generated `.md` text for `root_node`.
pub fn render_markdown_preview_panel(
    ctx: &egui::Context,
    root_node: &FileNode,
    project_kind: ProjectKind,
    selection: &SelectionState,
    max_file_size_bytes: u64,
) {
    egui::SidePanel::right("markdown_preview_panel")
        .resizable(true)
        .default_width(DEFAULT_PREVIEW_WIDTH)
        .show(ctx, |ui| {
            ui.heading("Markdown Preview");
            ui.small("Live preview of the .md file that will be exported.");
            ui.separator();

            let preview_markdown = build_markdown_document(
                root_node,
                project_kind,
                selection,
                max_file_size_bytes,
            );

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut preview_markdown.as_str())
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY),
                    );
                });
        });
}
