//! Configuration types for the PDF exporter.
//!
//! See `docs/technical/planning/pdf-export-pipeline.md` for the design rationale.

use serde::{Deserialize, Serialize};

/// Standard page size, in PDF points (1 pt = 1/72 inch).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PdfPageSize {
    /// 595 × 842 pt (210 × 297 mm)
    A4,
    /// 612 × 792 pt (8.5 × 11 in)
    UsLetter,
    /// 612 × 1008 pt (8.5 × 14 in)
    UsLegal,
    /// 420 × 595 pt (148 × 210 mm)
    A5,
}

impl PdfPageSize {
    /// Page width in PDF points.
    pub fn width(self) -> f32 {
        match self {
            PdfPageSize::A4 => 595.0,
            PdfPageSize::UsLetter => 612.0,
            PdfPageSize::UsLegal => 612.0,
            PdfPageSize::A5 => 420.0,
        }
    }

    /// Page height in PDF points.
    pub fn height(self) -> f32 {
        match self {
            PdfPageSize::A4 => 842.0,
            PdfPageSize::UsLetter => 792.0,
            PdfPageSize::UsLegal => 1008.0,
            PdfPageSize::A5 => 595.0,
        }
    }

    /// Human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            PdfPageSize::A4 => "A4",
            PdfPageSize::UsLetter => "US Letter",
            PdfPageSize::UsLegal => "US Legal",
            PdfPageSize::A5 => "A5",
        }
    }

    /// All variants in display order.
    pub fn all() -> &'static [PdfPageSize] {
        &[
            PdfPageSize::A4,
            PdfPageSize::UsLetter,
            PdfPageSize::UsLegal,
            PdfPageSize::A5,
        ]
    }

    /// Default page size derived from the system locale: US locales get Letter,
    /// everything else gets A4.
    pub fn locale_default() -> Self {
        if let Some(loc) = sys_locale::get_locale() {
            let lower = loc.to_ascii_lowercase();
            if lower.ends_with("-us") || lower == "en-us" {
                return PdfPageSize::UsLetter;
            }
        }
        PdfPageSize::A4
    }
}

impl Default for PdfPageSize {
    fn default() -> Self {
        Self::locale_default()
    }
}

/// Page margins in PDF points. All four sides are independently configurable
/// so power users can tune the layout.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PdfMargins {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl PdfMargins {
    /// Comfortable default margins (~0.75 in / ~19 mm on every side).
    pub fn comfortable() -> Self {
        Self {
            top: 54.0,
            right: 54.0,
            bottom: 54.0,
            left: 54.0,
        }
    }

    /// Narrow margins (~0.4 in / ~10 mm) for dense content.
    pub fn narrow() -> Self {
        Self {
            top: 28.0,
            right: 28.0,
            bottom: 28.0,
            left: 28.0,
        }
    }

    /// Wide margins (~1 in / ~25 mm) for printed material.
    pub fn wide() -> Self {
        Self {
            top: 72.0,
            right: 72.0,
            bottom: 72.0,
            left: 72.0,
        }
    }
}

impl Default for PdfMargins {
    fn default() -> Self {
        Self::comfortable()
    }
}

/// Named margin preset used by the export dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PdfMarginPreset {
    Narrow,
    Comfortable,
    Wide,
    Custom,
}

impl PdfMarginPreset {
    pub fn label(self) -> &'static str {
        match self {
            PdfMarginPreset::Narrow => "Narrow (10 mm)",
            PdfMarginPreset::Comfortable => "Comfortable (19 mm)",
            PdfMarginPreset::Wide => "Wide (25 mm)",
            PdfMarginPreset::Custom => "Custom…",
        }
    }

    pub fn margins(self, custom: PdfMargins) -> PdfMargins {
        match self {
            PdfMarginPreset::Narrow => PdfMargins::narrow(),
            PdfMarginPreset::Comfortable => PdfMargins::comfortable(),
            PdfMarginPreset::Wide => PdfMargins::wide(),
            PdfMarginPreset::Custom => custom,
        }
    }

    pub fn all() -> &'static [PdfMarginPreset] {
        &[
            PdfMarginPreset::Narrow,
            PdfMarginPreset::Comfortable,
            PdfMarginPreset::Wide,
            PdfMarginPreset::Custom,
        ]
    }
}

impl Default for PdfMarginPreset {
    fn default() -> Self {
        Self::Comfortable
    }
}

/// Persisted PDF export options. Wired into `Settings` and surfaced through
/// the export dialog.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PdfExportOptions {
    pub page_size: PdfPageSize,
    pub margin_preset: PdfMarginPreset,
    /// Custom margins, only used when `margin_preset == Custom`.
    pub custom_margins: PdfMargins,
    /// If true, every H1 (after the first) starts on a new page.
    pub page_break_before_h1: bool,
    /// If true, apply the active theme's colors to the exported PDF.
    /// Off by default because most users print on white paper.
    pub use_theme_colors: bool,
    /// If true, include a page footer with `Page N of M`.
    pub include_page_numbers: bool,
    /// If true, open the resulting file with the OS default viewer after export.
    pub open_after_export: bool,
}

impl Default for PdfExportOptions {
    fn default() -> Self {
        Self {
            page_size: PdfPageSize::default(),
            margin_preset: PdfMarginPreset::Comfortable,
            custom_margins: PdfMargins::comfortable(),
            page_break_before_h1: false,
            use_theme_colors: false,
            include_page_numbers: true,
            open_after_export: false,
        }
    }
}

impl PdfExportOptions {
    /// Resolve the effective margins from the preset + custom overrides.
    pub fn effective_margins(&self) -> PdfMargins {
        self.margin_preset.margins(self.custom_margins)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_size_dimensions() {
        assert_eq!(PdfPageSize::A4.width(), 595.0);
        assert_eq!(PdfPageSize::A4.height(), 842.0);
        assert_eq!(PdfPageSize::UsLetter.width(), 612.0);
        assert_eq!(PdfPageSize::UsLetter.height(), 792.0);
    }

    #[test]
    fn margin_presets_resolve() {
        let custom = PdfMargins {
            top: 1.0,
            right: 2.0,
            bottom: 3.0,
            left: 4.0,
        };
        assert_eq!(
            PdfMarginPreset::Narrow.margins(custom),
            PdfMargins::narrow()
        );
        assert_eq!(PdfMarginPreset::Custom.margins(custom), custom);
    }

    #[test]
    fn options_default_serialization_round_trips() {
        let options = PdfExportOptions::default();
        let json = serde_json::to_string(&options).unwrap();
        let back: PdfExportOptions = serde_json::from_str(&json).unwrap();
        assert_eq!(options, back);
    }
}
