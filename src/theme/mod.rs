//! Theme System for MDOxide
//!
//! This module provides a comprehensive theming system that defines colors,
//! fonts, and spacing for consistent UI styling across the application.

// Allow dead code - this module has comprehensive theme utilities for future use
#![allow(dead_code)]

//! # Architecture
//!
//! The theme system is built around the `ThemeColors` struct which contains
//! all color definitions needed for the UI. The existing `Theme` enum in
//! `config::settings` (Light/Dark/System) is used to select which palette
//! to use at runtime.
//!
//! # Usage
//!
//! ```ignore
//! use crate::theme::{ThemeColors, ThemeSpacing, ThemeFonts};
//! use crate::config::Theme;
//!
//! // Get colors for the current theme
//! let colors = ThemeColors::from_theme(Theme::Dark, &ctx.global_style().visuals);
//!
//! // Use in egui
//! ui.label(RichText::new("Hello").color(colors.text.primary));
//!
//! // Apply theme to egui context
//! let visuals = colors.to_visuals();
//! ctx.set_visuals(visuals);
//! ```
//!
//! # Theme Files
//!
//! - `light.rs` - Light theme configuration and egui Visuals
//! - `dark.rs` - Dark theme configuration and egui Visuals
//! - `colors.rs` - Color constants and utilities
//!
//! # Color Categories
//!
//! - **Base colors**: Background, foreground, borders
//! - **Text colors**: Primary, secondary, muted, link, code
//! - **Editor colors**: Headings, blockquotes, code blocks, lists
//! - **Syntax colors**: Keywords, strings, comments, etc.
//! - **UI colors**: Accent, success, warning, error

pub mod accent;
pub mod dark;
pub mod light;
pub mod manager;

pub use manager::ThemeManager;

use eframe::egui::Color32;

// ─────────────────────────────────────────────────────────────────────────────
// Theme Colors
// ─────────────────────────────────────────────────────────────────────────────

/// Comprehensive theme colors for the entire application.
///
/// This struct consolidates all color definitions needed for consistent
/// UI theming, replacing the fragmented `EditorColors` and `WidgetColors`
/// with a unified system.
#[derive(Debug, Clone, PartialEq)]
pub struct ThemeColors {
    /// Base UI colors (backgrounds, borders)
    pub base: BaseColors,
    /// Text colors for various contexts
    pub text: TextColors,
    /// Colors for the markdown editor
    pub editor: EditorThemeColors,
    /// Syntax highlighting colors for code blocks
    pub syntax: SyntaxColors,
    /// UI feedback colors (success, warning, error)
    pub ui: UiColors,
}

impl ThemeColors {
    /// Apply the user-selected accent (headings, checkbox, UI accent, selection).
    /// Markdown / UI hyperlinks intentionally stay [`accent::standard_link_color`].
    pub fn apply_user_accent(&mut self, accent: Color32) {
        let dark = self.is_dark();
        self.editor.heading = accent;
        self.editor.checkbox = accent;
        self.ui.accent = accent;
        self.ui.accent_hover = accent::accent_hover(accent, dark);
        self.base.selected = accent::selection_fill(accent, dark);
    }

    /// Create theme colors for the given theme variant with a MDOxide accent.
    ///
    /// This is the primary way to get themed colors. It automatically
    /// selects the appropriate palette based on the theme setting.
    pub fn from_theme(
        theme: crate::config::Theme,
        visuals: &eframe::egui::Visuals,
        accent: Color32,
    ) -> Self {
        let mut palette = match theme {
            crate::config::Theme::Dark => Self::dark(),
            crate::config::Theme::Light => Self::light(),
            crate::config::Theme::System => {
                if visuals.dark_mode {
                    Self::dark()
                } else {
                    Self::light()
                }
            }
        };
        palette.apply_user_accent(accent);
        palette
    }

    /// Get the light theme colors.
    pub fn light() -> Self {
        Self {
            base: BaseColors::light(),
            text: TextColors::light(),
            editor: EditorThemeColors::light(),
            syntax: SyntaxColors::light(),
            ui: UiColors::light(),
        }
    }

    /// Get the dark theme colors.
    pub fn dark() -> Self {
        Self {
            base: BaseColors::dark(),
            text: TextColors::dark(),
            editor: EditorThemeColors::dark(),
            syntax: SyntaxColors::dark(),
            ui: UiColors::dark(),
        }
    }

    /// Check if this is a dark theme (useful for conditional styling).
    pub fn is_dark(&self) -> bool {
        // Dark themes have darker backgrounds
        self.base.background.r() < 128
    }

    /// Convert theme colors to egui Visuals for UI styling.
    ///
    /// This is the primary method to apply the theme to egui. It creates
    /// a complete `Visuals` struct configured with the theme's colors.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use crate::theme::ThemeColors;
    ///
    /// let colors = ThemeColors::dark();
    /// let visuals = colors.to_visuals();
    /// ctx.set_visuals(visuals);
    /// ```
    pub fn to_visuals(&self) -> eframe::egui::Visuals {
        if self.is_dark() {
            dark::visuals_from_palette(self)
        } else {
            light::visuals_from_palette(self)
        }
    }

    /// Create visuals for the given theme variant.
    ///
    /// Convenience method that combines `from_theme` and `to_visuals`.
    pub fn visuals_for_theme(
        theme: crate::config::Theme,
        system_visuals: &eframe::egui::Visuals,
        accent: Color32,
    ) -> eframe::egui::Visuals {
        Self::from_theme(theme, system_visuals, accent).to_visuals()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Base Colors
// ─────────────────────────────────────────────────────────────────────────────

/// Base UI colors for backgrounds and borders.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BaseColors {
    /// Primary background color
    pub background: Color32,
    /// Secondary/elevated background (panels, cards)
    pub background_secondary: Color32,
    /// Tertiary background (inputs, code blocks)
    pub background_tertiary: Color32,
    /// Primary border color
    pub border: Color32,
    /// Subtle border color (dividers)
    pub border_subtle: Color32,
    /// Hover state background
    pub hover: Color32,
    /// Selected/active state background
    pub selected: Color32,
}

impl BaseColors {
    /// Light theme base colors.
    ///
    /// Contrast ratios against white background (#FFFFFF):
    /// - border: ~3.2:1 (meets WCAG AA for UI components)
    /// - border_subtle: ~2.3:1 (for subtle dividers, enhanced from previous)
    /// - hover/selected: sufficient visual distinction
    pub fn light() -> Self {
        Self {
            background: Color32::from_rgb(255, 255, 255),
            background_secondary: Color32::from_rgb(250, 250, 250),
            background_tertiary: Color32::from_rgb(245, 245, 245),
            border: Color32::from_rgb(224, 224, 224), // MarkText: #e0e0e0
            border_subtle: Color32::from_rgb(234, 234, 234), // MarkText: #eaeaea
            hover: Color32::from_rgb(242, 242, 242),
            // Saturated enough to remain visible at the ~40% alpha the
            // MDOxideEditor selection overlay applies on top of text (#121).
            // At full alpha this also works as a light-theme widget-selected
            // color without being harsh.
            selected: Color32::from_rgb(211, 227, 253), // MarkText selection #d3e3fd
        }
    }

    /// Dark theme base colors.
    pub fn dark() -> Self {
        Self {
            background: Color32::from_rgb(40, 44, 52),      // MarkText dark: #282c34
            background_secondary: Color32::from_rgb(33, 37, 43), // #21252b
            background_tertiary: Color32::from_rgb(44, 49, 58),  // #2c313a
            border: Color32::from_rgb(62, 68, 81),          // #3e4451
            border_subtle: Color32::from_rgb(51, 56, 66),   // #333842
            hover: Color32::from_rgb(47, 52, 61),           // #2f343d
            selected: Color32::from_rgb(58, 66, 80),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Text Colors
// ─────────────────────────────────────────────────────────────────────────────

/// Text colors for various contexts.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextColors {
    /// Primary text color (main content)
    pub primary: Color32,
    /// Secondary text color (descriptions, labels)
    pub secondary: Color32,
    /// Muted text color (hints, placeholders)
    pub muted: Color32,
    /// Disabled text color
    pub disabled: Color32,
    /// Link text color
    pub link: Color32,
    /// Code text color (inline code)
    pub code: Color32,
}

impl TextColors {
    /// Light theme text colors.
    ///
    /// Contrast ratios against white background (#FFFFFF):
    /// - primary: ~12.6:1 (exceeds WCAG AAA)
    /// - secondary: ~5.9:1 (exceeds WCAG AA)
    /// - muted: ~5.3:1 (exceeds WCAG AA - improved from ~4.5:1)
    /// - disabled: ~3.5:1 (improved visibility, disabled exempt from WCAG)
    /// - link: ~5.7:1 (exceeds WCAG AA)
    /// - code: ~5.9:1 (exceeds WCAG AA)
    pub fn light() -> Self {
        Self {
            primary: Color32::from_rgb(51, 51, 51),    // MarkText body text #333
            secondary: Color32::from_rgb(102, 102, 102),
            muted: Color32::from_rgb(153, 153, 153),
            disabled: Color32::from_rgb(187, 187, 187),
            link: Color32::from_rgb(64, 158, 255),     // #409eff
            code: Color32::from_rgb(199, 37, 78),      // inline code #c7254e
        }
    }

    /// Dark theme text colors.
    pub fn dark() -> Self {
        Self {
            primary: Color32::from_rgb(171, 178, 191),  // MarkText dark: #abb2bf
            secondary: Color32::from_rgb(157, 165, 180),
            muted: Color32::from_rgb(122, 130, 144),
            disabled: Color32::from_rgb(92, 99, 112),
            link: Color32::from_rgb(97, 175, 239),      // #61afef
            code: Color32::from_rgb(229, 192, 123),     // #e5c07b
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Editor Theme Colors
// ─────────────────────────────────────────────────────────────────────────────

/// Colors specific to the markdown editor.
///
/// These colors are used for rendering markdown elements in both
/// raw and WYSIWYG (rendered) editing modes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EditorThemeColors {
    /// Heading text color (H1-H6)
    pub heading: Color32,
    /// Block quote border color
    pub blockquote_border: Color32,
    /// Block quote text color
    pub blockquote_text: Color32,
    /// Code block background color
    pub code_block_bg: Color32,
    /// Code block border color
    pub code_block_border: Color32,
    /// Horizontal rule color
    pub horizontal_rule: Color32,
    /// List marker color (bullets, numbers)
    pub list_marker: Color32,
    /// Task checkbox color
    pub checkbox: Color32,
    /// Table border color
    pub table_border: Color32,
    /// Table header background
    pub table_header_bg: Color32,
}

impl EditorThemeColors {
    /// Light theme editor colors.
    ///
    /// Contrast ratios against white/light backgrounds:
    /// - heading: ~5.7:1 (WCAG AA compliant for text)
    /// - blockquote_border: ~3.2:1 (meets UI component requirement)
    /// - blockquote_text: ~5.3:1 (WCAG AA compliant)
    /// - horizontal_rule: ~3.2:1 (meets UI component requirement)
    /// - list_marker: ~5.3:1 (WCAG AA compliant)
    pub fn light() -> Self {
        Self {
            heading: Color32::from_rgb(43, 43, 43),          // MarkText #333, one step darker for hierarchy
            blockquote_border: Color32::from_rgb(223, 226, 229), // #dfe2e5
            blockquote_text: Color32::from_rgb(119, 119, 119),   // #777
            code_block_bg: Color32::from_rgb(246, 248, 250),     // #f6f8fa
            code_block_border: Color32::from_rgb(225, 228, 232), // #e1e4e8
            horizontal_rule: Color32::from_rgb(229, 229, 229),   // #e5e5e5
            list_marker: Color32::from_rgb(51, 51, 51),
            checkbox: Color32::from_rgb(64, 158, 255),           // #409eff
            table_border: Color32::from_rgb(223, 226, 229),
            table_header_bg: Color32::from_rgb(246, 248, 250),
        }
    }

    /// Dark theme editor colors.
    pub fn dark() -> Self {
        Self {
            heading: Color32::from_rgb(220, 223, 228),          // #dcdfe4
            blockquote_border: Color32::from_rgb(75, 82, 99),   // #4b5263
            blockquote_text: Color32::from_rgb(157, 165, 180),
            code_block_bg: Color32::from_rgb(33, 37, 43),       // #21252b
            code_block_border: Color32::from_rgb(62, 68, 81),
            horizontal_rule: Color32::from_rgb(62, 68, 81),
            list_marker: Color32::from_rgb(171, 178, 191),
            checkbox: Color32::from_rgb(97, 175, 239),
            table_border: Color32::from_rgb(62, 68, 81),
            table_header_bg: Color32::from_rgb(33, 37, 43),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Syntax Colors
// ─────────────────────────────────────────────────────────────────────────────

/// Colors for syntax highlighting in code blocks.
///
/// These colors are used when syntax highlighting is not available
/// or as fallback colors. The full syntax highlighting uses syntect
/// themes which have their own color definitions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SyntaxColors {
    /// Keyword color (if, else, fn, let, etc.)
    pub keyword: Color32,
    /// String literal color
    pub string: Color32,
    /// Number literal color
    pub number: Color32,
    /// Comment color
    pub comment: Color32,
    /// Function name color
    pub function: Color32,
    /// Type/class name color
    pub type_name: Color32,
    /// Variable name color
    pub variable: Color32,
    /// Operator color (+, -, =, etc.)
    pub operator: Color32,
    /// Punctuation color (brackets, semicolons)
    pub punctuation: Color32,
}

impl SyntaxColors {
    /// Light theme syntax colors.
    pub fn light() -> Self {
        Self {
            keyword: Color32::from_rgb(175, 0, 175),       // Purple
            string: Color32::from_rgb(0, 128, 0),          // Green
            number: Color32::from_rgb(0, 128, 128),        // Teal
            comment: Color32::from_rgb(128, 128, 128),     // Gray
            function: Color32::from_rgb(0, 0, 175),        // Blue
            type_name: Color32::from_rgb(0, 100, 150),     // Dark cyan
            variable: Color32::from_rgb(50, 50, 50),       // Dark gray
            operator: Color32::from_rgb(80, 80, 80),       // Gray
            punctuation: Color32::from_rgb(100, 100, 100), // Medium gray
        }
    }

    /// Dark theme syntax colors.
    pub fn dark() -> Self {
        Self {
            keyword: Color32::from_rgb(198, 120, 221),   // Light purple
            string: Color32::from_rgb(152, 195, 121),    // Light green
            number: Color32::from_rgb(209, 154, 102),    // Orange
            comment: Color32::from_rgb(92, 99, 112),     // Gray
            function: Color32::from_rgb(97, 175, 239),   // Light blue
            type_name: Color32::from_rgb(229, 192, 123), // Yellow
            variable: Color32::from_rgb(224, 108, 117),  // Red/pink
            operator: Color32::from_rgb(171, 178, 191),  // Light gray
            punctuation: Color32::from_rgb(150, 150, 150), // Gray
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// UI Colors
// ─────────────────────────────────────────────────────────────────────────────

/// Colors for UI feedback and interactive elements.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiColors {
    /// Primary accent color (buttons, active elements)
    pub accent: Color32,
    /// Accent color for hover state
    pub accent_hover: Color32,
    /// Success color (confirmations, positive actions)
    pub success: Color32,
    /// Warning color (cautions, alerts)
    pub warning: Color32,
    /// Error color (errors, destructive actions)
    pub error: Color32,
    /// Info color (informational messages)
    pub info: Color32,
    /// Background color for matching bracket highlight
    pub matching_bracket_bg: Color32,
    /// Border color for matching bracket highlight
    pub matching_bracket_border: Color32,
}

impl UiColors {
    /// Light theme UI colors.
    pub fn light() -> Self {
        Self {
            accent: Color32::from_rgb(64, 158, 255),      // #409eff
            accent_hover: Color32::from_rgb(102, 177, 255), // #66b1ff
            success: Color32::from_rgb(103, 194, 58),
            warning: Color32::from_rgb(230, 162, 60),
            error: Color32::from_rgb(245, 108, 108),
            info: Color32::from_rgb(144, 147, 153),
            // Subtle gold/yellow tint for bracket matching - visible but not overpowering
            matching_bracket_bg: Color32::from_rgba_unmultiplied(255, 220, 100, 80),
            matching_bracket_border: Color32::from_rgb(200, 170, 50),
        }
    }

    /// Dark theme UI colors.
    pub fn dark() -> Self {
        Self {
            accent: Color32::from_rgb(97, 175, 239),     // #61afef
            accent_hover: Color32::from_rgb(130, 195, 245),
            success: Color32::from_rgb(152, 195, 121),
            warning: Color32::from_rgb(229, 192, 123),
            error: Color32::from_rgb(224, 108, 117),
            info: Color32::from_rgb(86, 182, 194),
            // Subtle cyan/blue tint for bracket matching - visible on dark backgrounds
            matching_bracket_bg: Color32::from_rgba_unmultiplied(80, 180, 220, 60),
            matching_bracket_border: Color32::from_rgb(100, 180, 220),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Theme Spacing
// ─────────────────────────────────────────────────────────────────────────────

/// Spacing values for consistent layout.
///
/// These values define the standard spacing used throughout the UI
/// to maintain visual consistency.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThemeSpacing {
    /// Extra small spacing (2px)
    pub xs: f32,
    /// Small spacing (4px)
    pub sm: f32,
    /// Medium spacing (8px)
    pub md: f32,
    /// Large spacing (16px)
    pub lg: f32,
    /// Extra large spacing (24px)
    pub xl: f32,
    /// Double extra large spacing (32px)
    pub xxl: f32,
}

impl Default for ThemeSpacing {
    fn default() -> Self {
        Self {
            xs: 2.0,
            sm: 4.0,
            md: 8.0,
            lg: 16.0,
            xl: 24.0,
            xxl: 32.0,
        }
    }
}

impl ThemeSpacing {
    /// Create the default spacing values.
    pub fn new() -> Self {
        Self::default()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::accent;

    #[test]
    fn test_theme_colors_light() {
        let colors = ThemeColors::light();

        // Light theme should have light background
        assert!(colors.base.background.r() > 200);
        assert!(!colors.is_dark());
    }

    #[test]
    fn test_theme_colors_dark() {
        let colors = ThemeColors::dark();

        // Dark theme should have dark background
        assert!(colors.base.background.r() < 50);
        assert!(colors.is_dark());
    }

    #[test]
    fn test_theme_colors_from_theme() {
        let dark_colors = ThemeColors::from_theme(
            crate::config::Theme::Dark,
            &eframe::egui::Visuals::dark(),
            accent::default_accent(),
        );
        assert!(dark_colors.is_dark());

        let light_colors = ThemeColors::from_theme(
            crate::config::Theme::Light,
            &eframe::egui::Visuals::light(),
            accent::default_accent(),
        );
        assert!(!light_colors.is_dark());
    }

    #[test]
    fn test_base_colors_light() {
        let colors = BaseColors::light();
        assert!(colors.background.r() > 200);
        assert!(colors.background_secondary.r() > 200);
    }

    #[test]
    fn test_base_colors_dark() {
        let colors = BaseColors::dark();
        assert!(colors.background.r() < 50);
        assert!(colors.background_secondary.r() < 50);
    }

    #[test]
    fn test_text_colors_contrast() {
        fn luma(c: Color32) -> f32 {
            0.299 * f32::from(c.r()) + 0.587 * f32::from(c.g()) + 0.114 * f32::from(c.b())
        }
        // Light theme: dark text on light background
        assert!(luma(TextColors::light().primary) < 80.0);
        // Dark theme: light text on dark background (MarkText #abb2bf is not pure white)
        assert!(luma(TextColors::dark().primary) > 150.0);
    }

    #[test]
    fn test_editor_colors_heading_distinct() {
        // Headings should be visually distinct
        let light = EditorThemeColors::light();
        let dark = EditorThemeColors::dark();

        // Light theme heading should be different from standard text
        assert_ne!(light.heading, TextColors::light().primary);

        // Dark theme heading should be different from standard text
        assert_ne!(dark.heading, TextColors::dark().primary);
    }

    #[test]
    fn test_syntax_colors_variety() {
        let light = SyntaxColors::light();

        // All syntax colors should be distinct for readability
        assert_ne!(light.keyword, light.string);
        assert_ne!(light.string, light.comment);
        assert_ne!(light.function, light.type_name);
    }

    #[test]
    fn test_ui_colors_feedback() {
        let colors = UiColors::light();

        // Success should be greenish
        assert!(colors.success.g() > colors.success.r());

        // Error should be reddish
        assert!(colors.error.r() > colors.error.g());

        // Warning should be yellowish
        assert!(colors.warning.r() > 200 && colors.warning.g() > 150);
    }

    #[test]
    fn test_spacing_default() {
        let spacing = ThemeSpacing::default();

        assert_eq!(spacing.xs, 2.0);
        assert_eq!(spacing.sm, 4.0);
        assert_eq!(spacing.md, 8.0);
        assert_eq!(spacing.lg, 16.0);
        assert_eq!(spacing.xl, 24.0);
        assert_eq!(spacing.xxl, 32.0);
    }

    #[test]
    fn test_theme_colors_to_visuals_light() {
        let colors = ThemeColors::light();
        let visuals = colors.to_visuals();

        // Light theme visuals should not be dark mode
        assert!(!visuals.dark_mode);

        // Panel fill should match our theme's background
        assert_eq!(visuals.panel_fill, colors.base.background);
    }

    #[test]
    fn test_theme_colors_to_visuals_dark() {
        let colors = ThemeColors::dark();
        let visuals = colors.to_visuals();

        // Dark theme visuals should be dark mode
        assert!(visuals.dark_mode);

        // Panel fill should match our theme's background
        assert_eq!(visuals.panel_fill, colors.base.background);
    }

    #[test]
    fn test_visuals_for_theme_light() {
        let visuals = ThemeColors::visuals_for_theme(
            crate::config::Theme::Light,
            &eframe::egui::Visuals::light(),
            accent::default_accent(),
        );
        assert!(!visuals.dark_mode);
    }

    #[test]
    fn test_visuals_for_theme_dark() {
        let visuals = ThemeColors::visuals_for_theme(
            crate::config::Theme::Dark,
            &eframe::egui::Visuals::dark(),
            accent::default_accent(),
        );
        assert!(visuals.dark_mode);
    }

    #[test]
    fn test_visuals_for_theme_system() {
        // System theme follows the provided visuals
        let dark_visuals = ThemeColors::visuals_for_theme(
            crate::config::Theme::System,
            &eframe::egui::Visuals::dark(),
            accent::default_accent(),
        );
        assert!(dark_visuals.dark_mode);

        let light_visuals = ThemeColors::visuals_for_theme(
            crate::config::Theme::System,
            &eframe::egui::Visuals::light(),
            accent::default_accent(),
        );
        assert!(!light_visuals.dark_mode);
    }
}
