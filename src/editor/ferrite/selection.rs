//! Selection rendering and utilities for FerriteEditor.
//!
//! This module contains:
//! - Selection rendering (`render_selection`, `render_line_selection`)
//! - Word boundary detection (`find_word_boundaries`)
//! - Select all functionality (`select_all`)

use egui::{Color32, FontId, Pos2, Rect, Vec2};

use super::cursor::{Cursor, Selection};
use super::editor::FerriteEditor;

impl FerriteEditor {
    /// Finds word boundaries around a cursor position.
    /// Returns (word_start, word_end) cursors.
    pub(crate) fn find_word_boundaries(&self, cursor: Cursor) -> (Cursor, Cursor) {
        if let Some(line_content) = self.buffer.get_line(cursor.line) {
            let line = line_content.trim_end_matches(['\r', '\n']);
            let chars: Vec<char> = line.chars().collect();
            let col = cursor.column.min(chars.len());

            if chars.is_empty() {
                return (cursor, cursor);
            }

            // Classify the character at cursor position
            let char_at_cursor = if col < chars.len() {
                chars[col]
            } else if col > 0 {
                chars[col - 1]
            } else {
                return (cursor, cursor);
            };

            let is_word_char = |c: char| c.is_alphanumeric() || c == '_';
            let char_class = if is_word_char(char_at_cursor) {
                0 // word char
            } else if char_at_cursor.is_whitespace() {
                1 // whitespace
            } else {
                2 // punctuation
            };

            // Find start of word
            let mut start_col = col;
            while start_col > 0 {
                let prev_char = chars[start_col - 1];
                let prev_class = if is_word_char(prev_char) {
                    0
                } else if prev_char.is_whitespace() {
                    1
                } else {
                    2
                };
                if prev_class != char_class {
                    break;
                }
                start_col -= 1;
            }

            // Find end of word
            let mut end_col = col;
            while end_col < chars.len() {
                let curr_char = chars[end_col];
                let curr_class = if is_word_char(curr_char) {
                    0
                } else if curr_char.is_whitespace() {
                    1
                } else {
                    2
                };
                if curr_class != char_class {
                    break;
                }
                end_col += 1;
            }

            (
                Cursor::new(cursor.line, start_col),
                Cursor::new(cursor.line, end_col),
            )
        } else {
            (cursor, cursor)
        }
    }

    /// Selects all text in the buffer, clearing extra cursors.
    pub fn select_all(&mut self) {
        let start = Cursor::start();
        let last_line = self.buffer.line_count().saturating_sub(1);
        let last_col = self
            .buffer
            .get_line(last_line)
            .map(|l| l.trim_end_matches(['\r', '\n']).chars().count())
            .unwrap_or(0);
        let end = Cursor::new(last_line, last_col);
        self.set_selection(Selection::new(start, end));
    }

    /// Renders selection backgrounds for all selections (multi-cursor support).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render_all_selections(
        &self,
        painter: &egui::Painter,
        rect: Rect,
        text_start_x: f32,
        font_id: &FontId,
        wrap_width: f32,
        start_line: usize,
        end_line: usize,
        selection_color: Color32,
    ) {
        for selection in &self.selections {
            if selection.is_range() {
                self.render_single_selection(
                    painter,
                    rect,
                    text_start_x,
                    font_id,
                    wrap_width,
                    start_line,
                    end_line,
                    selection_color,
                    selection,
                );
            }
        }
    }

    /// Renders selection background for a single selection across visible lines.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render_single_selection(
        &self,
        painter: &egui::Painter,
        rect: Rect,
        text_start_x: f32,
        font_id: &FontId,
        wrap_width: f32,
        start_line: usize,
        end_line: usize,
        selection_color: Color32,
        selection: &Selection,
    ) {
        let (sel_start, sel_end) = selection.ordered();

        // Only render if selection intersects visible range
        if sel_end.line < start_line || sel_start.line >= end_line {
            return;
        }

        for line_idx in start_line..end_line {
            if !selection.touches_line(line_idx) {
                continue;
            }

            // Calculate line Y position
            let line_y = if self.wrap_enabled && !self.view.wrap_info().is_empty() {
                rect.min.y + self.view.get_line_y_offset(line_idx)
                    - self.view.get_line_y_offset(self.view.first_visible_line())
                    - self.view.scroll_offset_y()
            } else {
                rect.min.y + self.view.line_to_pixel(line_idx)
            };

            if let Some(line_content) = self.buffer.get_line(line_idx) {
                let line = line_content.trim_end_matches(['\r', '\n']);
                let line_len = line.chars().count();

                // Determine selection range on this line
                let line_sel_start = if line_idx == sel_start.line {
                    sel_start.column
                } else {
                    0
                };

                let line_sel_end = if line_idx == sel_end.line {
                    sel_end.column
                } else {
                    // Select to end of line (include newline indicator)
                    line_len + 1
                };

                // Clamp to valid range
                let line_sel_start = line_sel_start.min(line_len);
                let line_sel_end = line_sel_end.min(line_len + 1);

                if line_sel_start >= line_sel_end && line_idx != sel_end.line {
                    continue;
                }

                // Render selection rectangles for this line
                self.render_line_selection(
                    painter,
                    line,
                    line_idx,
                    line_y,
                    text_start_x,
                    font_id,
                    wrap_width,
                    line_sel_start,
                    line_sel_end,
                    selection_color,
                    rect.max.x,
                );
            }
        }
    }

    /// Renders selection background for a single line.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render_line_selection(
        &self,
        painter: &egui::Painter,
        line: &str,
        _line_idx: usize,
        line_y: f32,
        text_start_x: f32,
        font_id: &FontId,
        wrap_width: f32,
        sel_start: usize,
        sel_end: usize,
        selection_color: Color32,
        max_x: f32,
    ) {
        let chars: Vec<char> = line.chars().collect();
        let line_len = chars.len();

        if self.wrap_enabled && wrap_width > 0.0 {
            // For wrapped text, use galley to get selection rectangles
            let galley = painter.layout(
                line.to_string(),
                font_id.clone(),
                Color32::WHITE,
                wrap_width,
            );

            // Get cursor positions for start and end
            let start_ccursor = egui::text::CCursor::new(sel_start.min(line_len));
            let end_ccursor = egui::text::CCursor::new(sel_end.min(line_len));

            let start_cursor = galley.from_ccursor(start_ccursor);
            let end_cursor = galley.from_ccursor(end_ccursor);

            // If on the same row, draw a single rectangle
            if start_cursor.rcursor.row == end_cursor.rcursor.row {
                let start_rect = galley.pos_from_cursor(&start_cursor);
                let end_rect = galley.pos_from_cursor(&end_cursor);

                let sel_rect = Rect::from_min_max(
                    Pos2::new(text_start_x + start_rect.min.x, line_y + start_rect.min.y),
                    Pos2::new(text_start_x + end_rect.min.x, line_y + end_rect.max.y),
                );
                painter.rect_filled(sel_rect, 0.0, selection_color);
            } else {
                // Selection spans multiple visual rows - draw each row
                for row_idx in start_cursor.rcursor.row..=end_cursor.rcursor.row {
                    if row_idx >= galley.rows.len() {
                        break;
                    }

                    let row = &galley.rows[row_idx];
                    let row_y = line_y + row.rect.min.y;
                    let row_height = row.rect.height();

                    let row_start_x = if row_idx == start_cursor.rcursor.row {
                        let pos = galley.pos_from_cursor(&start_cursor);
                        text_start_x + pos.min.x
                    } else {
                        text_start_x
                    };

                    let row_end_x = if row_idx == end_cursor.rcursor.row {
                        let pos = galley.pos_from_cursor(&end_cursor);
                        text_start_x + pos.min.x
                    } else {
                        text_start_x + row.rect.width()
                    };

                    let sel_rect = Rect::from_min_max(
                        Pos2::new(row_start_x, row_y),
                        Pos2::new(row_end_x, row_y + row_height),
                    );
                    painter.rect_filled(sel_rect, 0.0, selection_color);
                }
            }

            // If selecting past end of line, add a small indicator
            if sel_end > line_len {
                let end_cursor = galley.from_ccursor(egui::text::CCursor::new(line_len));
                let end_rect = galley.pos_from_cursor(&end_cursor);
                let newline_rect = Rect::from_min_size(
                    Pos2::new(text_start_x + end_rect.min.x, line_y + end_rect.min.y),
                    Vec2::new(8.0, end_rect.height()), // Small rectangle for newline
                );
                painter.rect_filled(newline_rect, 0.0, selection_color);
            }
        } else {
            // Non-wrapped mode: rectangle calculation
            // Use HarfRust shaped advances for complex-script text
            let use_shaped = crate::fonts::needs_complex_script_fonts(line);
            let shaped_data = if use_shaped {
                let font_bytes = crate::fonts::ttf_bytes_for_font_id_shaping(font_id);
                super::shaping::shape_text(line, font_bytes, font_id.size)
                    .ok()
                    .filter(|g| !g.is_empty())
                    .map(|g| super::shaping::group_clusters(&g, line.len()))
            } else {
                None
            };

            let measure_x = |col: usize| -> f32 {
                if col == 0 {
                    return text_start_x - self.view.horizontal_scroll();
                }
                if let Some(ref clusters) = shaped_data {
                    let x = super::shaping::column_to_x_offset(line, clusters, col);
                    text_start_x + x - self.view.horizontal_scroll()
                } else {
                    let prefix: String = chars.iter().take(col).collect();
                    let galley = painter.layout_no_wrap(prefix, font_id.clone(), Color32::WHITE);
                    text_start_x + galley.size().x - self.view.horizontal_scroll()
                }
            };

            let start_x = measure_x(sel_start);

            let end_x = if sel_end >= line_len {
                let text_end = measure_x(line_len);
                if sel_end > line_len {
                    (text_end + 8.0).min(max_x)
                } else {
                    text_end
                }
            } else {
                measure_x(sel_end)
            };

            let sel_rect = Rect::from_min_max(
                Pos2::new(start_x, line_y),
                Pos2::new(end_x, line_y + self.view.line_height()),
            );
            painter.rect_filled(sel_rect, 0.0, selection_color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_all() {
        let mut editor = FerriteEditor::from_string("Hello\nWorld");

        editor.select_all();
        assert!(editor.has_selection());
        assert_eq!(editor.selection().start_pos(), Cursor::new(0, 0));
        assert_eq!(editor.selection().end_pos(), Cursor::new(1, 5));
        assert_eq!(editor.selected_text(), "Hello\nWorld");
    }

    #[test]
    fn test_find_word_boundaries_word() {
        let editor = FerriteEditor::from_string("hello world test");

        // Cursor in middle of "world"
        let (start, end) = editor.find_word_boundaries(Cursor::new(0, 7));
        assert_eq!(start.column, 6);
        assert_eq!(end.column, 11);
    }

    #[test]
    fn test_find_word_boundaries_start() {
        let editor = FerriteEditor::from_string("hello world");

        // Cursor at start of "hello"
        let (start, end) = editor.find_word_boundaries(Cursor::new(0, 0));
        assert_eq!(start.column, 0);
        assert_eq!(end.column, 5);
    }

    #[test]
    fn test_find_word_boundaries_whitespace() {
        let editor = FerriteEditor::from_string("hello   world");

        // Cursor on whitespace between words
        let (start, end) = editor.find_word_boundaries(Cursor::new(0, 6));
        assert_eq!(start.column, 5);
        assert_eq!(end.column, 8);
    }

    #[test]
    fn test_find_word_boundaries_punctuation() {
        let editor = FerriteEditor::from_string("hello, world");

        // Cursor on comma
        let (start, end) = editor.find_word_boundaries(Cursor::new(0, 5));
        assert_eq!(start.column, 5);
        assert_eq!(end.column, 6);
    }

    #[test]
    fn test_find_word_boundaries_empty_line() {
        let editor = FerriteEditor::from_string("");

        let (start, end) = editor.find_word_boundaries(Cursor::new(0, 0));
        assert_eq!(start.column, 0);
        assert_eq!(end.column, 0);
    }
}
