//! Editor module for MDOxide
//!
//! This module contains the text editor widget and related functionality
//! for editing markdown documents.

// MDOxide editor module (custom high-performance editor)
mod mdoxide;

// Other editor modules
mod find_replace;
pub mod folding;
mod line_numbers;
pub mod matching;
mod minimap;
mod outline;
mod stats;
mod widget;

// Re-export MDOxide editor types
pub use mdoxide::{compute_edit_ops, EditHistory};

// Re-export other editor types
pub use find_replace::{FindReplacePanel, FindState};
pub use line_numbers::count_lines;
pub use minimap::{
    show_split_sync_footer, Minimap, SemanticMinimap, SplitSyncFooterOutput,
    SPLIT_SYNC_FOOTER_HEIGHT,
};
pub use outline::{
    extract_outline, extract_outline_for_file, ContentType, DocumentOutline, OutlineItem,
    OutlineType, StructuredStats,
};
pub use stats::{DocumentStats, TextStats};
pub use widget::{EditorWidget, SearchHighlights};

// Re-export MDOxideEditor access helpers
pub use widget::{cleanup_mdoxide_editor, get_mdoxide_editor_mut};

/// Read plain text from the OS clipboard for context-menu paste.
///
/// egui only delivers paste as an `Event::Paste`; menu actions happen outside
/// the input pass, so we read the clipboard directly. Returns `None` when the
/// clipboard is unavailable or does not contain text.
pub(crate) fn read_clipboard_text() -> Option<String> {
    arboard::Clipboard::new()
        .ok()
        .and_then(|mut clipboard| clipboard.get_text().ok())
}
