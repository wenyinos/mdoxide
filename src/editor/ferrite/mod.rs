//! Ferrite editor module - custom text editor widget for Ferrite.
//!
//! This module provides a high-performance text editor with:
//! - Rope-based text storage (`TextBuffer`)
//! - Virtual scrolling (`ViewState`)
//! - Galley caching (`LineCache`)
//! - Operation-based undo/redo (`EditHistory`)
//! - Modular input handling and rendering

mod buffer;
mod cursor;
mod editor;
mod find_replace;
mod highlights;
mod history;
mod input;
mod line_cache;
mod mouse;
mod rendering;
mod search;
mod selection;
mod view;
pub mod vim;

// Re-export the main types for external use
pub use buffer::TextBuffer;
pub use cursor::{Cursor, Selection};
pub use editor::{FerriteEditor, SearchMatch};
pub use history::{EditHistory, EditOperation};
pub use line_cache::{HighlightedSegment, LineCache};
pub use view::ViewState;
pub use vim::VimState;
