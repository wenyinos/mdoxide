//! Document Export Module for Ferrite
//!
//! This module provides functionality for exporting markdown documents to various formats
//! including standalone themed HTML and copying HTML to clipboard.
//!
//! # Supported Export Formats
//!
//! - **HTML File**: Complete HTML document with inlined theme CSS
//! - **Clipboard HTML**: Copy rendered HTML to clipboard for pasting in other apps
//!
//! # Architecture
//!
//! - `options.rs` - Export configuration and options
//! - `html.rs` - HTML document generation with theme styling
//! - `clipboard.rs` - Platform clipboard operations

pub mod clipboard;
pub mod flowchart_svg;
pub mod html;
pub mod html_options;
pub mod options;
pub mod pdf;

pub use clipboard::copy_html_to_clipboard;
pub use html::{
    generate_html_document_export, resolve_html_theme_for_export, syntax_dark_mode_for_export,
};
pub use html_options::{HtmlExportOptions, HtmlExportThemeChoice};
pub use pdf::{render_markdown_to_pdf, PdfExportOptions, PdfMarginPreset, PdfPageSize, PdfTheme};
