//! Terminal widget for egui rendering.
//!
//! This module provides an egui widget that renders the terminal screen
//! buffer and handles keyboard input.

use super::screen::TerminalScreen;
use eframe::egui::{self, Color32, FontId, Key, Modifiers, Rect, Sense, Ui, Vec2};
use std::sync::{Arc, Mutex};

/// Output from the terminal widget.
#[derive(Debug, Default)]
pub struct TerminalWidgetOutput {
    /// Keyboard input to send to the terminal (as bytes)
    pub input: Vec<u8>,
    /// Whether the widget has focus
    pub has_focus: bool,
    /// New size if terminal was resized (cols, rows)
    pub new_size: Option<(u16, u16)>,
}

/// Widget for rendering a terminal in egui.
pub struct TerminalWidget<'a> {
    /// The terminal screen buffer to render
    screen: &'a Arc<Mutex<TerminalScreen>>,
    /// Font size in pixels
    font_size: f32,
    /// Whether the terminal is focused
    focused: bool,
    /// Scroll offset into scrollback (0 = current screen)
    scroll_offset: usize,
    /// Whether dark theme is active
    is_dark: bool,
}

impl<'a> TerminalWidget<'a> {
    /// Create a new terminal widget.
    pub fn new(screen: &'a Arc<Mutex<TerminalScreen>>) -> Self {
        Self {
            screen,
            font_size: 14.0,
            focused: false,
            scroll_offset: 0,
            is_dark: true,
        }
    }

    /// Set the font size.
    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    /// Set whether the terminal is focused.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Set the scroll offset into scrollback.
    pub fn scroll_offset(mut self, offset: usize) -> Self {
        self.scroll_offset = offset;
        self
    }

    /// Set whether dark theme is active.
    pub fn is_dark(mut self, dark: bool) -> Self {
        self.is_dark = dark;
        self
    }

    /// Calculate character dimensions for the monospace font.
    fn char_size(&self, ui: &Ui) -> Vec2 {
        let font_id = FontId::monospace(self.font_size);
        let char_width = ui.fonts(|f| f.glyph_width(&font_id, 'M'));
        let line_height = self.font_size * 1.2;
        Vec2::new(char_width, line_height)
    }

    /// Show the terminal widget and return output.
    pub fn show(self, ui: &mut Ui) -> TerminalWidgetOutput {
        let mut output = TerminalWidgetOutput::default();

        let screen = self.screen.lock().unwrap();
        let (cols, rows) = screen.size();
        let char_size = self.char_size(ui);

        // Calculate desired size
        let desired_size = Vec2::new(
            char_size.x * cols as f32,
            char_size.y * rows as f32,
        );

        // Allocate space and get response
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());
        output.has_focus = response.has_focus() || self.focused;

        // Check if size changed and calculate new terminal dimensions
        let available_size = ui.available_size();
        let new_cols = (available_size.x / char_size.x).floor() as u16;
        let new_rows = (available_size.y / char_size.y).floor() as u16;
        if new_cols > 0 && new_rows > 0 && (new_cols != cols || new_rows != rows) {
            output.new_size = Some((new_cols.max(10), new_rows.max(2)));
        }

        // Handle keyboard input
        if output.has_focus {
            self.handle_keyboard_input(ui, &mut output);
        }

        // Request focus on click
        if response.clicked() {
            response.request_focus();
        }

        // Render the terminal
        self.render_screen(ui, rect, &screen, char_size);

        // Render cursor if focused
        if output.has_focus && screen.cursor_visible() {
            self.render_cursor(ui, rect, &screen, char_size);
        }

        output
    }

    /// Handle keyboard input and convert to terminal bytes.
    fn handle_keyboard_input(&self, ui: &Ui, output: &mut TerminalWidgetOutput) {
        ui.input(|input| {
            // Handle text input
            for event in &input.events {
                match event {
                    egui::Event::Text(text) => {
                        output.input.extend_from_slice(text.as_bytes());
                    }
                    egui::Event::Key {
                        key,
                        pressed: true,
                        modifiers,
                        ..
                    } => {
                        if let Some(bytes) = self.key_to_bytes(*key, *modifiers) {
                            output.input.extend_from_slice(&bytes);
                        }
                    }
                    _ => {}
                }
            }
        });
    }

    /// Convert a key press to terminal escape sequence bytes.
    fn key_to_bytes(&self, key: Key, modifiers: Modifiers) -> Option<Vec<u8>> {
        let ctrl = modifiers.ctrl || modifiers.command;
        let shift = modifiers.shift;
        let alt = modifiers.alt;

        // Control key combinations (Ctrl+A = 0x01, etc.)
        if ctrl && !alt && !shift {
            match key {
                Key::A => return Some(vec![0x01]),
                Key::B => return Some(vec![0x02]),
                Key::C => return Some(vec![0x03]), // SIGINT
                Key::D => return Some(vec![0x04]), // EOF
                Key::E => return Some(vec![0x05]),
                Key::F => return Some(vec![0x06]),
                Key::G => return Some(vec![0x07]),
                Key::H => return Some(vec![0x08]),
                Key::I => return Some(vec![0x09]),
                Key::J => return Some(vec![0x0A]),
                Key::K => return Some(vec![0x0B]),
                Key::L => return Some(vec![0x0C]), // Clear
                Key::M => return Some(vec![0x0D]),
                Key::N => return Some(vec![0x0E]),
                Key::O => return Some(vec![0x0F]),
                Key::P => return Some(vec![0x10]),
                Key::Q => return Some(vec![0x11]),
                Key::R => return Some(vec![0x12]),
                Key::S => return Some(vec![0x13]),
                Key::T => return Some(vec![0x14]),
                Key::U => return Some(vec![0x15]),
                Key::V => return Some(vec![0x16]),
                Key::W => return Some(vec![0x17]),
                Key::X => return Some(vec![0x18]),
                Key::Y => return Some(vec![0x19]),
                Key::Z => return Some(vec![0x1A]), // SIGTSTP
                _ => {}
            }
        }

        // Special keys
        match key {
            Key::Enter => return Some(vec![0x0D]),
            Key::Tab => return Some(vec![0x09]),
            Key::Backspace => return Some(vec![0x7F]),
            Key::Escape => return Some(vec![0x1B]),
            Key::Delete => return Some(b"\x1b[3~".to_vec()),
            Key::Insert => return Some(b"\x1b[2~".to_vec()),
            Key::Home => return Some(b"\x1b[H".to_vec()),
            Key::End => return Some(b"\x1b[F".to_vec()),
            Key::PageUp => return Some(b"\x1b[5~".to_vec()),
            Key::PageDown => return Some(b"\x1b[6~".to_vec()),
            Key::ArrowUp => {
                if shift {
                    return Some(b"\x1b[1;2A".to_vec());
                } else if ctrl {
                    return Some(b"\x1b[1;5A".to_vec());
                } else if alt {
                    return Some(b"\x1b[1;3A".to_vec());
                }
                return Some(b"\x1b[A".to_vec());
            }
            Key::ArrowDown => {
                if shift {
                    return Some(b"\x1b[1;2B".to_vec());
                } else if ctrl {
                    return Some(b"\x1b[1;5B".to_vec());
                } else if alt {
                    return Some(b"\x1b[1;3B".to_vec());
                }
                return Some(b"\x1b[B".to_vec());
            }
            Key::ArrowRight => {
                if shift {
                    return Some(b"\x1b[1;2C".to_vec());
                } else if ctrl {
                    return Some(b"\x1b[1;5C".to_vec());
                } else if alt {
                    return Some(b"\x1b[1;3C".to_vec());
                }
                return Some(b"\x1b[C".to_vec());
            }
            Key::ArrowLeft => {
                if shift {
                    return Some(b"\x1b[1;2D".to_vec());
                } else if ctrl {
                    return Some(b"\x1b[1;5D".to_vec());
                } else if alt {
                    return Some(b"\x1b[1;3D".to_vec());
                }
                return Some(b"\x1b[D".to_vec());
            }
            Key::F1 => return Some(b"\x1bOP".to_vec()),
            Key::F2 => return Some(b"\x1bOQ".to_vec()),
            Key::F3 => return Some(b"\x1bOR".to_vec()),
            Key::F4 => return Some(b"\x1bOS".to_vec()),
            Key::F5 => return Some(b"\x1b[15~".to_vec()),
            Key::F6 => return Some(b"\x1b[17~".to_vec()),
            Key::F7 => return Some(b"\x1b[18~".to_vec()),
            Key::F8 => return Some(b"\x1b[19~".to_vec()),
            Key::F9 => return Some(b"\x1b[20~".to_vec()),
            Key::F10 => return Some(b"\x1b[21~".to_vec()),
            Key::F11 => return Some(b"\x1b[23~".to_vec()),
            Key::F12 => return Some(b"\x1b[24~".to_vec()),
            _ => {}
        }

        None
    }

    /// Render the terminal screen content.
    fn render_screen(
        &self,
        ui: &Ui,
        rect: Rect,
        screen: &TerminalScreen,
        char_size: Vec2,
    ) {
        let painter = ui.painter();

        // Draw background
        let bg_color = if self.is_dark {
            Color32::from_rgb(30, 30, 30)
        } else {
            Color32::from_rgb(255, 255, 255)
        };
        painter.rect_filled(rect, 0.0, bg_color);

        // Get cells to render
        let cells = screen.cells();
        let font_id = FontId::monospace(self.font_size);

        // Render each cell
        for (row_idx, row) in cells.iter().enumerate() {
            let y = rect.top() + (row_idx as f32 * char_size.y);

            for (col_idx, cell) in row.iter().enumerate() {
                let x = rect.left() + (col_idx as f32 * char_size.x);
                let cell_rect = Rect::from_min_size(
                    egui::pos2(x, y),
                    char_size,
                );

                // Draw cell background if not default
                let bg = cell.bg.to_egui(false, self.is_dark);
                if bg != Color32::TRANSPARENT {
                    painter.rect_filled(cell_rect, 0.0, bg);
                }

                // Draw character
                if cell.character != ' ' {
                    let mut fg = cell.fg.to_egui(true, self.is_dark);

                    // Apply attributes
                    if cell.attrs.dim {
                        fg = Color32::from_rgba_unmultiplied(
                            fg.r(),
                            fg.g(),
                            fg.b(),
                            (fg.a() as f32 * 0.5) as u8,
                        );
                    }
                    if cell.attrs.reverse {
                        // Swap fg and bg
                        let temp_bg = cell.bg.to_egui(false, self.is_dark);
                        if temp_bg != Color32::TRANSPARENT {
                            fg = temp_bg;
                        } else {
                            fg = bg_color;
                        }
                        painter.rect_filled(cell_rect, 0.0, cell.fg.to_egui(true, self.is_dark));
                    }
                    if cell.attrs.hidden {
                        fg = bg_color;
                    }

                    // Use bold font variant if bold
                    let font = if cell.attrs.bold {
                        FontId::monospace(self.font_size) // egui doesn't have easy bold monospace
                    } else {
                        font_id.clone()
                    };

                    painter.text(
                        egui::pos2(x, y),
                        egui::Align2::LEFT_TOP,
                        cell.character,
                        font,
                        fg,
                    );

                    // Draw underline
                    if cell.attrs.underline {
                        let underline_y = y + char_size.y - 2.0;
                        painter.line_segment(
                            [
                                egui::pos2(x, underline_y),
                                egui::pos2(x + char_size.x, underline_y),
                            ],
                            egui::Stroke::new(1.0, fg),
                        );
                    }

                    // Draw strikethrough
                    if cell.attrs.strikethrough {
                        let strike_y = y + char_size.y / 2.0;
                        painter.line_segment(
                            [
                                egui::pos2(x, strike_y),
                                egui::pos2(x + char_size.x, strike_y),
                            ],
                            egui::Stroke::new(1.0, fg),
                        );
                    }
                }
            }
        }
    }

    /// Render the cursor.
    fn render_cursor(
        &self,
        ui: &Ui,
        rect: Rect,
        screen: &TerminalScreen,
        char_size: Vec2,
    ) {
        let cursor = screen.cursor();
        let x = rect.left() + (cursor.col as f32 * char_size.x);
        let y = rect.top() + (cursor.row as f32 * char_size.y);

        let cursor_rect = Rect::from_min_size(
            egui::pos2(x, y),
            char_size,
        );

        // Draw block cursor with transparency
        let cursor_color = if self.is_dark {
            Color32::from_rgba_unmultiplied(200, 200, 200, 180)
        } else {
            Color32::from_rgba_unmultiplied(50, 50, 50, 180)
        };

        ui.painter().rect_filled(cursor_rect, 0.0, cursor_color);
    }
}
