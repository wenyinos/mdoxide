//! File tree sidebar panel for workspace mode.
//!
//! This module provides a collapsible left sidebar that displays
//! the workspace file tree with icons, expand/collapse, and click-to-open.

// Allow dead code - includes panel sizing methods and constants for future
// configurable panel width and drag-to-resize functionality
#![allow(dead_code)]

use crate::ui::docked_sidebar::{self, DockedSidebarEdge};
use crate::ui::icons::{phosphor_font, phosphor_rich_text};
use crate::ui::phosphor_icons::{CARET_DOWN, CARET_RIGHT, FOLDER, X};
use crate::workspaces::{FileTreeNode, FileTreeNodeKind};
use eframe::egui::{self, Color32, RichText, Sense, Ui, Vec2};
use rust_i18n::t;
use std::path::PathBuf;

/// Default width of the file tree panel.
const DEFAULT_PANEL_WIDTH: f32 = 250.0;

/// Minimum width of the file tree panel.
const MIN_PANEL_WIDTH: f32 = 150.0;

/// Maximum width of the file tree panel.
const MAX_PANEL_WIDTH: f32 = 500.0;

/// Height of the workspace name header row (matches ribbon row height).
const WORKSPACE_HEADER_HEIGHT: f32 = 28.0;

/// Indentation per tree level.
const INDENT_PER_LEVEL: f32 = 16.0;

/// Height of each tree item row.
const ROW_HEIGHT: f32 = 22.0;

/// Output from the file tree panel.
#[derive(Debug, Clone, Default)]
pub struct FileTreeOutput {
    /// File that was clicked (should be opened in a tab)
    pub file_clicked: Option<PathBuf>,

    /// Path that was toggled (expand/collapse)
    pub path_toggled: Option<PathBuf>,

    /// Directory that needs its children loaded (lazy loading)
    pub needs_loading: Option<PathBuf>,

    /// Whether close button was clicked
    pub close_requested: bool,

    /// New panel width if resized
    pub new_width: Option<f32>,

    /// Context menu action requested
    pub context_action: Option<FileTreeContextAction>,
}

/// Actions from the file tree context menu.
#[derive(Debug, Clone)]
pub enum FileTreeContextAction {
    /// Create a new file in the selected directory
    NewFile(PathBuf),
    /// Create a new folder in the selected directory
    NewFolder(PathBuf),
    /// Rename the selected item
    Rename(PathBuf),
    /// Delete the selected item
    Delete(PathBuf),
    /// Reveal in system file explorer
    RevealInExplorer(PathBuf),
    /// Refresh the file tree
    Refresh,
}

/// File tree sidebar panel.
pub struct FileTreePanel {
    /// Current panel width
    width: f32,
    /// Whether we're currently resizing
    is_resizing: bool,
}

impl Default for FileTreePanel {
    fn default() -> Self {
        Self::new()
    }
}

impl FileTreePanel {
    /// Create a new file tree panel with default width.
    pub fn new() -> Self {
        Self {
            width: DEFAULT_PANEL_WIDTH,
            is_resizing: false,
        }
    }

    /// Create with a specific width.
    pub fn with_width(mut self, width: f32) -> Self {
        self.width = width.clamp(MIN_PANEL_WIDTH, MAX_PANEL_WIDTH);
        self
    }

    /// Set the panel width.
    pub fn set_width(&mut self, width: f32) {
        self.width = width.clamp(MIN_PANEL_WIDTH, MAX_PANEL_WIDTH);
    }

    /// Get the current panel width.
    pub fn width(&self) -> f32 {
        self.width
    }

    /// Render the file tree panel and return any output.
    ///
    /// # Arguments
    /// * `ctx` - The egui context
    /// * `file_tree` - The file tree root node
    /// * `workspace_name` - Name to display in the panel header
    /// * `is_dark` - Whether dark theme is active
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        file_tree: &FileTreeNode,
        workspace_name: &str,
        is_dark: bool,
    ) -> FileTreeOutput {
        let mut output = FileTreeOutput::default();

        // Panel colors
        let panel_bg = if is_dark {
            Color32::from_rgb(30, 30, 30)
        } else {
            Color32::from_rgb(245, 245, 245)
        };

        let border_color = if is_dark {
            Color32::from_rgb(60, 60, 60)
        } else {
            Color32::from_rgb(200, 200, 200)
        };

        egui::Panel::left("file_tree_panel")
            .resizable(true)
            .default_size(self.width)
            .size_range(MIN_PANEL_WIDTH..=MAX_PANEL_WIDTH)
            .frame(docked_sidebar::frame(panel_bg))
            .show_inside(ui, |ui| {
                // Update width from panel
                let panel_width = ui.available_width();
                let resized = (panel_width - self.width).abs() > 1.0;
                if resized {
                    self.width = panel_width;
                }

                output = self.show_content(ui, file_tree, workspace_name, is_dark);
                if resized {
                    output.new_width = Some(panel_width);
                }

                docked_sidebar::paint_vertical_divider(ui, border_color, DockedSidebarEdge::Left);
            });

        output
    }

    /// Render the tree body without owning a side panel.
    ///
    /// Used by the unified sidebar (MarkText-style tabs), where the container
    /// panel is shared with the outline and search tabs.
    pub fn show_content(
        &mut self,
        ui: &mut egui::Ui,
        file_tree: &FileTreeNode,
        workspace_name: &str,
        is_dark: bool,
    ) -> FileTreeOutput {
        let mut output = FileTreeOutput::default();

        // Header with workspace name and close button
        ui.allocate_ui_with_layout(
            Vec2::new(ui.available_width(), WORKSPACE_HEADER_HEIGHT),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.add_space(4.0);

                ui.label(phosphor_rich_text(FOLDER, 14.0));

                ui.add(
                    egui::Label::new(RichText::new(workspace_name).size(12.0).strong()).truncate(),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(8.0);
                    if ui
                        .add(
                            egui::Button::new(phosphor_rich_text(X, 12.0))
                                .frame(false)
                                .min_size(Vec2::new(20.0, 20.0)),
                        )
                        .on_hover_text(t!("workspace.close_folder").to_string())
                        .clicked()
                    {
                        output.close_requested = true;
                    }
                });
            },
        );

        ui.separator();

        // Scrollable tree area
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add_space(4.0);
                self.render_tree_node(ui, file_tree, 0, is_dark, &mut output);
                ui.add_space(4.0);
            });

        output
    }

    /// Render a single tree node and its children (if expanded).
    fn render_tree_node(
        &self,
        ui: &mut Ui,
        node: &FileTreeNode,
        depth: usize,
        is_dark: bool,
        output: &mut FileTreeOutput,
    ) {
        let indent = depth as f32 * INDENT_PER_LEVEL;

        // Colors
        let text_color = if is_dark {
            Color32::from_rgb(220, 220, 220)
        } else {
            Color32::from_rgb(40, 40, 40)
        };

        let hover_bg = if is_dark {
            Color32::from_rgb(50, 50, 60)
        } else {
            Color32::from_rgb(220, 225, 235)
        };

        let _selected_bg = if is_dark {
            Color32::from_rgb(45, 55, 75)
        } else {
            Color32::from_rgb(200, 210, 230)
        };

        // Determine if this is a directory (loaded or not)
        let is_dir = matches!(
            node.kind,
            FileTreeNodeKind::Directory { .. } | FileTreeNodeKind::DirectoryNotLoaded
        );

        // Check if this directory needs to be loaded
        let needs_loading = matches!(node.kind, FileTreeNodeKind::DirectoryNotLoaded);

        // Calculate row height for consistent sizing
        let row_height = 20.0;

        // Allocate space for the entire row first to detect hover
        let row_width = ui.available_width();
        let (row_rect, row_response) =
            ui.allocate_exact_size(Vec2::new(row_width, row_height), Sense::click());

        // Paint hover background FIRST (before text)
        if row_response.hovered() {
            ui.painter().rect_filled(row_rect, 2.0, hover_bg);
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        // Now render the row content on top of the background
        let mut content_pos = row_rect.left_top() + Vec2::new(indent + 4.0, 2.0);

        // Expand/collapse arrow for directories
        if is_dir {
            let arrow = if node.is_expanded {
                CARET_DOWN
            } else {
                CARET_RIGHT
            };
            ui.painter().text(
                content_pos + Vec2::new(0.0, 0.0),
                egui::Align2::LEFT_TOP,
                arrow,
                phosphor_font(10.0),
                text_color,
            );
        }
        content_pos.x += 14.0; // Space for arrow

        // Icon
        let icon = node.icon();
        ui.painter().text(
            content_pos,
            egui::Align2::LEFT_TOP,
            icon,
            phosphor_font(14.0),
            text_color,
        );
        content_pos.x += 18.0; // Space for icon

        ui.painter().text(
            content_pos,
            egui::Align2::LEFT_TOP,
            &node.name,
            egui::FontId::proportional(12.0),
            text_color,
        );

        // Calculate name width for badge positioning
        let name_galley = ui.fonts_mut(|f| {
            f.layout_no_wrap(
                node.name.clone(),
                egui::FontId::proportional(12.0),
                text_color,
            )
        });
        content_pos.x += name_galley.size().x + 4.0;

        // Handle click
        if row_response.clicked() {
            if is_dir {
                // Toggle expansion for directories
                output.path_toggled = Some(node.path.clone());

                // If expanding and needs loading, signal that
                if !node.is_expanded && needs_loading {
                    output.needs_loading = Some(node.path.clone());
                }
            } else {
                // Open file for files
                output.file_clicked = Some(node.path.clone());
            }
        }

        row_response.clone().on_hover_text(&node.name);

        row_response.context_menu(|ui| {
            self.render_context_menu(ui, node, output);
        });

        // Render children if expanded (only for loaded directories)
        if let FileTreeNodeKind::Directory { children } = &node.kind {
            if node.is_expanded {
                for child in children {
                    self.render_tree_node(ui, child, depth + 1, is_dark, output);
                }
            }
        }
        // Note: DirectoryNotLoaded nodes won't render children until loaded
    }

    /// Render the context menu for a tree node.
    ///
    /// Uses simple Unicode symbols (from COMMON_SYMBOLS in fonts.rs) instead of
    /// emoji characters that may not render correctly in Inter/JetBrains fonts.
    /// Emoji like ✏️ contain variation selectors (U+FE0F) that cause doubled or
    /// square glyph rendering when the font doesn't support the emoji variant.
    fn render_context_menu(&self, ui: &mut Ui, node: &FileTreeNode, output: &mut FileTreeOutput) {
        let is_dir = matches!(
            node.kind,
            FileTreeNodeKind::Directory { .. } | FileTreeNodeKind::DirectoryNotLoaded
        );

        if is_dir {
            // + for "new/add" action (universally understood)
            if ui
                .button(format!("+ {}", t!("workspace.new_file")))
                .clicked()
            {
                output.context_action = Some(FileTreeContextAction::NewFile(node.path.clone()));
                ui.close();
            }
            // ▪ (small filled square) for folder icon
            if ui
                .button(format!("▪ {}", t!("workspace.new_folder")))
                .clicked()
            {
                output.context_action = Some(FileTreeContextAction::NewFolder(node.path.clone()));
                ui.close();
            }
            ui.separator();
        }

        // Text-only for rename (no simple icon fits well)
        if ui.button(t!("workspace.rename").to_string()).clicked() {
            output.context_action = Some(FileTreeContextAction::Rename(node.path.clone()));
            ui.close();
        }

        // ✕ (multiplication x) for delete action - in COMMON_SYMBOLS
        if ui
            .button(format!("{} {}", X, t!("workspace.delete")))
            .clicked()
        {
            output.context_action = Some(FileTreeContextAction::Delete(node.path.clone()));
            ui.close();
        }

        ui.separator();

        // → (right arrow) for "reveal/navigate" action - in COMMON_SYMBOLS
        if ui
            .button(format!("→ {}", t!("tab.reveal_in_explorer")))
            .clicked()
        {
            output.context_action =
                Some(FileTreeContextAction::RevealInExplorer(node.path.clone()));
            ui.close();
        }

        ui.separator();

        // ↻ for refresh - simple clockwise arrow (not in COMMON_SYMBOLS but widely supported)
        // Fallback: text-only if this still causes issues
        if ui
            .button(format!("↻ {}", t!("workspace.refresh")))
            .clicked()
        {
            output.context_action = Some(FileTreeContextAction::Refresh);
            ui.close();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_tree_panel_new() {
        let panel = FileTreePanel::new();
        assert_eq!(panel.width(), DEFAULT_PANEL_WIDTH);
    }

    #[test]
    fn test_file_tree_panel_with_width() {
        let panel = FileTreePanel::new().with_width(300.0);
        assert_eq!(panel.width(), 300.0);
    }

    #[test]
    fn test_file_tree_panel_width_clamping() {
        let panel = FileTreePanel::new().with_width(50.0); // Below min
        assert_eq!(panel.width(), MIN_PANEL_WIDTH);

        let panel = FileTreePanel::new().with_width(1000.0); // Above max
        assert_eq!(panel.width(), MAX_PANEL_WIDTH);
    }

    #[test]
    fn test_file_tree_output_default() {
        let output = FileTreeOutput::default();
        assert!(output.file_clicked.is_none());
        assert!(output.path_toggled.is_none());
        assert!(!output.close_requested);
        assert!(output.new_width.is_none());
        assert!(output.context_action.is_none());
    }
}
