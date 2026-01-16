//! Editor module for Ferrite
//!
//! This module contains the text editor widget and related functionality
//! for editing markdown documents.

mod find_replace;
pub mod folding;
mod line_numbers;
pub mod matching;
mod minimap;
mod outline;
mod stats;
mod widget;

// Only export what's actually used by the app
pub use find_replace::{FindReplacePanel, FindState};
pub use line_numbers::count_lines;
pub use minimap::{Minimap, SemanticMinimap};
pub use outline::{
    extract_outline, extract_outline_for_file, ContentType, DocumentOutline, OutlineItem,
    OutlineType, StructuredStats,
};
pub use stats::{DocumentStats, TextStats};
pub use widget::{EditorWidget, SearchHighlights};
