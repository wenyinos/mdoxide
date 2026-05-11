//! Persisted options for HTML export (themed document export).

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Which color theme to bake into exported HTML.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum HtmlExportThemeChoice {
    /// Match the active editor palette (light / dark / system) and accent.
    #[default]
    FollowEditor,
    Light,
    Dark,
    /// `prefers-color-scheme` in CSS (body + syntax themes when possible).
    Auto,
}

impl HtmlExportThemeChoice {
    pub fn label(&self) -> &'static str {
        match self {
            HtmlExportThemeChoice::FollowEditor => "Follow editor",
            HtmlExportThemeChoice::Light => "Light",
            HtmlExportThemeChoice::Dark => "Dark",
            HtmlExportThemeChoice::Auto => "Auto (system)",
        }
    }

    pub fn all() -> &'static [HtmlExportThemeChoice] {
        &[
            HtmlExportThemeChoice::FollowEditor,
            HtmlExportThemeChoice::Light,
            HtmlExportThemeChoice::Dark,
            HtmlExportThemeChoice::Auto,
        ]
    }
}

/// User-facing HTML export settings (mirrors the pre-export dialog).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct HtmlExportOptions {
    /// Single self-contained file vs linked assets (images use [`crate::export::ImageHandling`] when not self-contained).
    pub self_contained: bool,
    pub image_handling: crate::export::options::ImageHandling,
    /// Insert a document outline (TOC) before the main content.
    pub include_outline: bool,
    /// Emit HTML comments (`data-sourcepos` on blocks + export header comment).
    pub include_html_comments: bool,
    /// Optional directory used to resolve relative `href` / image paths when not self-contained.
    pub link_base_path: String,
    pub theme: HtmlExportThemeChoice,
    pub include_title: bool,
    pub include_syntax_highlighting: bool,
    pub open_after_export: bool,
    /// Apply Ferrite palette CSS (headings, tables, blockquotes). Off = structural CSS only.
    pub use_theme_colors: bool,
    /// Extra user stylesheet appended at end of `<style>`.
    pub custom_css: Option<String>,
}

impl Default for HtmlExportOptions {
    fn default() -> Self {
        Self {
            self_contained: true,
            image_handling: crate::export::options::ImageHandling::RelativePaths,
            include_outline: false,
            include_html_comments: false,
            link_base_path: String::new(),
            theme: HtmlExportThemeChoice::default(),
            include_title: true,
            include_syntax_highlighting: true,
            open_after_export: false,
            use_theme_colors: true,
            custom_css: None,
        }
    }
}

impl HtmlExportOptions {
    pub fn resolved_link_base(&self, document_dir: Option<&std::path::Path>) -> Option<PathBuf> {
        let trimmed = self.link_base_path.trim();
        if !trimmed.is_empty() {
            return Some(PathBuf::from(trimmed));
        }
        document_dir.map(PathBuf::from)
    }
}
