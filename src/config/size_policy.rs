//! Single source of truth for document size thresholds.
//!
//! Historically the same file could be "large" for one subsystem and "small"
//! for another (1 MB tab mode, 5 MB background load / editor sync, 10 MB
//! warning). These constants keep behaviour consistent.

/// Tab switches to hash-based modification tracking at/above this size.
pub const DOCUMENT_LARGE_BYTES: usize = 1_000_000;

/// Raw editor enters its reduced-feature path at/above this size.
pub const EDITOR_LARGE_BYTES: usize = 5 * 1024 * 1024;

/// Files at/above this size are loaded on a background thread.
pub const BACKGROUND_LOAD_BYTES: u64 = 5 * 1024 * 1024;

/// Show the performance warning toast at/above this size.
pub const LARGE_FILE_WARNING_BYTES: u64 = 10 * 1024 * 1024;
