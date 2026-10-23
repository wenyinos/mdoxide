//! LaTeX math rendering via RaTeX (pure Rust, KaTeX-compatible).
//!
//! The pipeline is:
//! `LaTeX -> ratex-parser -> ratex-layout -> DisplayList -> PNG bytes`.
//!
//! Callers decode the PNG into an egui texture and cache it per
//! `(formula, color, font_size, dpr)`; this module is stateless.

use ratex_layout::{layout, to_display_list, LayoutOptions};
use ratex_render::{render_to_png, RenderOptions};
use ratex_types::Color;

/// Remove surrounding `$…$`, `$$…$$`, `\(…\)` or `\[…\]` delimiters.
pub fn strip_math_delimiters(input: &str) -> &str {
    let t = input.trim();
    for (open, close) in [("$$", "$$"), ("$", "$"), ("\\[", "\\]"), ("\\(", "\\)")] {
        if let Some(rest) = t.strip_prefix(open) {
            if let Some(inner) = rest.strip_suffix(close) {
                return inner.trim();
            }
        }
    }
    t
}

/// Render a LaTeX formula to PNG bytes.
///
/// * `font_size` – logical size in points (matches the surrounding text).
/// * `color` – RGB text colour.
/// * `dpr` – device pixel ratio; pass the window scale factor for crisp output.
pub fn render_latex_png(
    latex: &str,
    font_size: f32,
    color: [u8; 3],
    dpr: f32,
) -> Result<Vec<u8>, String> {
    let source = strip_math_delimiters(latex);
    if source.is_empty() {
        return Err("empty formula".to_string());
    }

    let nodes = ratex_parser::parse(source).map_err(|e| format!("parse error: {e:?}"))?;
    if nodes.is_empty() {
        return Err("empty formula".to_string());
    }

    let palette = Color::rgb(
        f32::from(color[0]) / 255.0,
        f32::from(color[1]) / 255.0,
        f32::from(color[2]) / 255.0,
    );
    let layout_opts = LayoutOptions::default().with_color(palette);
    let layout_box = layout(&nodes, &layout_opts);
    let display_list = to_display_list(&layout_box);

    let mut opts = RenderOptions::default();
    opts.font_size = font_size.max(6.0);
    opts.padding = 2.0;
    opts.background_color = Color::new(0.0, 0.0, 0.0, 0.0);
    opts.device_pixel_ratio = dpr.clamp(1.0, 4.0);

    render_to_png(&display_list, &opts)
}

#[cfg(test)]
mod tests {
    use super::{render_latex_png, strip_math_delimiters};

    #[test]
    fn strips_common_delimiters() {
        assert_eq!(strip_math_delimiters("$x^2$"), "x^2");
        assert_eq!(strip_math_delimiters("$$\\frac{a}{b}$$"), "\\frac{a}{b}");
        assert_eq!(strip_math_delimiters("\\(x\\)"), "x");
        assert_eq!(strip_math_delimiters("x^2"), "x^2");
    }

    #[test]
    fn renders_simple_formula_to_png() {
        let png = render_latex_png(r"\frac{a}{b}+c", 16.0, [40, 40, 40], 2.0).unwrap();
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
        assert!(png.len() > 200);
    }

    #[test]
    fn rejects_empty_formula() {
        assert!(render_latex_png("$$", 16.0, [0, 0, 0], 1.0).is_err());
    }
}
