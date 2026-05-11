//! Adapter from Ferrite's `ThemeColors` to the colors the PDF renderer uses.
//!
//! By default the PDF exporter renders on a clean white page with dark text
//! (the "print default" theme) regardless of whether the editor is in dark
//! mode, because most users print on white paper. The `use_theme_colors`
//! option in `PdfExportOptions` opts in to using the active editor theme.

use krilla::color::rgb;

use crate::theme::ThemeColors;

/// Colors the PDF renderer needs.
#[derive(Debug, Clone, Copy)]
pub struct PdfTheme {
    /// Optional background fill for the page. None ⇒ leave page white.
    pub background: Option<rgb::Color>,
    /// Body text color.
    pub body: rgb::Color,
    /// Heading text color.
    pub heading: rgb::Color,
    /// Hyperlink color.
    pub link: rgb::Color,
    /// Inline `code` color.
    pub code_inline: rgb::Color,
    /// Background tint behind code blocks (and table headers).
    pub code_block_bg: rgb::Color,
    /// Color for muted UI affordances (HR, blockquote bar, table borders, footers).
    pub muted: rgb::Color,
}

impl PdfTheme {
    /// Print‑friendly defaults: white page, near‑black text, classic blue links.
    pub fn print_default() -> Self {
        Self {
            background: None,
            body: rgb::Color::new(33, 37, 41),
            heading: rgb::Color::new(17, 17, 17),
            link: rgb::Color::new(0, 102, 204),
            code_inline: rgb::Color::new(199, 37, 78),
            code_block_bg: rgb::Color::new(245, 246, 248),
            muted: rgb::Color::new(170, 170, 170),
        }
    }

    /// Build a `PdfTheme` from Ferrite's active `ThemeColors` (used when the
    /// user opts in via `PdfExportOptions::use_theme_colors`).
    pub fn from_theme_colors(colors: &ThemeColors) -> Self {
        Self {
            background: Some(rgb_from_color32(colors.base.background)),
            body: rgb_from_color32(colors.text.primary),
            heading: rgb_from_color32(colors.editor.heading),
            link: rgb_from_color32(colors.text.link),
            code_inline: rgb_from_color32(colors.text.code),
            code_block_bg: rgb_from_color32(colors.editor.code_block_bg),
            muted: rgb_from_color32(colors.base.border),
        }
    }
}

fn rgb_from_color32(c: eframe::egui::Color32) -> rgb::Color {
    rgb::Color::new(c.r(), c.g(), c.b())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_default_has_no_background() {
        let theme = PdfTheme::print_default();
        assert!(theme.background.is_none());
    }
}
