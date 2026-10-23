//! egui widget that renders a LaTeX formula with RaTeX and caches textures.

use std::collections::HashMap;

use eframe::egui::{self, Color32, Id, TextureHandle, TextureOptions, Ui};

use super::math_render::render_latex_png;

/// Cache key: formula plus the parameters that affect the bitmap.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MathKey {
    latex: String,
    color: [u8; 3],
    size_tenths: u32,
    dpr_tenths: u32,
}

#[derive(Default, Clone)]
struct MathCache {
    textures: HashMap<MathKey, TextureHandle>,
}

fn cache_id() -> Id {
    Id::new("mdoxide_math_textures")
}

/// Render `latex` inline (or centred when `display` is true).
///
/// Falls back to a monospace rendering of the raw formula when RaTeX rejects it.
pub fn show_math(ui: &mut Ui, latex: &str, display: bool, font_size: f32, color: Color32) {
    let dpr = ui.ctx().pixels_per_point().clamp(1.0, 4.0);
    let key = MathKey {
        latex: latex.to_string(),
        color: [color.r(), color.g(), color.b()],
        size_tenths: (font_size * 10.0).round().max(60.0) as u32,
        dpr_tenths: (dpr * 10.0).round() as u32,
    };

    let cached = ui
        .ctx()
        .data(|d| d.get_temp::<MathCache>(cache_id()))
        .and_then(|c| c.textures.get(&key).cloned());

    let texture = match cached {
        Some(texture) => texture,
        None => match render_texture(ui, latex, font_size, color, dpr) {
            Some(texture) => {
                ui.ctx().data_mut(|d| {
                    d.get_temp_mut_or_default::<MathCache>(cache_id())
                        .textures
                        .insert(key, texture.clone());
                });
                texture
            }
            None => {
                fallback(ui, latex, display, font_size, color);
                return;
            }
        },
    };

    let logical = texture.size_vec2() / dpr;
    let image = egui::Image::new(&texture).fit_to_exact_size(logical);
    if display {
        ui.vertical_centered(|ui| {
            ui.add(image);
        });
    } else {
        ui.add(image);
    }
}

fn render_texture(
    ui: &mut Ui,
    latex: &str,
    font_size: f32,
    color: Color32,
    dpr: f32,
) -> Option<TextureHandle> {
    let png = match render_latex_png(latex, font_size, [color.r(), color.g(), color.b()], dpr) {
        Ok(png) => png,
        Err(err) => {
            log::debug!("RaTeX failed for {latex:?}: {err}");
            return None;
        }
    };

    let decoded = image::load_from_memory(&png).ok()?.to_rgba8();
    let size = [decoded.width() as usize, decoded.height() as usize];
    let image = egui::ColorImage::from_rgba_unmultiplied(size, decoded.as_raw());
    let name = format!("math:{}", latex.chars().take(24).collect::<String>());
    Some(ui.ctx().load_texture(name, image, TextureOptions::LINEAR))
}

fn fallback(ui: &mut Ui, latex: &str, display: bool, font_size: f32, color: Color32) {
    let text = if display {
        format!("$${latex}$$")
    } else {
        format!("${latex}$")
    };
    if display {
        ui.vertical_centered(|ui| {
            ui.label(
                egui::RichText::new(text)
                    .monospace()
                    .size(font_size)
                    .color(color),
            );
        });
    } else {
        ui.label(
            egui::RichText::new(text)
                .monospace()
                .size(font_size)
                .color(color),
        );
    }
}
