//! User-configurable Ferrite accent (headings, selection tint, UI chrome).
//! Standard hyperlink blues are fixed in [`standard_link_color`].

use eframe::egui::Color32;

pub const DEFAULT_ACCENT_RGB: [u8; 3] = [100, 180, 255];

#[inline]
pub fn default_accent() -> Color32 {
    Color32::from_rgb(
        DEFAULT_ACCENT_RGB[0],
        DEFAULT_ACCENT_RGB[1],
        DEFAULT_ACCENT_RGB[2],
    )
}

/// Classic link blues (not controlled by accent).
#[inline]
pub fn standard_link_color(dark_mode: bool) -> Color32 {
    if dark_mode {
        Color32::from_rgb(100, 180, 255)
    } else {
        Color32::from_rgb(0, 90, 170)
    }
}

#[inline]
fn lerp_channel(a: u8, b: u8, t: f32) -> u8 {
    ((f32::from(a)) * (1.0 - t) + (f32::from(b)) * t)
        .round()
        .clamp(0.0, 255.0) as u8
}

pub fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    Color32::from_rgb(
        lerp_channel(a.r(), b.r(), t),
        lerp_channel(a.g(), b.g(), t),
        lerp_channel(a.b(), b.b(), t),
    )
}

pub fn accent_hover(accent: Color32, dark: bool) -> Color32 {
    if dark {
        lerp_color(accent, Color32::WHITE, 0.12)
    } else {
        lerp_color(accent, Color32::BLACK, 0.15)
    }
}

/// egui selection / “open” widget fill derived from accent.
pub fn selection_fill(accent: Color32, dark: bool) -> Color32 {
    if dark {
        let bg = Color32::from_rgb(30, 30, 30);
        lerp_color(bg, accent, 0.42)
    } else {
        lerp_color(Color32::WHITE, accent, 0.28)
    }
}

/// Outline / sidebar highlights (muted vs full selection_fill).
pub fn panel_highlight_fill(
    panel_bg: Color32,
    accent: Color32,
    dark: bool,
    strength: f32,
) -> Color32 {
    let t = if dark { strength } else { strength * 0.55 };
    lerp_color(panel_bg, accent, t.clamp(0.0, 1.0))
}
