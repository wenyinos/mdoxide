//! Text rendering for FerriteEditor.

use egui::{Color32, FontId, Pos2};
use std::sync::Arc;

use super::super::line_cache::LineCache;

/// Renders a line of text using the cache.
pub fn render_line(
    painter: &egui::Painter,
    line_cache: &mut LineCache,
    line_content: &str,
    x: f32,
    y: f32,
    font_id: FontId,
    text_color: Color32,
) -> Arc<egui::Galley> {
    // Strip trailing newline for display
    let display_content = line_content.trim_end_matches(['\r', '\n']);

    // Get or create galley from cache
    let galley = line_cache.get_galley(display_content, painter, font_id, text_color);

    // Draw the galley
    painter.galley(Pos2::new(x, y), Arc::clone(&galley), text_color);

    galley
}
