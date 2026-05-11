//! Light Theme Configuration
//!
//! This module provides the light theme for Ferrite.
//! It converts the `ThemeColors::light()` palette into egui's `Visuals`
//! for consistent UI styling.

// Allow dead code - exports are available for future use
#![allow(dead_code)]

//! # Design Principles
//!
//! - High contrast for readability
//! - Soft backgrounds to reduce eye strain
//! - Professional appearance suitable for extended use
//! - Accessible color choices (WCAG AA compliant)

use eframe::egui::{self, Color32, CornerRadius, Stroke, Visuals};

use super::{ThemeColors, ThemeSpacing};

/// Create egui Visuals configured for the light theme.
///
/// This converts our custom `ThemeColors::light()` palette into egui's
/// native `Visuals` structure for consistent UI styling.
///
/// # Example
///
/// ```ignore
/// use crate::theme::light::create_light_visuals;
///
/// let ctx = &egui::Context::default();
/// ctx.set_visuals(create_light_visuals());
/// ```
/// Build Ferrite visuals from an already-built light palette (`ThemeColors`).
pub(crate) fn visuals_from_palette(colors: &ThemeColors) -> Visuals {
    let spacing = ThemeSpacing::default();

    let mut visuals = Visuals::light();

    // ─────────────────────────────────────────────────────────────────────────
    // Window & Panel Background
    // ─────────────────────────────────────────────────────────────────────────
    visuals.panel_fill = colors.base.background;
    visuals.window_fill = colors.base.background;
    visuals.extreme_bg_color = colors.base.background_tertiary;
    visuals.faint_bg_color = colors.base.background_secondary;
    visuals.code_bg_color = colors.editor.code_block_bg;

    // ─────────────────────────────────────────────────────────────────────────
    // Text Colors
    // ─────────────────────────────────────────────────────────────────────────
    // Use theme primary so all widget text (slider values, combobox, drag value,
    // labels) has readable contrast on light background in both themes.
    visuals.override_text_color = Some(colors.text.primary);
    visuals.warn_fg_color = colors.ui.warning;
    visuals.error_fg_color = colors.ui.error;
    visuals.hyperlink_color = colors.text.link;

    // ─────────────────────────────────────────────────────────────────────────
    // Selection
    // ─────────────────────────────────────────────────────────────────────────
    visuals.selection.bg_fill = colors.base.selected;
    visuals.selection.stroke = Stroke::new(1.0, colors.ui.accent);

    // ─────────────────────────────────────────────────────────────────────────
    // Widget Styling (Noninteractive)
    // ─────────────────────────────────────────────────────────────────────────
    visuals.widgets.noninteractive.bg_fill = colors.base.background_secondary;
    visuals.widgets.noninteractive.weak_bg_fill = colors.base.background_tertiary;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, colors.base.border_subtle);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, colors.text.primary);
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(spacing.sm as u8);

    // ─────────────────────────────────────────────────────────────────────────
    // Widget Styling (Inactive/Default)
    // ─────────────────────────────────────────────────────────────────────────
    visuals.widgets.inactive.bg_fill = colors.base.background_secondary;
    visuals.widgets.inactive.weak_bg_fill = colors.base.background_tertiary;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, colors.base.border);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, colors.text.secondary);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(spacing.sm as u8);

    // ─────────────────────────────────────────────────────────────────────────
    // Widget Styling (Hovered)
    // ─────────────────────────────────────────────────────────────────────────
    visuals.widgets.hovered.bg_fill = colors.base.hover;
    visuals.widgets.hovered.weak_bg_fill = colors.base.hover;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, colors.ui.accent);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.5, colors.text.primary);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(spacing.sm as u8);

    // ─────────────────────────────────────────────────────────────────────────
    // Widget Styling (Active/Pressed)
    // ─────────────────────────────────────────────────────────────────────────
    // NOTE: `active.fg_stroke.color` is also returned by `Visuals::strong_text_color()`
    // which egui uses for `RichText::strong()`. Using WHITE here would make all
    // `.strong()` labels invisible on light backgrounds. We use the primary text
    // color which has good contrast on both the accent bg_fill and light panels.
    visuals.widgets.active.bg_fill = colors.ui.accent;
    visuals.widgets.active.weak_bg_fill = colors.base.selected;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, colors.ui.accent_hover);
    visuals.widgets.active.fg_stroke = Stroke::new(2.0, colors.text.primary);
    visuals.widgets.active.corner_radius = CornerRadius::same(spacing.sm as u8);

    // ─────────────────────────────────────────────────────────────────────────
    // Widget Styling (Open/Expanded)
    // ─────────────────────────────────────────────────────────────────────────
    visuals.widgets.open.bg_fill = colors.base.selected;
    visuals.widgets.open.weak_bg_fill = colors.base.selected;
    visuals.widgets.open.bg_stroke = Stroke::new(1.0, colors.ui.accent);
    visuals.widgets.open.fg_stroke = Stroke::new(1.0, colors.text.primary);
    visuals.widgets.open.corner_radius = CornerRadius::same(spacing.sm as u8);

    // ─────────────────────────────────────────────────────────────────────────
    // Window & Popup Styling
    // ─────────────────────────────────────────────────────────────────────────
    visuals.window_corner_radius = CornerRadius::same(spacing.md as u8);
    visuals.window_shadow = egui::epaint::Shadow {
        offset: [0, 2],
        blur: 8,
        spread: 0,
        color: Color32::from_black_alpha(25),
    };
    visuals.window_stroke = Stroke::new(1.0, colors.base.border);

    visuals.popup_shadow = egui::epaint::Shadow {
        offset: [0, 4],
        blur: 12,
        spread: 0,
        color: Color32::from_black_alpha(30),
    };

    visuals.menu_corner_radius = CornerRadius::same(spacing.sm as u8);

    // ─────────────────────────────────────────────────────────────────────────
    // Miscellaneous
    // ─────────────────────────────────────────────────────────────────────────
    visuals.resize_corner_size = 12.0;
    visuals.clip_rect_margin = 3.0;
    visuals.button_frame = true;
    visuals.collapsing_header_frame = false;
    visuals.indent_has_left_vline = true;
    visuals.striped = true;
    visuals.slider_trailing_fill = true;
    visuals.interact_cursor = Some(egui::CursorIcon::PointingHand);

    // Dark mode flag
    visuals.dark_mode = false;

    visuals
}

/// Ferrite light theme egui visuals with the given Ferrite accent.
pub fn create_light_visuals(accent: Color32) -> Visuals {
    let mut palette = ThemeColors::light();
    palette.apply_user_accent(accent);
    visuals_from_palette(&palette)
}

/// Alias for callers that omit accent (defaults to canonical Ferrite blue).
pub fn create_light_visuals_defaults() -> Visuals {
    create_light_visuals(super::accent::default_accent())
}

/// Get the light theme colors.
///
/// This is a convenience re-export of `ThemeColors::light()`.
pub fn colors() -> ThemeColors {
    ThemeColors::light()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_light_visuals_is_light_mode() {
        let visuals = create_light_visuals_defaults();
        assert!(!visuals.dark_mode);
    }

    #[test]
    fn test_light_visuals_has_light_background() {
        let visuals = create_light_visuals_defaults();
        // Light theme should have bright panel fill
        assert!(visuals.panel_fill.r() > 200);
        assert!(visuals.panel_fill.g() > 200);
        assert!(visuals.panel_fill.b() > 200);
    }

    #[test]
    fn test_light_colors_available() {
        let colors = colors();
        assert!(!colors.is_dark());
    }

    #[test]
    fn test_light_visuals_selection_visible() {
        let visuals = create_light_visuals_defaults();
        // Selection should be visually distinct
        assert_ne!(visuals.selection.bg_fill, visuals.panel_fill);
    }

    #[test]
    fn test_light_visuals_text_contrast() {
        let visuals = create_light_visuals_defaults();
        let colors = colors();

        // Text stroke should be dark for contrast on light background
        assert!(visuals.widgets.noninteractive.fg_stroke.color.r() < 100);

        // Verify we're using our theme colors
        assert_eq!(
            visuals.widgets.noninteractive.fg_stroke.color,
            colors.text.primary
        );
    }
}
