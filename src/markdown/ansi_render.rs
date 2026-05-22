//! Lightweight ANSI parser for the inline code-execution output panel.
//!
//! This is intentionally a small subset of full terminal emulation: we only
//! handle SGR (Select Graphic Rendition) sequences, basic line breaking, and
//! carriage returns. The heavy lifting of ANSI byte parsing is delegated to
//! `vte`, the same crate used by the integrated terminal — see
//! [`crate::terminal`] for the full emulator.
//!
//! Colors map onto [`crate::terminal::AnsiColor`] so that the inline output
//! panel reuses the same palette resolution and theme conventions as the
//! terminal pane (see `terminal/screen.rs::Color::to_egui`).
//!
//! The output is a [`Vec<AnsiLine>`] suitable for rendering via [`render_lines`].

use std::borrow::Cow;

use eframe::egui::{self, Color32, FontId, RichText, Stroke, Ui};

use crate::terminal::AnsiColor;

/// Text attributes parsed from SGR sequences.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SgrAttrs {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub dim: bool,
    pub reverse: bool,
}

/// A styled run of characters within an ANSI line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnsiSegment {
    pub text: String,
    pub fg: AnsiColor,
    pub bg: AnsiColor,
    pub attrs: SgrAttrs,
}

/// One line of ANSI-decoded output.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AnsiLine {
    pub segments: Vec<AnsiSegment>,
}

#[cfg(test)]
impl AnsiLine {
    /// Total visible character count (sum of segment text lengths).
    pub fn char_count(&self) -> usize {
        self.segments.iter().map(|s| s.text.chars().count()).sum()
    }

    /// Concatenate segments into a plain (no-style) string.
    pub fn plain_text(&self) -> String {
        let mut out = String::new();
        for seg in &self.segments {
            out.push_str(&seg.text);
        }
        out
    }
}

/// Convert raw bytes (typically captured stdout/stderr) into styled lines.
///
/// Newlines split lines; carriage returns rewrite the current line from column
/// 0 (matching terminal convention used by progress meters, e.g. `pip install`).
///
/// **Windows CRLF:** Subprocess output often uses `\r\n`. A bare `\r` clears the
/// current line in this parser (for `\r`-overwrite progress text). Sequences
/// `\r\n` are normalized to `\n` **before** parsing so a line like `Hello\r\n`
/// still renders as `Hello` instead of an erased line.
pub fn parse(bytes: &[u8]) -> Vec<AnsiLine> {
    let normalized = normalize_crlf(bytes);
    let mut parser = vte::Parser::new();
    let mut perf = AnsiPerformer::new();
    for &byte in normalized.as_ref() {
        parser.advance(&mut perf, byte);
    }
    perf.finish();
    perf.lines
}

/// Map `\r\n` → `\n` so end-of-line on Windows does not trigger a wiping `\r`.
/// Standalone `\r` is preserved for intentional carriage-return overwrites.
fn normalize_crlf(bytes: &[u8]) -> Cow<'_, [u8]> {
    if !bytes.windows(2).any(|w| w == [b'\r', b'\n']) {
        return Cow::Borrowed(bytes);
    }
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\r' && i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
            out.push(b'\n');
            i += 2;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    Cow::Owned(out)
}

/// Convert a UTF-8 string into styled lines (tests).
#[cfg(test)]
pub fn parse_str(s: &str) -> Vec<AnsiLine> {
    parse(s.as_bytes())
}

/// Render styled lines as monospace text in egui.
///
/// `default_fg` provides the fallback foreground color for unstyled text.
/// `palette` supplies the 16-color ANSI palette; the terminal theme palette is
/// reused so output colors stay consistent across the app.
pub fn render_lines(
    ui: &mut Ui,
    lines: &[AnsiLine],
    font_size: f32,
    default_fg: Color32,
    default_bg: Color32,
    palette: &[Color32; 16],
) {
    if lines.is_empty() {
        return;
    }

    let font = FontId::monospace(font_size);
    for line in lines {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            if line.segments.is_empty() {
                ui.label(RichText::new(" ").font(font.clone()));
                return;
            }
            for seg in &line.segments {
                if seg.text.is_empty() {
                    continue;
                }
                let mut fg = seg.fg.to_egui(true, palette, default_fg, default_bg);
                let mut bg = seg.bg.to_egui(false, palette, default_fg, default_bg);

                if seg.attrs.reverse {
                    std::mem::swap(&mut fg, &mut bg);
                }

                if seg.attrs.dim {
                    fg = blend(fg, default_bg, 0.45);
                }

                let mut text = RichText::new(&seg.text).font(font.clone()).color(fg);
                if seg.attrs.bold {
                    text = text.strong();
                }
                if seg.attrs.italic {
                    text = text.italics();
                }
                if seg.attrs.underline {
                    text = text.underline();
                }
                if seg.attrs.strikethrough {
                    text = text.strikethrough();
                }

                if bg == Color32::TRANSPARENT {
                    ui.label(text);
                } else {
                    egui::Frame::new()
                        .fill(bg)
                        .stroke(Stroke::NONE)
                        .inner_margin(egui::Margin::ZERO)
                        .show(ui, |ui| {
                            ui.label(text);
                        });
                }
            }
        });
    }
}

fn blend(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let lerp = |x: u8, y: u8| ((x as f32) * (1.0 - t) + (y as f32) * t).round() as u8;
    Color32::from_rgba_unmultiplied(
        lerp(a.r(), b.r()),
        lerp(a.g(), b.g()),
        lerp(a.b(), b.b()),
        a.a(),
    )
}

struct AnsiPerformer {
    lines: Vec<AnsiLine>,
    current_line: AnsiLine,
    current_col: usize,
    fg: AnsiColor,
    bg: AnsiColor,
    attrs: SgrAttrs,
}

impl AnsiPerformer {
    fn new() -> Self {
        Self {
            lines: Vec::new(),
            current_line: AnsiLine::default(),
            current_col: 0,
            fg: AnsiColor::Default,
            bg: AnsiColor::Default,
            attrs: SgrAttrs::default(),
        }
    }

    fn finish(&mut self) {
        if !self.current_line.segments.is_empty() || !self.lines.is_empty() {
            self.lines.push(std::mem::take(&mut self.current_line));
        }
    }

    fn append_char(&mut self, ch: char) {
        let last_matches =
            self.current_line.segments.last().is_some_and(|seg| {
                seg.fg == self.fg && seg.bg == self.bg && seg.attrs == self.attrs
            });
        if last_matches {
            if let Some(seg) = self.current_line.segments.last_mut() {
                seg.text.push(ch);
            }
        } else {
            self.current_line.segments.push(AnsiSegment {
                text: ch.to_string(),
                fg: self.fg,
                bg: self.bg,
                attrs: self.attrs,
            });
        }
        self.current_col += 1;
    }

    fn newline(&mut self) {
        self.lines.push(std::mem::take(&mut self.current_line));
        self.current_col = 0;
    }

    fn carriage_return(&mut self) {
        self.current_line.segments.clear();
        self.current_col = 0;
    }

    fn reset_sgr(&mut self) {
        self.fg = AnsiColor::Default;
        self.bg = AnsiColor::Default;
        self.attrs = SgrAttrs::default();
    }

    fn handle_sgr(&mut self, params: &[u16]) {
        if params.is_empty() {
            self.reset_sgr();
            return;
        }
        let mut i = 0;
        while i < params.len() {
            let p = params[i];
            match p {
                0 => self.reset_sgr(),
                1 => self.attrs.bold = true,
                2 => self.attrs.dim = true,
                3 => self.attrs.italic = true,
                4 => self.attrs.underline = true,
                7 => self.attrs.reverse = true,
                9 => self.attrs.strikethrough = true,
                22 => {
                    self.attrs.bold = false;
                    self.attrs.dim = false;
                }
                23 => self.attrs.italic = false,
                24 => self.attrs.underline = false,
                27 => self.attrs.reverse = false,
                29 => self.attrs.strikethrough = false,
                30..=37 => self.fg = AnsiColor::Indexed((p - 30) as u8),
                38 => {
                    if let Some(color) = parse_extended_color(params, &mut i) {
                        self.fg = color;
                        continue;
                    }
                }
                39 => self.fg = AnsiColor::Default,
                40..=47 => self.bg = AnsiColor::Indexed((p - 40) as u8),
                48 => {
                    if let Some(color) = parse_extended_color(params, &mut i) {
                        self.bg = color;
                        continue;
                    }
                }
                49 => self.bg = AnsiColor::Default,
                90..=97 => self.fg = AnsiColor::Indexed((p - 90 + 8) as u8),
                100..=107 => self.bg = AnsiColor::Indexed((p - 100 + 8) as u8),
                _ => {}
            }
            i += 1;
        }
    }
}

/// Consume `params[i+1..]` for the trailing 38/48 sub-sequence, advancing `i`.
/// Returns `Some(color)` when a complete sequence was parsed; otherwise `None`.
fn parse_extended_color(params: &[u16], i: &mut usize) -> Option<AnsiColor> {
    let mode = *params.get(*i + 1)?;
    match mode {
        5 => {
            let idx = *params.get(*i + 2)? as u8;
            *i += 3;
            Some(AnsiColor::Indexed(idx))
        }
        2 => {
            let r = *params.get(*i + 2)? as u8;
            let g = *params.get(*i + 3)? as u8;
            let b = *params.get(*i + 4)? as u8;
            *i += 5;
            Some(AnsiColor::Rgb(r, g, b))
        }
        _ => {
            *i += 1;
            None
        }
    }
}

impl vte::Perform for AnsiPerformer {
    fn print(&mut self, c: char) {
        self.append_char(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => self.newline(),
            b'\r' => self.carriage_return(),
            b'\t' => {
                let to_next = 8 - (self.current_col % 8);
                for _ in 0..to_next {
                    self.append_char(' ');
                }
            }
            0x08 => {
                if let Some(seg) = self.current_line.segments.last_mut() {
                    if seg.text.pop().is_some() && self.current_col > 0 {
                        self.current_col -= 1;
                    }
                    if seg.text.is_empty() {
                        self.current_line.segments.pop();
                    }
                }
            }
            _ => {}
        }
    }

    fn csi_dispatch(
        &mut self,
        params: &vte::Params,
        _intermediates: &[u8],
        _ignore: bool,
        action: char,
    ) {
        if action != 'm' {
            return;
        }
        let mut flat: Vec<u16> = Vec::new();
        for slice in params.iter() {
            for &v in slice {
                flat.push(v);
            }
        }
        self.handle_sgr(&flat);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_plain_lines() {
        let lines = parse_str("hello\nworld");
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].plain_text(), "hello");
        assert_eq!(lines[1].plain_text(), "world");
    }

    #[test]
    fn parses_basic_sgr_red() {
        let lines = parse_str("\x1b[31mfoo\x1b[0m bar");
        assert_eq!(lines.len(), 1);
        let segs = &lines[0].segments;
        assert!(segs.len() >= 2);
        assert_eq!(segs[0].text, "foo");
        assert_eq!(segs[0].fg, AnsiColor::Indexed(1));
        assert_eq!(segs[1].text, " bar");
        assert_eq!(segs[1].fg, AnsiColor::Default);
    }

    #[test]
    fn parses_truecolor() {
        let lines = parse_str("\x1b[38;2;10;20;30mxy");
        let seg = &lines[0].segments[0];
        assert_eq!(seg.fg, AnsiColor::Rgb(10, 20, 30));
        assert_eq!(seg.text, "xy");
    }

    #[test]
    fn parses_256_color() {
        let lines = parse_str("\x1b[38;5;202mhi");
        let seg = &lines[0].segments[0];
        assert_eq!(seg.fg, AnsiColor::Indexed(202));
    }

    #[test]
    fn carriage_return_rewrites_line() {
        let lines = parse_str("first\rover");
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].plain_text(), "over");
    }

    #[test]
    fn crlf_does_not_erase_line() {
        // Windows Python (and many CLIs) end lines with CRLF; `\r` must not
        // clear the line when followed immediately by `\n`.
        let lines = parse_str("Hello\r\n");
        assert!(!lines.is_empty(), "{lines:?}");
        assert_eq!(lines[0].plain_text(), "Hello");
    }

    #[test]
    fn crlf_bytes_match_print_windows() {
        let bytes = b"Hello\r\n";
        let lines = parse(bytes);
        assert!(!lines.is_empty());
        assert_eq!(lines[0].plain_text(), "Hello");
    }

    #[test]
    fn bold_then_reset() {
        let lines = parse_str("\x1b[1mB\x1b[22mn");
        let segs = &lines[0].segments;
        assert!(segs[0].attrs.bold);
        assert!(!segs[1].attrs.bold);
    }

    #[test]
    fn empty_input_yields_no_lines() {
        let lines = parse_str("");
        assert!(lines.is_empty());
    }

    #[test]
    fn trailing_newline_yields_blank_line() {
        let lines = parse_str("a\n");
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].plain_text(), "a");
        assert_eq!(lines[1].plain_text(), "");
    }
}
