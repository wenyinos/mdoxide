//! Ferrite Phosphor icon mappings (MIT — [Phosphor Icons](https://phosphoricons.com/)).
//!
//! Central place for toolbar/panel icon glyphs so we don't scatter emoji across the UI.

use std::path::Path;

pub use crate::ui::icons::{phosphor_font, phosphor_rich_text};

// Full regular-weight set — import specific icons at call sites or via this re-export.
pub use egui_phosphor::regular::*;

/// Folder glyph for expanded vs collapsed tree nodes.
pub fn folder_icon(expanded: bool) -> &'static str {
    if expanded {
        FOLDER_OPEN
    } else {
        FOLDER
    }
}

/// Pick a Phosphor file icon for a lowercase extension (no dot).
pub fn file_icon_for_extension(ext: Option<&str>) -> &'static str {
    match ext {
        Some("md" | "markdown" | "mdown" | "mkd") => FILE_MD,
        Some("txt" | "text") => FILE_TEXT,
        Some("rs") => FILE_RS,
        Some("js" | "jsx") => FILE_JS,
        Some("ts" | "tsx") => FILE_TS,
        Some("py") => FILE_PY,
        Some("html" | "htm") => FILE_HTML,
        Some("css" | "scss" | "sass") => FILE_CSS,
        Some("json") => FILE_CODE,
        Some("yaml" | "yml" | "toml") => FILE_CODE,
        Some("xml") => FILE_CODE,
        Some("gitignore" | "env") => WRENCH,
        Some("png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" | "ico" | "bmp") => FILE_IMAGE,
        Some("pdf") => FILE_PDF,
        Some("zip" | "tar" | "gz" | "rar" | "7z") => FILE_ZIP,
        Some("csv" | "tsv") => FILE_TEXT,
        Some("c" | "cpp" | "h" | "hpp" | "go" | "java" | "rb" | "sh" | "bash" | "zsh") => FILE_CODE,
        _ => FILE,
    }
}

/// Pick a Phosphor file icon from a path.
pub fn file_icon_for_path(path: &Path) -> &'static str {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_lowercase);
    file_icon_for_extension(ext.as_deref())
}
