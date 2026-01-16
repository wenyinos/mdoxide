//! Font management for Ferrite
//!
//! This module handles loading custom fonts with proper bold/italic variants.
//! Fonts are embedded at compile time using `include_bytes!`.
//!
//! ## Font Selection Features
//!
//! - Built-in fonts: Inter (proportional) and JetBrains Mono (monospace)
//! - Custom system font selection via font-kit
//! - CJK regional font preferences for correct glyph variants
//! - Runtime font reloading without restart

// Allow dead code - includes utility functions for font styling that may be
// used for advanced text rendering features
#![allow(dead_code)]

use egui::{FontData, FontDefinitions, FontFamily, FontId, TextStyle};
use log::{info, warn};
use std::collections::BTreeMap;
use std::sync::OnceLock;

// ─────────────────────────────────────────────────────────────────────────────
// Font Data - Embedded at compile time
// ─────────────────────────────────────────────────────────────────────────────

// Inter font family (UI/proportional)
const INTER_REGULAR: &[u8] = include_bytes!("../assets/fonts/Inter-Regular.ttf");
const INTER_BOLD: &[u8] = include_bytes!("../assets/fonts/Inter-Bold.ttf");
const INTER_ITALIC: &[u8] = include_bytes!("../assets/fonts/Inter-Italic.ttf");
const INTER_BOLD_ITALIC: &[u8] = include_bytes!("../assets/fonts/Inter-BoldItalic.ttf");

// JetBrains Mono font family (code/monospace)
const JETBRAINS_REGULAR: &[u8] = include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf");
const JETBRAINS_BOLD: &[u8] = include_bytes!("../assets/fonts/JetBrainsMono-Bold.ttf");
const JETBRAINS_ITALIC: &[u8] = include_bytes!("../assets/fonts/JetBrainsMono-Italic.ttf");
const JETBRAINS_BOLD_ITALIC: &[u8] = include_bytes!("../assets/fonts/JetBrainsMono-BoldItalic.ttf");

/// Cache for system font list (expensive to compute, do once)
static SYSTEM_FONTS_CACHE: OnceLock<Vec<String>> = OnceLock::new();

// ─────────────────────────────────────────────────────────────────────────────
// System Font Detection
// ─────────────────────────────────────────────────────────────────────────────

use font_kit::family_name::FamilyName;
use font_kit::handle::Handle;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;

// NanumGothic bundled fallback removed per user request.
// We strictly rely on system fonts now.

/// Attempt to load a specific system font from a list of candidates.
///
/// Returns `Some(FontData)` for the first candidate found on the system.
fn load_system_font(families: &[&str]) -> Option<FontData> {
    let source = SystemSource::new();

    for family in families {
        info!("Attempting to load system font: {}", family);
        if let Ok(handle) =
            source.select_best_match(&[FamilyName::Title(family.to_string())], &Properties::new())
        {
            match handle {
                Handle::Path { path, .. } => {
                    info!("Found system font at: {:?}", path);
                    // Read file content
                    if let Ok(bytes) = std::fs::read(&path) {
                        return Some(FontData::from_owned(bytes));
                    }
                }
                Handle::Memory { bytes, .. } => {
                    info!("Found system font in memory ({} bytes)", bytes.len());
                    return Some(FontData::from_owned(bytes.to_vec()));
                }
            }
        }
    }
    None
}

/// Load a specific system font by exact family name.
///
/// Returns `Some(FontData)` if the font is found on the system.
fn load_system_font_by_name(family_name: &str) -> Option<FontData> {
    let source = SystemSource::new();

    info!("Attempting to load custom font: {}", family_name);
    if let Ok(handle) = source.select_best_match(
        &[FamilyName::Title(family_name.to_string())],
        &Properties::new(),
    ) {
        match handle {
            Handle::Path { path, .. } => {
                info!("Found custom font at: {:?}", path);
                if let Ok(bytes) = std::fs::read(&path) {
                    return Some(FontData::from_owned(bytes));
                }
            }
            Handle::Memory { bytes, .. } => {
                info!("Found custom font in memory ({} bytes)", bytes.len());
                return Some(FontData::from_owned(bytes.to_vec()));
            }
        }
    }
    warn!("Custom font '{}' not found on system", family_name);
    None
}

// ─────────────────────────────────────────────────────────────────────────────
// System Font Enumeration
// ─────────────────────────────────────────────────────────────────────────────

/// Get a list of all available system font family names.
///
/// This function caches the result since font enumeration is expensive.
/// The list is sorted alphabetically and deduplicated.
pub fn list_system_fonts() -> &'static [String] {
    SYSTEM_FONTS_CACHE.get_or_init(|| {
        let mut families = std::collections::HashSet::new();
        let source = SystemSource::new();

        info!("Enumerating system fonts...");

        match source.all_families() {
            Ok(family_names) => {
                for name in family_names {
                    // Filter out internal/system fonts that users typically don't want
                    if !name.starts_with('.')
                        && !name.starts_with("System")
                        && !name.contains("LastResort")
                    {
                        families.insert(name);
                    }
                }
            }
            Err(e) => {
                warn!("Failed to enumerate system fonts: {}", e);
            }
        }

        let mut sorted: Vec<String> = families.into_iter().collect();
        sorted.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

        info!("Found {} system font families", sorted.len());
        sorted
    })
}

/// Check if a font family name is available on the system.
pub fn is_font_available(family_name: &str) -> bool {
    list_system_fonts()
        .iter()
        .any(|f| f.eq_ignore_ascii_case(family_name))
}

// ─────────────────────────────────────────────────────────────────────────────
// Font Family Names
// ─────────────────────────────────────────────────────────────────────────────

/// Custom font family for Inter (proportional UI font)
pub const FONT_INTER: &str = "Inter";
/// Custom font family for Inter Bold
pub const FONT_INTER_BOLD: &str = "Inter-Bold";
/// Custom font family for Inter Italic
pub const FONT_INTER_ITALIC: &str = "Inter-Italic";
/// Custom font family for Inter Bold Italic
pub const FONT_INTER_BOLD_ITALIC: &str = "Inter-BoldItalic";

/// Custom font family for JetBrains Mono (monospace/code font)
pub const FONT_JETBRAINS: &str = "JetBrainsMono";
/// Custom font family for JetBrains Mono Bold
pub const FONT_JETBRAINS_BOLD: &str = "JetBrainsMono-Bold";
/// Custom font family for JetBrains Mono Italic
pub const FONT_JETBRAINS_ITALIC: &str = "JetBrainsMono-Italic";
/// Custom font family for JetBrains Mono Bold Italic
pub const FONT_JETBRAINS_BOLD_ITALIC: &str = "JetBrainsMono-BoldItalic";

/// Keys for dynamically loaded CJK system fonts
const FONT_CJK_KR: &str = "CJK_KR";
const FONT_CJK_SC: &str = "CJK_SC";
const FONT_CJK_TC: &str = "CJK_TC";
const FONT_CJK_JP: &str = "CJK_JP";

/// Key for custom user-selected font
const FONT_CUSTOM: &str = "Custom";

// ─────────────────────────────────────────────────────────────────────────────
// Font Loading
// ─────────────────────────────────────────────────────────────────────────────

use crate::config::CjkFontPreference;

/// Track which CJK fonts were successfully loaded.
#[derive(Default)]
struct CjkFontState {
    kr_loaded: bool,
    sc_loaded: bool,
    tc_loaded: bool,
    jp_loaded: bool,
}

impl CjkFontState {
    /// Check if a font key was loaded.
    fn is_loaded(&self, key: &str) -> bool {
        match key {
            FONT_CJK_KR => self.kr_loaded,
            FONT_CJK_SC => self.sc_loaded,
            FONT_CJK_TC => self.tc_loaded,
            FONT_CJK_JP => self.jp_loaded,
            _ => false,
        }
    }
}

/// Load all CJK system fonts and return their loaded state.
fn load_cjk_fonts(fonts: &mut FontDefinitions) -> CjkFontState {
    let mut state = CjkFontState::default();

    // 1. Korean Recommendations
    // MacOS: Apple SD Gothic Neo
    // Windows: Malgun Gothic
    // Linux: Noto Sans CJK KR, NanumGothic
    let kr_candidates = [
        "Apple SD Gothic Neo",
        "Malgun Gothic",
        "Noto Sans CJK KR",
        "NanumGothic",
    ];
    if let Some(data) = load_system_font(&kr_candidates) {
        fonts.font_data.insert(FONT_CJK_KR.to_owned(), data);
        state.kr_loaded = true;
    }

    // 2. Simplified Chinese Recommendations
    // MacOS: PingFang SC
    // Windows: Microsoft YaHei
    // Linux: Noto Sans CJK SC
    let sc_candidates = ["PingFang SC", "Microsoft YaHei", "Noto Sans CJK SC"];
    if let Some(data) = load_system_font(&sc_candidates) {
        fonts.font_data.insert(FONT_CJK_SC.to_owned(), data);
        state.sc_loaded = true;
    }

    // 3. Traditional Chinese Recommendations
    // MacOS: PingFang TC
    // Windows: Microsoft JhengHei
    // Linux: Noto Sans CJK TC
    let tc_candidates = ["PingFang TC", "Microsoft JhengHei", "Noto Sans CJK TC"];
    if let Some(data) = load_system_font(&tc_candidates) {
        fonts.font_data.insert(FONT_CJK_TC.to_owned(), data);
        state.tc_loaded = true;
    }

    // 4. Japanese Recommendations
    // MacOS: Hiragino Sans, Hiragino Kaku Gothic ProN
    // Windows: Yu Gothic, Meiryo
    // Linux: Noto Sans CJK JP
    let jp_candidates = [
        "Hiragino Sans",
        "Hiragino Kaku Gothic ProN",
        "Yu Gothic",
        "Meiryo",
        "Noto Sans CJK JP",
    ];
    if let Some(data) = load_system_font(&jp_candidates) {
        fonts.font_data.insert(FONT_CJK_JP.to_owned(), data);
        state.jp_loaded = true;
    }

    if !state.kr_loaded && !state.sc_loaded && !state.tc_loaded && !state.jp_loaded {
        warn!("No system CJK fonts were found. CJK rendering may fail.");
    } else {
        info!(
            "System CJK fonts loaded: KR={}, SC={}, TC={}, JP={}",
            state.kr_loaded, state.sc_loaded, state.tc_loaded, state.jp_loaded
        );
    }

    state
}

/// Add CJK fonts to a font family in the specified order.
fn add_cjk_fallbacks(
    fonts: &mut FontDefinitions,
    family: FontFamily,
    cjk_state: &CjkFontState,
    preference: CjkFontPreference,
) {
    let order = preference.font_order();
    for key in order {
        if cjk_state.is_loaded(key) {
            fonts
                .families
                .entry(family.clone())
                .or_default()
                .push((*key).to_owned());
        }
    }
}

/// Create font definitions with custom fonts loaded.
///
/// This sets up:
/// - Inter as the proportional (UI) font with bold/italic variants
/// - JetBrains Mono as the monospace (code) font with bold/italic variants
/// - Custom named font families for explicit bold/italic access
/// - Optional custom system font
/// - CJK fonts in order based on user preference
pub fn create_font_definitions() -> FontDefinitions {
    create_font_definitions_with_settings(None, CjkFontPreference::Auto)
}

/// Create font definitions with custom settings.
///
/// # Arguments
///
/// * `custom_font` - Optional custom system font name to use as primary editor font
/// * `cjk_preference` - CJK font preference for regional glyph variants
pub fn create_font_definitions_with_settings(
    custom_font: Option<&str>,
    cjk_preference: CjkFontPreference,
) -> FontDefinitions {
    let mut fonts = FontDefinitions::default();

    // Insert Inter font variants (always available as UI fallback)
    fonts
        .font_data
        .insert(FONT_INTER.to_owned(), FontData::from_static(INTER_REGULAR));
    fonts.font_data.insert(
        FONT_INTER_BOLD.to_owned(),
        FontData::from_static(INTER_BOLD),
    );
    fonts.font_data.insert(
        FONT_INTER_ITALIC.to_owned(),
        FontData::from_static(INTER_ITALIC),
    );
    fonts.font_data.insert(
        FONT_INTER_BOLD_ITALIC.to_owned(),
        FontData::from_static(INTER_BOLD_ITALIC),
    );

    // Insert JetBrains Mono font variants
    fonts.font_data.insert(
        FONT_JETBRAINS.to_owned(),
        FontData::from_static(JETBRAINS_REGULAR),
    );
    fonts.font_data.insert(
        FONT_JETBRAINS_BOLD.to_owned(),
        FontData::from_static(JETBRAINS_BOLD),
    );
    fonts.font_data.insert(
        FONT_JETBRAINS_ITALIC.to_owned(),
        FontData::from_static(JETBRAINS_ITALIC),
    );
    fonts.font_data.insert(
        FONT_JETBRAINS_BOLD_ITALIC.to_owned(),
        FontData::from_static(JETBRAINS_BOLD_ITALIC),
    );

    // Load custom font if specified
    let custom_loaded = if let Some(font_name) = custom_font {
        if let Some(data) = load_system_font_by_name(font_name) {
            fonts.font_data.insert(FONT_CUSTOM.to_owned(), data);
            info!("Loaded custom font: {}", font_name);
            true
        } else {
            warn!("Custom font '{}' not found, falling back to Inter", font_name);
            false
        }
    } else {
        false
    };

    // Load CJK fonts
    let cjk_state = load_cjk_fonts(&mut fonts);

    // Set up Proportional font family
    // Order: Custom (if set) -> Inter -> CJK fonts (in preference order)
    if custom_loaded {
        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .push(FONT_CUSTOM.to_owned());
    }
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .push(FONT_INTER.to_owned());

    add_cjk_fallbacks(&mut fonts, FontFamily::Proportional, &cjk_state, cjk_preference);

    // Set up Monospace font family
    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .push(FONT_JETBRAINS.to_owned());

    add_cjk_fallbacks(&mut fonts, FontFamily::Monospace, &cjk_state, cjk_preference);

    // Get fallback fonts from default families for CJK/Korean support
    let proportional_fallbacks: Vec<String> = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    let monospace_fallbacks: Vec<String> = fonts
        .families
        .get(&FontFamily::Monospace)
        .cloned()
        .unwrap_or_default();

    // Create custom named font families for explicit style access
    // These allow us to directly select bold/italic fonts
    // Each family includes fallbacks for CJK character support

    // Custom font family (if loaded)
    if custom_loaded {
        let mut custom_family = vec![FONT_CUSTOM.to_owned()];
        custom_family.extend(proportional_fallbacks.clone());
        fonts
            .families
            .insert(FontFamily::Name(FONT_CUSTOM.into()), custom_family);
    }

    // Inter variants with proportional fallbacks
    let mut inter_family = vec![FONT_INTER.to_owned()];
    inter_family.extend(proportional_fallbacks.clone());
    fonts
        .families
        .insert(FontFamily::Name(FONT_INTER.into()), inter_family);

    let mut inter_bold_family = vec![FONT_INTER_BOLD.to_owned()];
    inter_bold_family.extend(proportional_fallbacks.clone());
    fonts
        .families
        .insert(FontFamily::Name(FONT_INTER_BOLD.into()), inter_bold_family);

    let mut inter_italic_family = vec![FONT_INTER_ITALIC.to_owned()];
    inter_italic_family.extend(proportional_fallbacks.clone());
    fonts.families.insert(
        FontFamily::Name(FONT_INTER_ITALIC.into()),
        inter_italic_family,
    );

    let mut inter_bold_italic_family = vec![FONT_INTER_BOLD_ITALIC.to_owned()];
    inter_bold_italic_family.extend(proportional_fallbacks);
    fonts.families.insert(
        FontFamily::Name(FONT_INTER_BOLD_ITALIC.into()),
        inter_bold_italic_family,
    );

    // JetBrains Mono variants with monospace fallbacks
    let mut jetbrains_family = vec![FONT_JETBRAINS.to_owned()];
    jetbrains_family.extend(monospace_fallbacks.clone());
    fonts
        .families
        .insert(FontFamily::Name(FONT_JETBRAINS.into()), jetbrains_family);

    let mut jetbrains_bold_family = vec![FONT_JETBRAINS_BOLD.to_owned()];
    jetbrains_bold_family.extend(monospace_fallbacks.clone());
    fonts.families.insert(
        FontFamily::Name(FONT_JETBRAINS_BOLD.into()),
        jetbrains_bold_family,
    );

    let mut jetbrains_italic_family = vec![FONT_JETBRAINS_ITALIC.to_owned()];
    jetbrains_italic_family.extend(monospace_fallbacks.clone());
    fonts.families.insert(
        FontFamily::Name(FONT_JETBRAINS_ITALIC.into()),
        jetbrains_italic_family,
    );

    let mut jetbrains_bold_italic_family = vec![FONT_JETBRAINS_BOLD_ITALIC.to_owned()];
    jetbrains_bold_italic_family.extend(monospace_fallbacks);
    fonts.families.insert(
        FontFamily::Name(FONT_JETBRAINS_BOLD_ITALIC.into()),
        jetbrains_bold_italic_family,
    );

    info!(
        "Loaded fonts: Inter, JetBrains Mono, CJK (preference: {:?}), custom: {}",
        cjk_preference,
        custom_font.unwrap_or("none")
    );

    fonts
}

/// Apply custom fonts to an egui context.
///
/// This should be called once during application initialization.
pub fn setup_fonts(ctx: &egui::Context) {
    setup_fonts_with_settings(ctx, None, CjkFontPreference::Auto);
}

/// Apply custom fonts to an egui context with settings.
///
/// # Arguments
///
/// * `ctx` - The egui context
/// * `custom_font` - Optional custom system font name
/// * `cjk_preference` - CJK font preference for regional glyph variants
pub fn setup_fonts_with_settings(
    ctx: &egui::Context,
    custom_font: Option<&str>,
    cjk_preference: CjkFontPreference,
) {
    let fonts = create_font_definitions_with_settings(custom_font, cjk_preference);
    ctx.set_fonts(fonts);

    // Configure text styles with appropriate sizes
    let text_styles: BTreeMap<TextStyle, FontId> = [
        (
            TextStyle::Heading,
            FontId::new(24.0, FontFamily::Proportional),
        ),
        (TextStyle::Body, FontId::new(14.0, FontFamily::Proportional)),
        (
            TextStyle::Monospace,
            FontId::new(14.0, FontFamily::Monospace),
        ),
        (
            TextStyle::Button,
            FontId::new(14.0, FontFamily::Proportional),
        ),
        (
            TextStyle::Small,
            FontId::new(12.0, FontFamily::Proportional),
        ),
    ]
    .into();

    ctx.style_mut(|style| {
        style.text_styles = text_styles.clone();
    });

    info!("Configured egui text styles with custom_font={:?}, cjk_preference={:?}", 
          custom_font, cjk_preference);
}

/// Reload fonts at runtime with new settings.
///
/// This can be called when font settings change in the UI.
pub fn reload_fonts(
    ctx: &egui::Context,
    custom_font: Option<&str>,
    cjk_preference: CjkFontPreference,
) {
    info!("Reloading fonts with custom_font={:?}, cjk_preference={:?}", 
          custom_font, cjk_preference);
    setup_fonts_with_settings(ctx, custom_font, cjk_preference);
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper Functions for Getting Font Families
// ─────────────────────────────────────────────────────────────────────────────

use crate::config::EditorFont;

/// Get the appropriate font family for styled text based on editor font setting.
///
/// This returns the correct font variant based on bold/italic flags and the
/// user's selected editor font.
///
/// Note: Custom system fonts don't have separate bold/italic variants loaded,
/// so they use the base custom font for all styles. The OS may synthesize
/// bold/italic styles, but this depends on the specific font and platform.
pub fn get_styled_font_family(bold: bool, italic: bool, editor_font: &EditorFont) -> FontFamily {
    match editor_font {
        EditorFont::JetBrainsMono => match (bold, italic) {
            (true, true) => FontFamily::Name(FONT_JETBRAINS_BOLD_ITALIC.into()),
            (true, false) => FontFamily::Name(FONT_JETBRAINS_BOLD.into()),
            (false, true) => FontFamily::Name(FONT_JETBRAINS_ITALIC.into()),
            (false, false) => FontFamily::Name(FONT_JETBRAINS.into()),
        },
        EditorFont::Inter => match (bold, italic) {
            (true, true) => FontFamily::Name(FONT_INTER_BOLD_ITALIC.into()),
            (true, false) => FontFamily::Name(FONT_INTER_BOLD.into()),
            (false, true) => FontFamily::Name(FONT_INTER_ITALIC.into()),
            (false, false) => FontFamily::Name(FONT_INTER.into()),
        },
        // Custom fonts don't have separate bold/italic variants
        // Use the custom font family which has CJK fallbacks
        EditorFont::Custom(_) => FontFamily::Name(FONT_CUSTOM.into()),
    }
}

/// Get the base font family for an editor font (regular weight, no style).
pub fn get_base_font_family(editor_font: &EditorFont) -> FontFamily {
    match editor_font {
        EditorFont::Inter => FontFamily::Name(FONT_INTER.into()),
        EditorFont::JetBrainsMono => FontFamily::Name(FONT_JETBRAINS.into()),
        EditorFont::Custom(_) => FontFamily::Name(FONT_CUSTOM.into()),
    }
}

/// Create a FontId for styled text.
///
/// Convenience function that combines size with the appropriate styled font family.
pub fn styled_font_id(size: f32, bold: bool, italic: bool, editor_font: &EditorFont) -> FontId {
    FontId::new(size, get_styled_font_family(bold, italic, editor_font))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_font_definitions() {
        let fonts = create_font_definitions();

        // Check that all font data is loaded
        assert!(fonts.font_data.contains_key(FONT_INTER));
        assert!(fonts.font_data.contains_key(FONT_INTER_BOLD));
        assert!(fonts.font_data.contains_key(FONT_INTER_ITALIC));
        assert!(fonts.font_data.contains_key(FONT_INTER_BOLD_ITALIC));

        assert!(fonts.font_data.contains_key(FONT_JETBRAINS));
        assert!(fonts.font_data.contains_key(FONT_JETBRAINS_BOLD));
        assert!(fonts.font_data.contains_key(FONT_JETBRAINS_ITALIC));
        assert!(fonts.font_data.contains_key(FONT_JETBRAINS_BOLD_ITALIC));

        // Check that font families are set up
        assert!(fonts.families.contains_key(&FontFamily::Proportional));
        assert!(fonts.families.contains_key(&FontFamily::Monospace));
    }

    #[test]
    fn test_get_styled_font_family_inter() {
        // Inter variants
        assert_eq!(
            get_styled_font_family(false, false, &EditorFont::Inter),
            FontFamily::Name(FONT_INTER.into())
        );
        assert_eq!(
            get_styled_font_family(true, false, &EditorFont::Inter),
            FontFamily::Name(FONT_INTER_BOLD.into())
        );
        assert_eq!(
            get_styled_font_family(false, true, &EditorFont::Inter),
            FontFamily::Name(FONT_INTER_ITALIC.into())
        );
        assert_eq!(
            get_styled_font_family(true, true, &EditorFont::Inter),
            FontFamily::Name(FONT_INTER_BOLD_ITALIC.into())
        );
    }

    #[test]
    fn test_get_styled_font_family_jetbrains() {
        // JetBrains Mono variants
        assert_eq!(
            get_styled_font_family(false, false, &EditorFont::JetBrainsMono),
            FontFamily::Name(FONT_JETBRAINS.into())
        );
        assert_eq!(
            get_styled_font_family(true, false, &EditorFont::JetBrainsMono),
            FontFamily::Name(FONT_JETBRAINS_BOLD.into())
        );
        assert_eq!(
            get_styled_font_family(false, true, &EditorFont::JetBrainsMono),
            FontFamily::Name(FONT_JETBRAINS_ITALIC.into())
        );
        assert_eq!(
            get_styled_font_family(true, true, &EditorFont::JetBrainsMono),
            FontFamily::Name(FONT_JETBRAINS_BOLD_ITALIC.into())
        );
    }

    #[test]
    fn test_get_styled_font_family_custom() {
        // Custom font always returns FONT_CUSTOM
        let custom = EditorFont::Custom("Test Font".to_string());
        assert_eq!(
            get_styled_font_family(false, false, &custom),
            FontFamily::Name(FONT_CUSTOM.into())
        );
        assert_eq!(
            get_styled_font_family(true, true, &custom),
            FontFamily::Name(FONT_CUSTOM.into())
        );
    }

    #[test]
    fn test_styled_font_id() {
        let font_id = styled_font_id(16.0, true, false, &EditorFont::Inter);
        assert_eq!(font_id.size, 16.0);
        assert_eq!(font_id.family, FontFamily::Name(FONT_INTER_BOLD.into()));
    }

    #[test]
    fn test_cjk_font_preference_order() {
        // Test that preference returns correct font order
        assert_eq!(
            CjkFontPreference::Korean.font_order(),
            &["CJK_KR", "CJK_SC", "CJK_TC", "CJK_JP"]
        );
        assert_eq!(
            CjkFontPreference::Japanese.font_order(),
            &["CJK_JP", "CJK_KR", "CJK_SC", "CJK_TC"]
        );
        assert_eq!(
            CjkFontPreference::SimplifiedChinese.font_order(),
            &["CJK_SC", "CJK_TC", "CJK_KR", "CJK_JP"]
        );
    }
}
