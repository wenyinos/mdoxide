//! PDF Export for Ferrite.
//!
//! Native‑Rust PDF export built on the [`krilla`](https://crates.io/crates/krilla)
//! crate (same author as the existing `hayro` viewer). Walks the comrak AST,
//! buffers draw operations per page, and replays them onto krilla pages on
//! finalize.
//!
//! See `docs/technical/planning/pdf-export-pipeline.md` for the full design
//! decision and v1 vs v2+ scope.
//!
//! # Public surface
//!
//! - [`render_markdown_to_pdf`] — render markdown source to a `Vec<u8>` PDF.
//! - [`PdfExportError`] — the error type the renderer returns.
//! - [`PdfExportOptions`], [`PdfPageSize`], [`PdfMargins`], [`PdfMarginPreset`]
//!   — the user‑configurable knobs.
//! - [`PdfTheme`] — the color set used for output.
//!
//! # Known v1 limitations
//!
//! - Mermaid code fences are rendered as plain code blocks (the SVG emit path
//!   is roadmap Phase 4).
//! - Images are rendered as italic placeholders (raster embedding is a
//!   follow‑up).
//! - Text wrapping uses approximate per‑character widths; non‑Latin scripts
//!   may wrap loosely.
//! - Right‑to‑left bidi reordering is not yet applied.

pub mod fonts;
pub mod options;
pub mod render;
pub mod theme;

pub use options::{PdfExportOptions, PdfMarginPreset, PdfPageSize};
// `PdfExportError` is part of the public PDF API surface so external callers
// can pattern-match on it even though no in-tree caller does today.
pub use render::render_markdown_to_pdf;
#[allow(unused_imports)]
pub use render::PdfExportError;
pub use theme::PdfTheme;
