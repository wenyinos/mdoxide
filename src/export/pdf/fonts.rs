//! Bundled fonts for the PDF exporter.
//!
//! We re-use the same Inter + JetBrains Mono font bytes that are already
//! `include_bytes!`-ed by `src/fonts.rs`. krilla subsets these on export,
//! so the on-disk PDF only contains the glyphs that are actually used.

use krilla::text::Font;
use std::sync::Arc;

// Bundled Inter (proportional)
const INTER_REGULAR: &[u8] = include_bytes!("../../../assets/fonts/Inter-Regular.ttf");
const INTER_BOLD: &[u8] = include_bytes!("../../../assets/fonts/Inter-Bold.ttf");
const INTER_ITALIC: &[u8] = include_bytes!("../../../assets/fonts/Inter-Italic.ttf");
const INTER_BOLD_ITALIC: &[u8] = include_bytes!("../../../assets/fonts/Inter-BoldItalic.ttf");

// Bundled JetBrains Mono (monospace, used for code blocks and inline code)
const JETBRAINS_REGULAR: &[u8] = include_bytes!("../../../assets/fonts/JetBrainsMono-Regular.ttf");
const JETBRAINS_BOLD: &[u8] = include_bytes!("../../../assets/fonts/JetBrainsMono-Bold.ttf");
const JETBRAINS_ITALIC: &[u8] = include_bytes!("../../../assets/fonts/JetBrainsMono-Italic.ttf");
const JETBRAINS_BOLD_ITALIC: &[u8] =
    include_bytes!("../../../assets/fonts/JetBrainsMono-BoldItalic.ttf");

/// Logical text style needed by the markdown renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FontStyle {
    Regular,
    Bold,
    Italic,
    BoldItalic,
}

/// All four variants of a single font family (regular, bold, italic, bold-italic).
#[derive(Clone)]
pub(crate) struct FontFamily {
    pub regular: Font,
    pub bold: Font,
    pub italic: Font,
    pub bold_italic: Font,
}

impl FontFamily {
    pub fn pick(&self, style: FontStyle) -> Font {
        match style {
            FontStyle::Regular => self.regular.clone(),
            FontStyle::Bold => self.bold.clone(),
            FontStyle::Italic => self.italic.clone(),
            FontStyle::BoldItalic => self.bold_italic.clone(),
        }
    }
}

/// Fonts available to the PDF exporter.
#[derive(Clone)]
pub(crate) struct PdfFonts {
    pub body: FontFamily,
    pub mono: FontFamily,
}

/// Errors produced while loading a bundled font into krilla.
#[derive(Debug)]
pub enum FontLoadError {
    /// krilla rejected the embedded TTF bytes.
    Invalid(&'static str),
}

impl std::fmt::Display for FontLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FontLoadError::Invalid(name) => {
                write!(f, "Failed to load bundled PDF font: {}", name)
            }
        }
    }
}

impl std::error::Error for FontLoadError {}

/// Build a single `krilla::text::Font` from a `&'static` byte slice.
fn make_font(bytes: &'static [u8], name: &'static str) -> Result<Font, FontLoadError> {
    // Wrap in Arc<Vec<u8>> so krilla can take ownership of the data without copying it.
    let data: Arc<Vec<u8>> = Arc::new(bytes.to_vec());
    Font::new(data.into(), 0).ok_or(FontLoadError::Invalid(name))
}

/// Load all four bundled font families into krilla `Font` handles.
///
/// This allocates roughly 5–7 MB of font data per export (one shared copy per
/// face), which is dropped as soon as `document.finish()` returns. We do not
/// cache it across calls: PDF export is a user‑initiated, infrequent action.
pub(crate) fn load_bundled_fonts() -> Result<PdfFonts, FontLoadError> {
    Ok(PdfFonts {
        body: FontFamily {
            regular: make_font(INTER_REGULAR, "Inter-Regular")?,
            bold: make_font(INTER_BOLD, "Inter-Bold")?,
            italic: make_font(INTER_ITALIC, "Inter-Italic")?,
            bold_italic: make_font(INTER_BOLD_ITALIC, "Inter-BoldItalic")?,
        },
        mono: FontFamily {
            regular: make_font(JETBRAINS_REGULAR, "JetBrainsMono-Regular")?,
            bold: make_font(JETBRAINS_BOLD, "JetBrainsMono-Bold")?,
            italic: make_font(JETBRAINS_ITALIC, "JetBrainsMono-Italic")?,
            bold_italic: make_font(JETBRAINS_BOLD_ITALIC, "JetBrainsMono-BoldItalic")?,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_fonts_load() {
        let fonts = load_bundled_fonts().expect("bundled fonts must load");
        // Sanity check via units_per_em.
        assert!(fonts.body.regular.units_per_em() > 0.0);
        assert!(fonts.mono.regular.units_per_em() > 0.0);
    }
}
