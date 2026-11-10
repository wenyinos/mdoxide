//! Document and tab domain types.
//!
//! Extracted from `state.rs` as the first increment of P1-04: the document
//! body, disk identity, encoding, revision counters and per-tab view state
//! live here; `AppState` only aggregates and coordinates them.

use super::*;

// Private parent helpers are not pulled in by the glob; import them explicitly.
use super::{hash_content, line_col_to_char_index};

/// Threshold in bytes above which a file is considered "large" and gets memory optimizations.
/// Large files:
/// - Use hash-based modification detection instead of storing full original_content
/// - Clear original_bytes after initial load to save memory
pub const LARGE_FILE_THRESHOLD: usize = crate::config::DOCUMENT_LARGE_BYTES;

/// Reduced max undo groups for large files (operations are tiny, but cap defensively).
pub(crate) const LARGE_FILE_MAX_UNDO_GROUPS: usize = 200;

// ─────────────────────────────────────────────────────────────────────────────
// Tab Kind (Document vs Special)
// ─────────────────────────────────────────────────────────────────────────────

/// Types of special (non-editable) tabs that display application UI.
///
/// Special tabs render their own content (settings, help, etc.) instead of a
/// document editor. They cannot be edited, have no view mode, and never prompt
/// to save. This is designed to be extensible for future panel types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialTabKind {
    /// Application settings panel
    Settings,
    /// About/Help information panel
    About,
    /// Application welcome panel
    Welcome,
}

impl SpecialTabKind {
    /// Get the display title for this special tab kind.
    pub fn title(&self) -> &'static str {
        match self {
            SpecialTabKind::Settings => "Settings",
            SpecialTabKind::About => "About / Help",
            SpecialTabKind::Welcome => "Welcome",
        }
    }

    /// Get the icon for this special tab kind.
    pub fn icon(&self) -> &'static str {
        use crate::ui::phosphor_icons::{GEAR, INFO, SPARKLE};
        match self {
            SpecialTabKind::Settings => GEAR,
            SpecialTabKind::About => INFO,
            SpecialTabKind::Welcome => SPARKLE,
        }
    }
}

/// State for an image viewer tab.
#[derive(Debug, Clone, PartialEq)]
pub struct ImageViewerState {
    /// Current zoom level (1.0 = original size, fit-to-window by default)
    pub zoom: f32,
    /// Image dimensions (width, height) — populated after first load
    pub dimensions: Option<(u32, u32)>,
    /// Image file size in bytes
    pub file_size: u64,
    /// Image format string (e.g., "PNG", "JPEG")
    pub format_label: String,
    /// Whether initial fit-to-window has been applied
    pub fitted: bool,
}

impl Default for ImageViewerState {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            dimensions: None,
            file_size: 0,
            format_label: String::new(),
            fitted: false,
        }
    }
}

/// State for a PDF viewer tab.
#[derive(Debug, Clone, PartialEq)]
pub struct PdfViewerState {
    /// Current page index (0-based)
    pub current_page: usize,
    /// Total number of pages
    pub page_count: usize,
    /// Current zoom level (1.0 = fit-to-window)
    pub zoom: f32,
    /// Whether initial fit-to-window has been applied
    pub fitted: bool,
    /// File size in bytes
    pub file_size: u64,
    /// Error message if PDF failed to load
    pub error: Option<String>,
    /// Overrides the tab label (used for print preview).
    pub display_title: Option<String>,
    /// When true, the file at [`Tab::path`] is deleted when this tab closes and the tab is not saved in session snapshots.
    pub ephemeral_temp_file: bool,
}

impl Default for PdfViewerState {
    fn default() -> Self {
        Self {
            current_page: 0,
            page_count: 0,
            zoom: 1.0,
            fitted: false,
            file_size: 0,
            error: None,
            display_title: None,
            ephemeral_temp_file: false,
        }
    }
}

/// Progress state for background file loading.
#[derive(Debug, Clone)]
pub struct LoadingProgress {
    /// File path being loaded
    pub path: PathBuf,
    /// Bytes loaded so far
    pub bytes_loaded: u64,
    /// Total file size in bytes
    pub total_size: u64,
}

impl LoadingProgress {
    /// Progress as a fraction (0.0 to 1.0).
    pub fn fraction(&self) -> f32 {
        if self.total_size == 0 {
            0.0
        } else {
            (self.bytes_loaded as f64 / self.total_size as f64) as f32
        }
    }

    /// Bytes loaded in megabytes.
    pub fn mb_loaded(&self) -> f64 {
        self.bytes_loaded as f64 / (1024.0 * 1024.0)
    }

    /// Total size in megabytes.
    pub fn mb_total(&self) -> f64 {
        self.total_size as f64 / (1024.0 * 1024.0)
    }
}

/// Content state for a tab — either loading in background or fully loaded.
#[derive(Debug, Clone)]
pub enum TabContent {
    /// File is being loaded in background thread.
    Loading(LoadingProgress),
    /// Content is fully loaded and ready for editing.
    Ready,
    /// File loading failed with an error message.
    Error(String),
}

impl Default for TabContent {
    fn default() -> Self {
        TabContent::Ready
    }
}

/// The kind of content a tab holds.
#[derive(Debug, Clone, PartialEq)]
pub enum TabKind {
    /// Regular document tab (file editing)
    Document,
    /// Special non-editable tab (settings, about, etc.)
    Special(SpecialTabKind),
    /// Image viewer tab (read-only image display with zoom)
    ImageViewer(ImageViewerState),
    /// PDF viewer tab (read-only PDF display with page navigation)
    PdfViewer(PdfViewerState),
}

impl Default for TabKind {
    fn default() -> Self {
        TabKind::Document
    }
}

///
/// This struct holds the complete state of an open document tab,
/// including content and editing state. Different from `TabInfo` which
/// is used for persistence/session restoration.
/// Owns the document body, revision and on-disk identity.
///
/// `Tab` dereferences to this type, so existing `tab.content` /
/// `tab.path` / `tab.content_version` access continues to work while the
/// body has a single owner.
#[derive(Debug, Clone)]
pub struct Document {
    /// File path (None for untitled documents).
    pub(crate) path: Option<PathBuf>,
    /// Current text.
    pub(crate) content: String,
    /// Original content for change detection (small files).
    pub(crate) original_content: String,
    /// Hash of original content (large files).
    pub(crate) original_content_hash: Option<u64>,
    /// Whether the file crossed `LARGE_FILE_THRESHOLD`.
    pub(crate) is_large_file: bool,
    /// Operation-based undo/redo history.
    pub(crate) edit_history: EditHistory,
    /// Monotonic content revision.
    pub(crate) content_version: u64,
    /// External-invalidation epoch for stable rendered ids.
    pub(crate) source_epoch: u64,
    /// File type inferred from the path.
    pub(crate) file_type: FileType,
    /// Hash at last auto-save.
    pub(crate) last_auto_save_content_hash: Option<u64>,
    /// Encoding detected at open time.
    pub(crate) detected_encoding: Option<&'static str>,
    /// Original bytes for re-decoding.
    pub(crate) original_bytes: Vec<u8>,
    /// Encoding used for saves.
    pub(crate) current_encoding: &'static str,
    /// Whether the file had a BOM.
    pub(crate) had_bom: bool,
    /// Pending diff snapshot for rendered-mode undo.
    pub(crate) pending_undo_snapshot: Option<String>,
    /// Revision at which `pending_undo_snapshot` was refreshed.
    pub(crate) undo_snapshot_version: u64,
    /// Cached text statistics.
    pub(crate) cached_text_stats: TextStats,
    pub(crate) cached_text_stats_version: u64,
    /// Cached modification state.
    pub(crate) cached_is_modified: bool,
    pub(crate) cached_is_modified_version: u64,
    /// Save counter for is_modified invalidation.
    pub(crate) save_version: u64,
    pub(crate) cached_is_modified_save_version: u64,
    /// CJK detection caches.
    pub(crate) cached_needs_cjk: bool,
    pub(crate) cached_needs_cjk_version: u64,
    pub(crate) cached_needs_complex_script: bool,
    pub(crate) cached_needs_complex_script_version: u64,
    /// Revision at last auto-save.
    pub(crate) last_auto_save_content_version: Option<u64>,
    /// Revision written to crash recovery.
    pub(crate) last_recovery_version: Option<u64>,
    /// Full-content hash cache keyed by revision.
    pub(crate) cached_content_hash: std::cell::Cell<Option<(u64, u64)>>,
}

#[derive(Debug, Clone)]
pub struct Tab {
    /// Unique identifier for this tab
    pub id: usize,
    /// Kind of tab (document or special panel)
    pub kind: TabKind,
    /// Loading state: whether content is still being loaded from disk.
    pub tab_content: TabContent,
    /// Document body, revision and disk identity.
    pub(crate) document: Document,
    /// Optional tab title for pathless documents (session-persisted quick notes).
    pub untitled_display_name: Option<String>,
    /// Multi-cursor state (supports multiple selections/cursors)
    pub cursors: MultiCursor,
    /// Legacy: Cursor position (line, column) - 0-indexed.
    /// Computed from primary cursor, kept for backwards compatibility.
    pub cursor_position: (usize, usize),
    /// Legacy: Text selection range (start_char_index, end_char_index) - None if no selection.
    /// Computed from primary cursor, kept for backwards compatibility.
    pub selection: Option<(usize, usize)>,
    /// Scroll offset in the editor
    pub scroll_offset: f32,
    /// Total content height inside the scroll area (for sync scrolling)
    pub content_height: f32,
    /// Viewport height of the scroll area (for sync scrolling)
    pub viewport_height: f32,
    /// Pending scroll offset to apply on next render (for sync scrolling on mode switch)
    pub pending_scroll_offset: Option<f32>,
    /// Pending cursor position to restore on next render (for undo/redo)
    /// When Some, the editor widget will restore cursor to this char index
    pub pending_cursor_restore: Option<usize>,
    /// Pending scroll ratio to apply (0.0 to 1.0) - used when content_height is unknown
    pub pending_scroll_ratio: Option<f32>,
    /// Line-to-Y mappings from last rendered mode render (for scroll sync)
    /// Vec of (start_line, end_line, rendered_y)
    pub rendered_line_mappings: Vec<(usize, usize, f32)>,
    /// Actual line height in Raw mode (for accurate scroll sync)
    pub raw_line_height: f32,
    /// Pending target line to scroll to (for sync scrolling, used with line mappings)
    pub pending_scroll_to_line: Option<usize>,
    /// Pending scroll anchor for wrap-aware raw scroll (1-indexed line, fraction within line)
    pub pending_scroll_anchor: Option<(usize, f32)>,
    /// Skip cursor sync from editor on next frame (set when navigating from outline/minimap)
    pub skip_cursor_sync: bool,
    /// View mode for this tab (raw or rendered)
    pub view_mode: ViewMode,
    /// Whether the editor should request focus on next frame
    pub needs_focus: bool,
    /// Transient highlight for search result navigation (not persisted).
    pub transient_highlight: TransientHighlight,
    /// Whether auto-save is enabled for this tab (per-tab toggle)
    pub auto_save_enabled: bool,
    /// Time of last content edit (for idle-based auto-save scheduling)
    pub last_edit_time: Option<std::time::Instant>,
    /// Fold state for code folding
    pub fold_state: FoldState,
    /// Split view ratio (0.0 to 1.0, proportion of width for left pane)
    /// Default is 0.5 (50/50 split). Only used when view_mode is Split.
    pub split_ratio: f32,
    /// Live Pipeline state for this tab (JSON/YAML command piping)
    pub pipeline_state: TabPipelineState,
}

impl std::ops::Deref for Tab {
    type Target = Document;
    fn deref(&self) -> &Document {
        &self.document
    }
}

impl std::ops::DerefMut for Tab {
    fn deref_mut(&mut self) -> &mut Document {
        &mut self.document
    }
}

impl Tab {
    /// Clear programmatic scroll targets from split/mode sync (not app-level outline nav).
    pub fn clear_sync_pending_scroll(&mut self) {
        self.pending_scroll_offset = None;
        self.pending_scroll_anchor = None;
        self.pending_scroll_ratio = None;
        self.pending_scroll_to_line = None;
    }

    /// Compute a 64-bit hash of content for modification detection.
    pub(crate) fn compute_content_hash(content: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    }

    /// Create a new empty tab.
    ///
    /// New tabs default to Raw view mode and Markdown file type.
    /// The editor will automatically receive focus on the next frame.
    pub fn new(id: usize) -> Self {
        Self {
        document: Document {
            path: None,
            content: String::new(),
            original_content: String::new(),
            original_content_hash: None,
            is_large_file: false,
            edit_history: EditHistory::new(),
            content_version: 0,
            source_epoch: 0,
            file_type: FileType::Markdown, // New tabs default to markdown
            last_auto_save_content_hash: None,
            detected_encoding: None, // New documents have no detected encoding
            original_bytes: Vec::new(), // No original bytes for new docs
            current_encoding: "utf-8", // Default to UTF-8 for new documents
            had_bom: false,          // New documents don't have a BOM
            pending_undo_snapshot: None,
            undo_snapshot_version: 0,
            cached_text_stats: TextStats::default(),
            cached_text_stats_version: u64::MAX,
            cached_is_modified: false,
            cached_is_modified_version: u64::MAX,
            save_version: 0,
            cached_is_modified_save_version: u64::MAX,
            cached_needs_cjk: false,
            cached_needs_cjk_version: u64::MAX,
            cached_needs_complex_script: false,
            cached_needs_complex_script_version: u64::MAX,
            last_auto_save_content_version: None,
            last_recovery_version: None,
            cached_content_hash: std::cell::Cell::new(None),
        },
            id,
            kind: TabKind::Document,
            tab_content: TabContent::Ready,
            untitled_display_name: None,
            cursors: MultiCursor::new(),
            cursor_position: (0, 0),
            selection: None,
            scroll_offset: 0.0,
            content_height: 0.0,
            viewport_height: 0.0,
            pending_scroll_offset: None,
            pending_cursor_restore: None,
            pending_scroll_ratio: None,
            rendered_line_mappings: Vec::new(),
            raw_line_height: 20.0, // Default, updated on first render
            pending_scroll_to_line: None,
            pending_scroll_anchor: None,
            skip_cursor_sync: false,
            view_mode: ViewMode::Raw, // New documents default to raw mode
            needs_focus: true,             // Auto-focus new tabs
            transient_highlight: TransientHighlight::new(),
            auto_save_enabled: false, // Will be set from settings by caller
            last_edit_time: None,
            fold_state: FoldState::new(),
            split_ratio: 0.5, // Default to 50/50 split
            pipeline_state: TabPipelineState::default(),
        }
    }

    /// Create a new empty tab with settings-based defaults.
    ///
    /// # Arguments
    /// * `id` - Unique tab identifier
    /// * `auto_save_default` - Whether auto-save is enabled by default
    /// * `default_view_mode` - Default view mode for new tabs (Raw, Rendered, or Split)
    pub fn new_with_settings(
        id: usize,
        auto_save_default: bool,
        default_view_mode: ViewMode,
    ) -> Self {
        let mut tab = Self::new(id);
        tab.auto_save_enabled = auto_save_default;
        tab.view_mode = default_view_mode;
        tab
    }

    /// Create a tab with content from a file.
    ///
    /// Newly opened files default to Raw view mode.
    /// File type is detected from the path extension.
    /// The editor will automatically receive focus on the next frame.
    pub fn with_file(id: usize, path: PathBuf, content: String) -> Self {
        let file_type = FileType::from_path(&path);
        let is_large_file = content.len() >= LARGE_FILE_THRESHOLD;

        // For large files, store hash instead of full content to save memory
        let (original_content, original_content_hash) = if is_large_file {
            log::info!(
                "Opening large file ({} bytes): using hash-based modification detection",
                content.len()
            );
            (String::new(), Some(Self::compute_content_hash(&content)))
        } else {
            (content.clone(), None)
        };

        let edit_history = if is_large_file {
            EditHistory::with_max_groups(LARGE_FILE_MAX_UNDO_GROUPS)
        } else {
            EditHistory::new()
        };

        Self {
        document: Document {
            path: Some(path),
            content_version: 0,
            source_epoch: 0,
            last_auto_save_content_hash: None,
            detected_encoding: Some("utf-8"),
            original_bytes: Vec::new(),
            current_encoding: "utf-8",
            had_bom: false,
            pending_undo_snapshot: None,
            undo_snapshot_version: 0,
            cached_text_stats: TextStats::default(),
            cached_text_stats_version: u64::MAX,
            cached_is_modified: false,
            cached_is_modified_version: u64::MAX,
            save_version: 0,
            cached_is_modified_save_version: u64::MAX,
            cached_needs_cjk: false,
            cached_needs_cjk_version: u64::MAX,
            cached_needs_complex_script: false,
            cached_needs_complex_script_version: u64::MAX,
            last_auto_save_content_version: None,
            last_recovery_version: None,
            cached_content_hash: std::cell::Cell::new(None),
            content,
            original_content,
            original_content_hash,
            is_large_file,
            edit_history,
            file_type,
        },
            id,
            kind: TabKind::Document,
            tab_content: TabContent::Ready,
            untitled_display_name: None,
            cursors: MultiCursor::new(),
            cursor_position: (0, 0),
            selection: None,
            scroll_offset: 0.0,
            content_height: 0.0,
            viewport_height: 0.0,
            pending_scroll_offset: None,
            pending_cursor_restore: None,
            pending_scroll_ratio: None,
            rendered_line_mappings: Vec::new(),
            raw_line_height: 20.0,
            pending_scroll_to_line: None,
            pending_scroll_anchor: None,
            skip_cursor_sync: false,
            view_mode: ViewMode::Raw,
            needs_focus: true,
            transient_highlight: TransientHighlight::new(),
            auto_save_enabled: false,
            last_edit_time: None,
            fold_state: FoldState::new(),
            split_ratio: 0.5,
            pipeline_state: TabPipelineState::default(),
        }
    }

    /// Create a tab with content loaded from file bytes with automatic encoding detection.
    ///
    /// Uses chardetng for encoding detection and encoding_rs for decoding.
    /// For large files (>1MB), uses hash-based modification detection to save memory.
    pub fn with_file_bytes(id: usize, path: PathBuf, bytes: Vec<u8>) -> Self {
        use chardetng::EncodingDetector;

        let file_type = FileType::from_path(&path);
        let bytes_len = bytes.len();
        let is_large_file = bytes_len >= LARGE_FILE_THRESHOLD;

        // Detect encoding using chardetng
        let mut detector = EncodingDetector::new();
        detector.feed(&bytes, true);
        let detected = detector.guess(None, true);
        let encoding_label = detected.name();

        // Check for BOM first - encoding_rs handles this
        let (content, actual_encoding, _had_errors, had_bom) =
            if let Some((bom_encoding, bom_len)) = encoding_rs::Encoding::for_bom(&bytes) {
                // BOM detected, use that encoding and skip BOM bytes
                // Use decode_without_bom_handling since we already handled the BOM
                let (decoded, had_errors) =
                    bom_encoding.decode_without_bom_handling(&bytes[bom_len..]);
                (decoded.into_owned(), bom_encoding.name(), had_errors, true)
            } else {
                // No BOM, use detected encoding
                let (decoded, _, had_errors) = detected.decode(&bytes);
                (decoded.into_owned(), encoding_label, had_errors, false)
            };

        let (original_content, original_content_hash, original_bytes) = if is_large_file {
            log::info!(
                "Opening large file ({} bytes): using hash-based modification detection",
                bytes_len
            );
            (
                String::new(),
                Some(Self::compute_content_hash(&content)),
                Vec::new(),
            )
        } else {
            (content.clone(), None, bytes)
        };

        let edit_history = if is_large_file {
            EditHistory::with_max_groups(LARGE_FILE_MAX_UNDO_GROUPS)
        } else {
            EditHistory::new()
        };

        Self {
        document: Document {
            path: Some(path),
            content_version: 0,
            source_epoch: 0,
            last_auto_save_content_hash: None,
            detected_encoding: Some(actual_encoding),
            current_encoding: actual_encoding,
            pending_undo_snapshot: None,
            undo_snapshot_version: 0,
            cached_text_stats: TextStats::default(),
            cached_text_stats_version: u64::MAX,
            cached_is_modified: false,
            cached_is_modified_version: u64::MAX,
            save_version: 0,
            cached_is_modified_save_version: u64::MAX,
            cached_needs_cjk: false,
            cached_needs_cjk_version: u64::MAX,
            cached_needs_complex_script: false,
            cached_needs_complex_script_version: u64::MAX,
            last_auto_save_content_version: None,
            last_recovery_version: None,
            cached_content_hash: std::cell::Cell::new(None),
            had_bom,
            original_bytes,
            content,
            original_content,
            original_content_hash,
            is_large_file,
            edit_history,
            file_type,
        },
            id,
            kind: TabKind::Document,
            tab_content: TabContent::Ready,
            untitled_display_name: None,
            cursors: MultiCursor::new(),
            cursor_position: (0, 0),
            selection: None,
            scroll_offset: 0.0,
            content_height: 0.0,
            viewport_height: 0.0,
            pending_scroll_offset: None,
            pending_cursor_restore: None,
            pending_scroll_ratio: None,
            rendered_line_mappings: Vec::new(),
            raw_line_height: 20.0,
            pending_scroll_to_line: None,
            pending_scroll_anchor: None,
            skip_cursor_sync: false,
            view_mode: ViewMode::Raw,
            needs_focus: true,
            transient_highlight: TransientHighlight::new(),
            auto_save_enabled: false,
            last_edit_time: None,
            fold_state: FoldState::new(),
            split_ratio: 0.5,
            pipeline_state: TabPipelineState::default(),
        }
    }

    /// Create a tab with content from a file, with settings-based defaults.
    ///
    /// # Arguments
    /// * `id` - Unique tab identifier
    /// * `path` - File path
    /// * `content` - File content
    /// * `auto_save_default` - Whether auto-save is enabled by default
    /// * `default_view_mode` - Default view mode for new tabs (Raw, Rendered, or Split)
    pub fn with_file_and_settings(
        id: usize,
        path: PathBuf,
        content: String,
        auto_save_default: bool,
        default_view_mode: ViewMode,
    ) -> Self {
        let mut tab = Self::with_file(id, path, content);
        tab.auto_save_enabled = auto_save_default;
        tab.view_mode = default_view_mode;
        tab
    }

    /// Create a tab from file bytes with encoding detection and settings.
    ///
    /// # Arguments
    /// * `id` - Unique tab identifier
    /// * `path` - File path
    /// * `bytes` - Raw file bytes for encoding detection
    /// * `auto_save_default` - Whether auto-save is enabled by default
    /// * `default_view_mode` - Default view mode for new tabs (Raw, Rendered, or Split)
    pub fn with_file_bytes_and_settings(
        id: usize,
        path: PathBuf,
        bytes: Vec<u8>,
        auto_save_default: bool,
        default_view_mode: ViewMode,
    ) -> Self {
        let mut tab = Self::with_file_bytes(id, path, bytes);
        tab.auto_save_enabled = auto_save_default;
        tab.view_mode = default_view_mode;
        tab
    }

    /// Create a tab from saved session info.
    ///
    /// Restores the view mode and split ratio from the saved session.
    /// File type is detected from the path extension.
    /// Restored tabs don't auto-focus since we're restoring previous state.
    pub fn from_tab_info(id: usize, info: &TabInfo, content: String) -> Self {
        let file_type = info
            .path
            .as_ref()
            .map(|p| FileType::from_path(p))
            .unwrap_or(FileType::Markdown);
        // Convert legacy cursor position to char index for MultiCursor
        let cursor_char_idx =
            line_col_to_char_index(&content, info.cursor_position.0, info.cursor_position.1);

        let is_large_file = content.len() >= LARGE_FILE_THRESHOLD;
        let (original_content, original_content_hash) = if is_large_file {
            (String::new(), Some(Self::compute_content_hash(&content)))
        } else {
            (content.clone(), None)
        };

        let edit_history = if is_large_file {
            EditHistory::with_max_groups(LARGE_FILE_MAX_UNDO_GROUPS)
        } else {
            EditHistory::new()
        };

        Self {
        document: Document {
            path: info.path.clone(),
            content_version: 0,
            source_epoch: 0,
            last_auto_save_content_hash: None,
            detected_encoding: Some("utf-8"),
            original_bytes: Vec::new(),
            current_encoding: "utf-8",
            had_bom: false,
            pending_undo_snapshot: None,
            undo_snapshot_version: 0,
            cached_text_stats: TextStats::default(),
            cached_text_stats_version: u64::MAX,
            cached_is_modified: false,
            cached_is_modified_version: u64::MAX,
            save_version: 0,
            cached_is_modified_save_version: u64::MAX,
            cached_needs_cjk: false,
            cached_needs_cjk_version: u64::MAX,
            cached_needs_complex_script: false,
            cached_needs_complex_script_version: u64::MAX,
            last_auto_save_content_version: None,
            last_recovery_version: None,
            cached_content_hash: std::cell::Cell::new(None),
            content,
            original_content,
            original_content_hash,
            is_large_file,
            edit_history,
            file_type,
        },
            id,
            kind: TabKind::Document,
            tab_content: TabContent::Ready,
            untitled_display_name: None,
            cursors: MultiCursor::single(cursor_char_idx),
            cursor_position: info.cursor_position,
            selection: None,
            scroll_offset: info.scroll_offset,
            content_height: 0.0,
            viewport_height: 0.0,
            pending_scroll_offset: None,
            pending_cursor_restore: None,
            pending_scroll_ratio: None,
            rendered_line_mappings: Vec::new(),
            raw_line_height: 20.0,
            pending_scroll_to_line: None,
            pending_scroll_anchor: None,
            skip_cursor_sync: false,
            view_mode: info.view_mode,
            needs_focus: false,
            transient_highlight: TransientHighlight::new(),
            auto_save_enabled: false,
            last_edit_time: None,
            fold_state: FoldState::new(),
            split_ratio: info.split_ratio,
            pipeline_state: TabPipelineState::default(),
        }
    }

    /// Create a tab from session info with settings-based auto-save.
    pub fn from_tab_info_with_settings(
        id: usize,
        info: &TabInfo,
        content: String,
        auto_save_default: bool,
    ) -> Self {
        let mut tab = Self::from_tab_info(id, info, content);
        tab.auto_save_enabled = auto_save_default;
        tab
    }

    /// Create a tab from session info using raw file bytes with encoding detection.
    ///
    /// This combines tab info restoration (view mode, split ratio) with
    /// automatic encoding detection from the file bytes.
    /// For large files, uses hash-based modification detection.
    pub fn from_tab_info_with_bytes(
        id: usize,
        info: &TabInfo,
        bytes: Vec<u8>,
        auto_save_default: bool,
    ) -> Self {
        use chardetng::EncodingDetector;

        let file_type = info
            .path
            .as_ref()
            .map(|p| FileType::from_path(p))
            .unwrap_or(FileType::Markdown);

        let bytes_len = bytes.len();
        let is_large_file = bytes_len >= LARGE_FILE_THRESHOLD;

        // Detect encoding
        let mut detector = EncodingDetector::new();
        detector.feed(&bytes, true);
        let detected = detector.guess(None, true);

        // Check for BOM first
        let (content, actual_encoding, had_bom) =
            if let Some((bom_encoding, bom_len)) = encoding_rs::Encoding::for_bom(&bytes) {
                // Use decode_without_bom_handling since we already handled the BOM
                let (decoded, _had_errors) =
                    bom_encoding.decode_without_bom_handling(&bytes[bom_len..]);
                (decoded.into_owned(), bom_encoding.name(), true)
            } else {
                let (decoded, _, _) = detected.decode(&bytes);
                (decoded.into_owned(), detected.name(), false)
            };

        // Convert legacy cursor position to char index
        let cursor_char_idx =
            line_col_to_char_index(&content, info.cursor_position.0, info.cursor_position.1);

        let (original_content, original_content_hash, original_bytes) = if is_large_file {
            log::info!(
                "Restoring large file ({} bytes): using hash-based modification detection",
                bytes_len
            );
            (
                String::new(),
                Some(Self::compute_content_hash(&content)),
                Vec::new(),
            )
        } else {
            (content.clone(), None, bytes)
        };

        let edit_history = if is_large_file {
            EditHistory::with_max_groups(LARGE_FILE_MAX_UNDO_GROUPS)
        } else {
            EditHistory::new()
        };

        Self {
        document: Document {
            path: info.path.clone(),
            content_version: 0,
            source_epoch: 0,
            last_auto_save_content_hash: None,
            detected_encoding: Some(actual_encoding),
            current_encoding: actual_encoding,
            pending_undo_snapshot: None,
            undo_snapshot_version: 0,
            cached_text_stats: TextStats::default(),
            cached_text_stats_version: u64::MAX,
            cached_is_modified: false,
            cached_is_modified_version: u64::MAX,
            save_version: 0,
            cached_is_modified_save_version: u64::MAX,
            cached_needs_cjk: false,
            cached_needs_cjk_version: u64::MAX,
            cached_needs_complex_script: false,
            cached_needs_complex_script_version: u64::MAX,
            last_auto_save_content_version: None,
            last_recovery_version: None,
            cached_content_hash: std::cell::Cell::new(None),
            had_bom,
            original_bytes,
            content,
            original_content,
            original_content_hash,
            is_large_file,
            edit_history,
            file_type,
        },
            id,
            kind: TabKind::Document,
            tab_content: TabContent::Ready,
            untitled_display_name: None,
            cursors: MultiCursor::single(cursor_char_idx),
            cursor_position: info.cursor_position,
            selection: None,
            scroll_offset: info.scroll_offset,
            content_height: 0.0,
            viewport_height: 0.0,
            pending_scroll_offset: None,
            pending_cursor_restore: None,
            pending_scroll_ratio: None,
            rendered_line_mappings: Vec::new(),
            raw_line_height: 20.0,
            pending_scroll_to_line: None,
            pending_scroll_anchor: None,
            skip_cursor_sync: false,
            view_mode: info.view_mode,
            needs_focus: false,
            transient_highlight: TransientHighlight::new(),
            auto_save_enabled: auto_save_default,
            last_edit_time: None,
            fold_state: FoldState::new(),
            split_ratio: info.split_ratio,
            pipeline_state: TabPipelineState::default(),
        }
    }

    /// Create a placeholder tab for a file that is being loaded in the background.
    ///
    /// The tab is immediately added and visible but renders a progress indicator
    /// instead of editor content until loading completes.
    pub fn new_loading(id: usize, path: PathBuf, total_size: u64) -> Self {
        let file_type = FileType::from_path(&path);
        let mut tab = Self::new(id);
        tab.tab_content = TabContent::Loading(LoadingProgress {
            path: path.clone(),
            bytes_loaded: 0,
            total_size,
        });
        tab.path = Some(path);
        tab.file_type = file_type;
        tab.needs_focus = true;
        tab
    }

    /// Finalize background loading: populate tab with decoded content and transition to Ready.
    ///
    /// This is called from the main thread when the background reader sends the complete bytes.
    pub fn finish_loading(
        &mut self,
        bytes: Vec<u8>,
        auto_save_default: bool,
        default_view_mode: crate::config::ViewMode,
    ) {
        use chardetng::EncodingDetector;

        let bytes_len = bytes.len();
        let is_large_file = bytes_len >= LARGE_FILE_THRESHOLD;

        let mut detector = EncodingDetector::new();
        detector.feed(&bytes, true);
        let detected = detector.guess(None, true);

        let (content, actual_encoding, had_bom) =
            if let Some((bom_encoding, bom_len)) = encoding_rs::Encoding::for_bom(&bytes) {
                let (decoded, _had_errors) =
                    bom_encoding.decode_without_bom_handling(&bytes[bom_len..]);
                (decoded.into_owned(), bom_encoding.name(), true)
            } else {
                let (decoded, _, _) = detected.decode(&bytes);
                (decoded.into_owned(), detected.name(), false)
            };

        let (original_content, original_content_hash, original_bytes) = if is_large_file {
            log::info!(
                "Background load complete ({} bytes): using hash-based modification detection",
                bytes_len
            );
            (
                String::new(),
                Some(Self::compute_content_hash(&content)),
                Vec::new(),
            )
        } else {
            (content.clone(), None, bytes)
        };

        self.content = content;
        self.original_content = original_content;
        self.original_content_hash = original_content_hash;
        self.is_large_file = is_large_file;
        self.original_bytes = original_bytes;
        self.detected_encoding = Some(actual_encoding);
        self.current_encoding = actual_encoding;
        self.had_bom = had_bom;
        self.auto_save_enabled = auto_save_default;
        self.view_mode = default_view_mode;
        self.edit_history = if is_large_file {
            EditHistory::with_max_groups(LARGE_FILE_MAX_UNDO_GROUPS)
        } else {
            EditHistory::new()
        };
        self.tab_content = TabContent::Ready;
        self.content_version = self.content_version.wrapping_add(1);
        self.bump_source_epoch();
    }

    /// Mark this tab's loading as failed with an error message.
    pub fn fail_loading(&mut self, error: String) {
        self.tab_content = TabContent::Error(error);
    }

    /// Check if the tab has unsaved changes (cached via content_version + save_version).
    ///
    /// For large files (>1MB), uses hash comparison instead of full string comparison
    /// to avoid storing a full copy of the original content.
    /// The result is cached and only recomputed when content or save state changes.
    pub fn is_modified(&self) -> bool {
        if self.cached_is_modified_version == self.content_version
            && self.cached_is_modified_save_version == self.save_version
        {
            return self.cached_is_modified;
        }
        self.is_modified_uncached()
    }

    /// Recompute is_modified and update cache (call from &mut self contexts).
    pub fn is_modified_cached(&mut self) -> bool {
        if self.cached_is_modified_version == self.content_version
            && self.cached_is_modified_save_version == self.save_version
        {
            return self.cached_is_modified;
        }
        let result = self.is_modified_uncached();
        self.cached_is_modified = result;
        self.cached_is_modified_version = self.content_version;
        self.cached_is_modified_save_version = self.save_version;
        result
    }

    fn is_modified_uncached(&self) -> bool {
        if self.is_special() {
            return false;
        }
        if let Some(hash) = self.original_content_hash {
            Self::compute_content_hash(&self.content) != hash
        } else {
            self.content != self.original_content
        }
    }

    /// Cached text statistics (word/char/line counts), recomputed only when content changes.
    pub fn text_stats(&mut self) -> TextStats {
        if self.cached_text_stats_version == self.content_version {
            return self.cached_text_stats;
        }
        self.cached_text_stats = TextStats::from_text(&self.content);
        self.cached_text_stats_version = self.content_version;
        self.cached_text_stats
    }

    /// Whether content contains CJK characters (cached via content_version).
    pub fn needs_cjk_cached(&mut self) -> bool {
        if self.cached_needs_cjk_version == self.content_version {
            return self.cached_needs_cjk;
        }
        self.cached_needs_cjk = crate::fonts::needs_cjk(&self.content);
        self.cached_needs_cjk_version = self.content_version;
        self.cached_needs_cjk
    }

    /// Whether content contains complex script characters (cached via content_version).
    pub fn needs_complex_script_cached(&mut self) -> bool {
        if self.cached_needs_complex_script_version == self.content_version {
            return self.cached_needs_complex_script;
        }
        self.cached_needs_complex_script = crate::fonts::needs_complex_script_fonts(&self.content);
        self.cached_needs_complex_script_version = self.content_version;
        self.cached_needs_complex_script
    }

    /// Check if this is a large file that uses memory-optimized storage.
    pub fn is_large_file(&self) -> bool {
        self.is_large_file
    }

    /// Hash of the disk content this tab was last loaded from (or last saved to).
    ///
    /// Used by session recovery (`RecoveryContent::original_content_hash`) and
    /// autosave identity checks to verify that a recovery file still belongs to
    /// the same on-disk file the tab was opened with — independent of whether
    /// the in-memory buffer has diverged.
    ///
    /// - Large files: returns the cached `original_content_hash` (computed at
    ///   load / `mark_saved`).
    /// - Small files (where `original_content` holds the disk text): hashes
    ///   `original_content` on the fly with the same `DefaultHasher` algorithm
    ///   used by `crate::config::session::hash_content`, so the value is
    ///   directly comparable to a hash of current disk content.
    /// - Untitled tabs that have never been written to disk: returns `None`.
    pub fn disk_content_hash(&self) -> Option<u64> {
        if let Some(hash) = self.original_content_hash {
            return Some(hash);
        }
        if self.path.is_none() && self.original_content.is_empty() {
            return None;
        }
        Some(Self::compute_content_hash(&self.original_content))
    }

    /// Check if this is a new/untitled file (not yet saved to disk).
    ///
    /// Returns `true` if the tab has no associated file path, meaning it was
    /// created in the app and has never been saved. This is distinct from
    /// files that were loaded from disk (even if they're empty).
    pub fn is_new_file(&self) -> bool {
        self.path.is_none()
    }

    /// Check if this is an unmodified empty untitled file.
    ///
    /// Returns `true` if:
    /// - The tab is a new file (no path, never saved)
    /// - The content matches the initial empty state
    ///
    /// These files can be closed without prompting to save since there's
    /// nothing meaningful to preserve.
    pub fn is_empty_untitled(&self) -> bool {
        self.is_new_file() && self.content.is_empty() && self.original_content.is_empty()
    }

    /// Determine if we should prompt to save before closing or exiting.
    ///
    /// The logic is:
    /// - If the file is modified (content differs from original), prompt to save
    /// - EXCEPTION: Skip prompt for empty untitled files (nothing to save)
    /// - EXCEPTION: When **Quick note workflow** is enabled, skip prompts for
    ///   pathless documents on **app exit** only; closing an individual untitled
    ///   tab with content still prompts. Session recovery preserves scratch buffers.
    pub fn should_prompt_to_save(&self, settings: &Settings, context: SavePromptContext) -> bool {
        // Special tabs, image viewer tabs, PDF viewer tabs, and loading tabs never need to save
        if self.is_special()
            || self.is_image_viewer()
            || self.is_pdf_viewer()
            || self.is_loading()
            || self.is_load_error()
        {
            return false;
        }

        if settings.quick_note_workflow
            && self.is_new_file()
            && context == SavePromptContext::AppExit
        {
            return false;
        }

        // Don't prompt for unmodified files
        if !self.is_modified() {
            return false;
        }

        // Don't prompt for empty untitled files (nothing meaningful to save)
        // This handles the case where content was typed and then deleted
        if self.is_new_file() && self.content.is_empty() {
            return false;
        }

        // All other modified files should prompt
        true
    }

    /// Label used for session persistence and crash recovery metadata (no `*` suffix).
    pub fn persisted_session_display_title(&self) -> String {
        if self.is_special() || self.is_image_viewer() || self.is_pdf_viewer() {
            return self.title();
        }
        if let Some(path) = &self.path {
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Untitled")
                .to_string()
        } else {
            self.untitled_display_name
                .clone()
                .unwrap_or_else(|| "Untitled".to_string())
        }
    }

    /// Initial text for the rename-untitled dialog.
    pub fn untitled_rename_buffer_initial(&self) -> String {
        self.untitled_display_name
            .clone()
            .unwrap_or_else(|| "Untitled".to_string())
    }

    /// Get the display title for this tab.
    pub fn title(&self) -> String {
        if let TabKind::Special(special) = &self.kind {
            return format!("{} {}", special.icon(), special.title());
        }

        if matches!(&self.kind, TabKind::ImageViewer(_)) {
            let name = self
                .path
                .as_ref()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("Image");
            return format!("\u{1F5BC} {}", name); // framed picture emoji
        }

        if let TabKind::PdfViewer(vs) = &self.kind {
            if let Some(title) = &vs.display_title {
                return format!("\u{1F4C4} {}", title);
            }
            let name = self
                .path
                .as_ref()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("PDF");
            return format!("\u{1F4C4} {}", name); // page facing up emoji
        }

        let name: String = if let Some(path) = &self.path {
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Untitled")
                .to_string()
        } else {
            self.untitled_display_name
                .clone()
                .unwrap_or_else(|| "Untitled".to_string())
        };

        if self.is_loading() {
            return format!("\u{23F3} {}", name); // hourglass
        }

        if self.is_load_error() {
            return format!("\u{26A0} {}", name); // warning sign
        }

        if self.is_modified() {
            // MarkText uses a dot rather than an asterisk for unsaved changes.
            format!("{} ●", name)
        } else {
            name
        }
    }

    /// Check if this tab is a special (non-editable) tab.
    pub fn is_special(&self) -> bool {
        matches!(self.kind, TabKind::Special(_))
    }

    /// Check if this tab is an image viewer tab.
    pub fn is_image_viewer(&self) -> bool {
        matches!(self.kind, TabKind::ImageViewer(_))
    }

    /// Check if this tab is a PDF viewer tab.
    pub fn is_pdf_viewer(&self) -> bool {
        matches!(self.kind, TabKind::PdfViewer(_))
    }

    /// Check if this tab is currently loading file content from disk.
    pub fn is_loading(&self) -> bool {
        matches!(self.tab_content, TabContent::Loading(_))
    }

    /// Check if this tab had a loading error.
    pub fn is_load_error(&self) -> bool {
        matches!(self.tab_content, TabContent::Error(_))
    }

    /// Get loading progress if this tab is currently loading.
    pub fn loading_progress(&self) -> Option<&LoadingProgress> {
        if let TabContent::Loading(ref progress) = self.tab_content {
            Some(progress)
        } else {
            None
        }
    }

    /// Mark the current content as saved (updates original_content or hash).
    /// Also clears auto-save state since content is now persisted.
    pub fn mark_saved(&mut self) {
        if self.is_large_file {
            self.original_content_hash = Some(Self::compute_content_hash(&self.content));
        } else {
            self.original_content = self.content.clone();
        }
        self.last_auto_save_content_hash = None;
        self.last_edit_time = None;
        self.save_version = self.save_version.wrapping_add(1);
        self.cached_is_modified = false;
        self.cached_is_modified_version = self.content_version;
        self.cached_is_modified_save_version = self.save_version;
        if self.path.is_some() {
            self.untitled_display_name = None;
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Encoding Methods
    // ─────────────────────────────────────────────────────────────────────────

    /// List of common encodings for the UI picker.
    pub const COMMON_ENCODINGS: &'static [&'static str] = &[
        "utf-8",
        "windows-1252",
        "iso-8859-1",
        "shift_jis",
        "euc-jp",
        "gbk",
        "euc-kr",
        "iso-8859-15",
        "utf-16le",
        "utf-16be",
    ];

    /// Get the display name for the current encoding (uppercase for UI).
    pub fn encoding_display_name(&self) -> String {
        self.current_encoding.to_uppercase()
    }

    /// Change the encoding and re-decode content from original bytes.
    ///
    /// Returns `Ok(())` if successful, or `Err` with a message if the encoding
    /// is invalid or the bytes cannot be decoded with the new encoding.
    ///
    /// Note: This only works if we have original_bytes stored (i.e., file was opened,
    /// not created new). For new documents, this just changes the save encoding.
    pub fn set_encoding(&mut self, new_encoding: &'static str) -> Result<(), String> {
        // Get the encoding from the label
        let encoding = encoding_rs::Encoding::for_label(new_encoding.as_bytes())
            .ok_or_else(|| format!("Unknown encoding: {}", new_encoding))?;

        // If we have original bytes, re-decode the content
        if !self.original_bytes.is_empty() {
            let (decoded, _actual_encoding, had_errors) = encoding.decode(&self.original_bytes);

            if had_errors {
                // Still update, but warn about errors
                log::warn!(
                    "Decoding with {} had errors - some characters may be replaced",
                    new_encoding
                );
            }

            // Update content (preserve cursor position as best we can)
            let old_len = self.content.len();
            self.content = decoded.into_owned();
            self.original_content = self.content.clone();
            self.content_version = self.content_version.wrapping_add(1);
            self.bump_source_epoch();
            self.undo_snapshot_version = u64::MAX;

            // If content length changed significantly, reset cursor
            if ((self.content.len() as isize) - (old_len as isize)).abs() > 100 {
                self.cursors = MultiCursor::new();
                self.cursor_position = (0, 0);
            }
        }

        // Update the encoding label for future saves
        self.current_encoding = new_encoding;
        self.detected_encoding = Some(new_encoding);

        log::info!("Changed encoding to: {}", new_encoding);
        Ok(())
    }

    /// Replace the whole document from bytes read from disk.
    ///
    /// This is the single entry point for external file changes. It updates
    /// content, encoding/BOM, the clean-on-disk anchor and the revision in one
    /// step so the raw editor, rendered view and caches all resynchronize.
    pub fn replace_from_disk(&mut self, bytes: Vec<u8>) {
        use chardetng::EncodingDetector;

        let is_large_file = bytes.len() >= LARGE_FILE_THRESHOLD;
        let (content, encoding_name, had_bom) = if let Some((bom_encoding, bom_len)) =
            encoding_rs::Encoding::for_bom(&bytes)
        {
            let (decoded, _) = bom_encoding.decode_without_bom_handling(&bytes[bom_len..]);
            (decoded.into_owned(), bom_encoding.name(), true)
        } else {
            let preferred = encoding_rs::Encoding::for_label(self.current_encoding.as_bytes());
            if let Some(encoding) = preferred.filter(|e| *e != encoding_rs::UTF_8) {
                let (decoded, _, had_errors) = encoding.decode(&bytes);
                if !had_errors {
                    (decoded.into_owned(), encoding.name(), false)
                } else {
                    let mut detector = EncodingDetector::new();
                    detector.feed(&bytes, true);
                    let detected = detector.guess(None, true);
                    let (decoded, _, _) = detected.decode(&bytes);
                    (decoded.into_owned(), detected.name(), false)
                }
            } else {
                let mut detector = EncodingDetector::new();
                detector.feed(&bytes, true);
                let detected = detector.guess(None, true);
                let (decoded, _, _) = detected.decode(&bytes);
                (decoded.into_owned(), detected.name(), false)
            }
        };

        self.content = content;
        self.is_large_file = is_large_file;
        if is_large_file {
            self.original_content.clear();
            self.original_content_hash = Some(Self::compute_content_hash(&self.content));
            self.original_bytes.clear();
        } else {
            self.original_content = self.content.clone();
            self.original_content_hash = None;
            self.original_bytes = bytes;
        }
        self.detected_encoding = Some(encoding_name);
        self.current_encoding = encoding_name;
        self.had_bom = had_bom;

        // Invalidate every revision-keyed cache and the Raw editor buffer.
        self.content_version = self.content_version.wrapping_add(1);
        self.bump_source_epoch();
        self.pending_undo_snapshot = None;
        self.undo_snapshot_version = u64::MAX;
        self.edit_history = if is_large_file {
            EditHistory::with_max_groups(LARGE_FILE_MAX_UNDO_GROUPS)
        } else {
            EditHistory::new()
        };

        let max_chars = self.content.chars().count();
        let head = self.cursors.primary().head.min(max_chars);
        self.cursors.set_single(Selection::cursor(head));
        self.sync_cursor_from_primary();
        self.pending_cursor_restore = Some(head);
    }

    /// Encode the current content to bytes using the selected encoding.
    ///
    /// Returns the encoded bytes. If the encoding doesn't support certain characters,
    /// they may be replaced with fallback characters.
    ///
    /// If the original file had a BOM (Byte Order Mark), it will be prepended to the output.
    /// This is important for UTF-16 files which require a BOM for proper detection.
    ///
    /// Note: encoding_rs does NOT support encoding TO UTF-16, only decoding FROM it.
    /// We handle UTF-16 encoding manually using Rust's built-in encode_utf16().
    pub fn encode_content(&self) -> Vec<u8> {
        let encoding_lower = self.current_encoding.to_lowercase();

        // Handle UTF-16 specially - encoding_rs doesn't support encoding TO UTF-16
        if encoding_lower == "utf-16le" || encoding_lower == "utf-16-le" {
            let mut result = Vec::new();
            // Add BOM if original had one
            if self.had_bom {
                result.extend_from_slice(&[0xff, 0xfe]);
            }
            // Encode to UTF-16LE (little endian)
            for code_unit in self.content.encode_utf16() {
                result.extend_from_slice(&code_unit.to_le_bytes());
            }
            return result;
        }

        if encoding_lower == "utf-16be" || encoding_lower == "utf-16-be" {
            let mut result = Vec::new();
            // Add BOM if original had one
            if self.had_bom {
                result.extend_from_slice(&[0xfe, 0xff]);
            }
            // Encode to UTF-16BE (big endian)
            for code_unit in self.content.encode_utf16() {
                result.extend_from_slice(&code_unit.to_be_bytes());
            }
            return result;
        }

        // For all other encodings, use encoding_rs
        let encoding = encoding_rs::Encoding::for_label(self.current_encoding.as_bytes())
            .unwrap_or(encoding_rs::UTF_8);

        let (encoded, _actual_encoding, _had_errors) = encoding.encode(&self.content);
        let mut result = encoded.into_owned();

        // Prepend BOM if the original file had one (for UTF-8 with BOM)
        if self.had_bom {
            let bom = Self::get_bom_for_encoding(self.current_encoding);
            if !bom.is_empty() {
                let mut with_bom = bom.to_vec();
                with_bom.append(&mut result);
                return with_bom;
            }
        }

        result
    }

    /// Get the BOM bytes for a given encoding label.
    fn get_bom_for_encoding(encoding_label: &str) -> &'static [u8] {
        match encoding_label.to_lowercase().as_str() {
            "utf-8" => &[0xef, 0xbb, 0xbf],
            "utf-16le" | "utf-16-le" => &[0xff, 0xfe],
            "utf-16be" | "utf-16-be" => &[0xfe, 0xff],
            _ => &[], // Other encodings don't use BOM
        }
    }

    /// Check if the current encoding is UTF-8.
    pub fn is_utf8(&self) -> bool {
        self.current_encoding.eq_ignore_ascii_case("utf-8")
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Auto-Save Methods
    // ─────────────────────────────────────────────────────────────────────────

    /// Toggle auto-save for this tab.
    pub fn toggle_auto_save(&mut self) {
        self.auto_save_enabled = !self.auto_save_enabled;
        if !self.auto_save_enabled {
            // Clear auto-save tracking when disabled
            self.last_edit_time = None;
        }
    }

    /// Mark that content was edited (updates last_edit_time for auto-save scheduling).
    pub fn mark_content_edited(&mut self) {
        if self.auto_save_enabled {
            self.last_edit_time = Some(std::time::Instant::now());
        }
    }

    /// Check if auto-save should trigger based on idle time.
    ///
    /// Uses cached is_modified() and content_version to avoid O(N) per-frame work.
    pub fn should_auto_save(&self, delay_ms: u32) -> bool {
        if !self.auto_save_enabled || !self.is_modified() {
            return false;
        }

        if let Some(last_ver) = self.last_auto_save_content_version {
            if self.content_version == last_ver {
                return false;
            }
        }

        if let Some(last_edit) = self.last_edit_time {
            last_edit.elapsed() >= std::time::Duration::from_millis(delay_ms as u64)
        } else {
            false
        }
    }

    /// Mark that auto-save was performed (stores content_version to avoid O(N) re-hash).
    pub fn mark_auto_saved(&mut self) {
        self.last_auto_save_content_hash = Some(hash_content(&self.content));
        self.last_auto_save_content_version = Some(self.content_version);
    }

    /// Get the content hash for change detection.
    pub fn content_hash(&self) -> u64 {
        let version = self.content_version;
        if let Some((cached_version, hash)) = self.cached_content_hash.get() {
            if cached_version == version {
                return hash;
            }
        }
        let hash = hash_content(&self.content);
        self.cached_content_hash.set(Some((version, hash)));
        hash
    }

    /// Set new content and record diff-based undo operation.
    pub fn set_content(&mut self, new_content: String) {
        if new_content != self.content {
            let ops = compute_edit_ops(&self.content, &new_content);
            self.edit_history.record_operations(ops);
            self.content = new_content;
            self.content_version = self.content_version.wrapping_add(1);
            self.bump_source_epoch();
            // The pending snapshot is stale; the next prepare refreshes it.
            self.undo_snapshot_version = u64::MAX;
            self.mark_content_edited();
        }
    }

    /// Undo the last edit group.
    ///
    /// Applies inverse operations to `self.content` and bumps `content_version`
    /// to signal UI widgets to re-sync. Updates the undo snapshot in-place
    /// (reusing buffer) and refreshes the blake3 hash to avoid a re-clone
    /// on the next frame. Returns cursor char position from the first
    /// operation in the group, or `None` if the undo stack was empty.
    pub fn undo(&mut self) -> Option<usize> {
        let cursor = self.document.edit_history.undo_string(&mut self.document.content);
        if cursor.is_some() {
            self.content_version = self.content_version.wrapping_add(1);
            self.bump_source_epoch();
            match self.document.pending_undo_snapshot.as_mut() {
                Some(snap) => snap.clone_from(&self.document.content),
                None => {
                    self.document.pending_undo_snapshot = Some(self.document.content.clone());
                }
            }
            self.undo_snapshot_version = self.content_version;
        }
        cursor
    }

    /// Redo the last undone edit group.
    ///
    /// Reapplies operations to `self.content` and bumps `content_version`.
    /// Updates the undo snapshot in-place.
    /// Returns cursor char position, or `None` if the redo stack was empty.
    pub fn redo(&mut self) -> Option<usize> {
        let cursor = self.document.edit_history.redo_string(&mut self.document.content);
        if cursor.is_some() {
            self.content_version = self.content_version.wrapping_add(1);
            self.bump_source_epoch();
            match self.document.pending_undo_snapshot.as_mut() {
                Some(snap) => snap.clone_from(&self.document.content),
                None => {
                    self.document.pending_undo_snapshot = Some(self.document.content.clone());
                }
            }
            self.undo_snapshot_version = self.content_version;
        }
        cursor
    }

    /// Check if undo is available.
    pub fn can_undo(&self) -> bool {
        self.edit_history.can_undo()
    }

    /// Check if redo is available.
    pub fn can_redo(&self) -> bool {
        self.edit_history.can_redo()
    }

    /// Get the number of undo groups.
    pub fn undo_count(&self) -> usize {
        self.edit_history.undo_count()
    }

    /// Get the number of redo groups.
    pub fn redo_count(&self) -> usize {
        self.edit_history.redo_count()
    }

    /// Break the current undo group so the next edit starts a new one.
    /// Used after formatting operations and other discrete actions.
    pub fn break_undo_group(&mut self) {
        self.edit_history.break_group();
    }

    /// Get the content version counter.
    pub fn content_version(&self) -> u64 {
        self.content_version
    }

    /// External-invalidation epoch for stable rendered-mode widget ids.
    ///
    /// Bumped on raw edits, file reload, undo/redo, and other changes that replace
    /// content outside the rendered edit session. Rendered WYSIWYG block commits do
    /// not bump this counter.
    pub fn source_epoch(&self) -> u64 {
        self.source_epoch
    }

    /// Bump [`source_epoch`] after external content invalidation.
    pub fn bump_source_epoch(&mut self) {
        self.source_epoch = self.source_epoch.saturating_add(1);
        log::trace!(
            "Tab {} source_epoch bumped to {}",
            self.id,
            self.source_epoch
        );
    }

    /// Signal external content change after assigning to [`Self::content`] directly.
    ///
    /// Bumps [`content_version`] and [`source_epoch`]. Does not record undo — pair with
    /// [`record_edit`](Self::record_edit) when undo support is needed.
    pub fn notify_external_content_change(&mut self) {
        self.content_version = self.content_version.wrapping_add(1);
        self.bump_source_epoch();
    }

    /// Increment the content version counter.
    ///
    /// Call this when content is modified programmatically (e.g., snippet expansion)
    /// to signal to UI widgets that they need to re-read content from the source.
    pub fn increment_content_version(&mut self) {
        self.content_version = self.content_version.wrapping_add(1);
    }

    /// Prepare an undo snapshot when the document revision changed.
    ///
    /// `content_version` is the single source of truth for "did the text
    /// change"; no per-frame hashing is needed. The clone only happens once
    /// per revision, and `clone_from` reuses the previous buffer.
    /// Call before non-Raw view widgets that can modify `content`.
    pub fn prepare_undo_snapshot_hashed(&mut self) {
        if self.undo_snapshot_version != self.content_version
            || self.pending_undo_snapshot.is_none()
        {
            match self.document.pending_undo_snapshot.as_mut() {
                Some(snap) => snap.clone_from(&self.document.content),
                None => {
                    self.document.pending_undo_snapshot = Some(self.document.content.clone());
                }
            }
            self.undo_snapshot_version = self.content_version;
        }
    }

    /// Record an edit by diffing the pending snapshot against current content.
    ///
    /// Rendered-mode edits: use this method (does **not** bump [`source_epoch`]).
    /// Raw / external paths: use [`record_external_edit_from_snapshot`].
    pub fn record_edit_from_snapshot(&mut self) {
        self.record_edit_from_snapshot_inner(false);
    }

    /// Apply queued rendered block commits as logical undo steps (one per commit boundary).
    ///
    /// Called from `central_panel` after `MarkdownEditor::show` drains
    /// [`crate::markdown::rendered_commit_undo::take_pending_commits`].
    pub fn apply_rendered_commit_undo_entries(
        &mut self,
        entries: impl IntoIterator<Item = crate::markdown::rendered_commit_undo::PendingRenderedCommitUndo>,
    ) {
        let final_content = self.content.clone();
        for entry in entries {
            if entry.break_group_before {
                self.break_undo_group();
            }
            self.content = entry.post_commit_snapshot;
            self.pending_undo_snapshot = Some(entry.pre_commit_snapshot);
            self.record_edit_from_snapshot();
        }
        self.content = final_content;
        if let Some(snap) = self.document.pending_undo_snapshot.as_mut() {
            snap.clone_from(&self.document.content);
        }
        self.undo_snapshot_version = self.content_version;
    }

    /// Like [`record_edit_from_snapshot`] but bumps [`source_epoch`] when content changes.
    pub fn record_external_edit_from_snapshot(&mut self) {
        self.record_edit_from_snapshot_inner(true);
    }

    fn record_edit_from_snapshot_inner(&mut self, bump_epoch: bool) {
        if let Some(mut old_content) = self.pending_undo_snapshot.take() {
            if old_content != self.content {
                let ops = compute_edit_ops(&old_content, &self.content);
                self.edit_history.record_operations(ops);
                old_content.clone_from(&self.content);
                self.content_version = self.content_version.wrapping_add(1);
                if bump_epoch {
                    self.bump_source_epoch();
                }
            }
            self.pending_undo_snapshot = Some(old_content);
            self.undo_snapshot_version = self.content_version;
        }
    }

    /// Record operations reported directly by the Raw editor.
    ///
    /// Avoids full-document `to_string()` + diff on every keystroke: positions
    /// are valid against the current `content`, so replaying them produces the
    /// editor buffer exactly.
    pub fn record_operations_external(&mut self, ops: Vec<EditOperation>) {
        if ops.is_empty() {
            return;
        }
        self.edit_history.record_operations(ops);
        self.content_version = self.content_version.wrapping_add(1);
        self.bump_source_epoch();
        self.pending_undo_snapshot = None;
        self.undo_snapshot_version = u64::MAX;
    }

    /// Legacy shim: record edit from explicit old content.
    /// Prefer `prepare_undo_snapshot_hashed` + `record_edit_from_snapshot` for new code.
    pub fn record_edit(&mut self, old_content: String, _old_cursor: usize) {
        if old_content != self.content {
            let ops = compute_edit_ops(&old_content, &self.content);
            self.edit_history.record_operations(ops);
            self.content_version = self.content_version.wrapping_add(1);
            self.undo_snapshot_version = u64::MAX;
            self.pending_undo_snapshot = None;
            self.bump_source_epoch();
        }
    }

    /// Convert to TabInfo for session persistence.
    pub fn to_tab_info(&self) -> TabInfo {
        TabInfo {
            path: self.path.clone(),
            modified: self.is_modified(),
            cursor_position: self.cursor_position,
            scroll_offset: self.scroll_offset,
            view_mode: self.view_mode,
            split_ratio: self.split_ratio,
        }
    }

    /// Get the current view mode for this tab.
    pub fn get_view_mode(&self) -> ViewMode {
        self.view_mode
    }

    /// Set the view mode for this tab.
    pub fn set_view_mode(&mut self, mode: ViewMode) {
        self.view_mode = mode;
    }

    /// Toggle the view mode: Raw → Split → Rendered → Raw
    pub fn toggle_view_mode(&mut self) -> ViewMode {
        self.view_mode = self.view_mode.toggle();
        self.view_mode
    }

    /// Get the split view ratio for this tab.
    pub fn get_split_ratio(&self) -> f32 {
        self.split_ratio
    }

    /// Set the split view ratio for this tab.
    /// The ratio is clamped to a valid range (0.2 to 0.8).
    pub fn set_split_ratio(&mut self, ratio: f32) {
        self.split_ratio = ratio.clamp(0.2, 0.8);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Live Pipeline Support
    // ─────────────────────────────────────────────────────────────────────────

    /// Check if pipeline panel is visible for this tab.
    pub fn pipeline_visible(&self) -> bool {
        self.pipeline_state.panel_visible
    }

    /// Toggle the pipeline panel visibility.
    pub fn toggle_pipeline_panel(&mut self) {
        self.pipeline_state.panel_visible = !self.pipeline_state.panel_visible;
    }

    /// Show the pipeline panel.
    pub fn show_pipeline_panel(&mut self) {
        self.pipeline_state.panel_visible = true;
    }

    /// Hide the pipeline panel.
    pub fn hide_pipeline_panel(&mut self) {
        self.pipeline_state.panel_visible = false;
    }

    /// Check if this tab's file type supports pipeline (JSON/YAML).
    pub fn supports_pipeline(&self) -> bool {
        matches!(self.file_type, FileType::Json | FileType::Yaml)
    }

    /// Get the file type for this tab.
    ///
    /// Returns the cached file type, which is determined from the
    /// file path extension. New/unsaved tabs default to Markdown.
    pub fn file_type(&self) -> FileType {
        self.file_type
    }

    /// Set the file path and update the cached file type.
    ///
    /// This should be called when saving a file with a new path
    /// (e.g., "Save As") to ensure the file type is updated.
    pub fn set_path(&mut self, path: PathBuf) {
        self.file_type = FileType::from_path(&path);
        self.path = Some(path);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Multi-Cursor Support
    // ─────────────────────────────────────────────────────────────────────────

    /// Sync legacy cursor_position and selection fields from the primary cursor.
    ///
    /// Call this after modifying the cursors to keep backwards compatibility
    /// with code that uses the legacy fields.
    pub fn sync_cursor_from_primary(&mut self) {
        self.cursor_position = self.cursors.cursor_position(&self.content);
        self.selection = self.cursors.selection_range();
    }

    /// Check if multi-cursor mode is active (more than one cursor).
    pub fn has_multiple_cursors(&self) -> bool {
        !self.cursors.is_single()
    }

    /// Get the number of active cursors.
    pub fn cursor_count(&self) -> usize {
        self.cursors.len()
    }

    /// Clear all cursors and reset to a single cursor at the given position.
    pub fn clear_to_single_cursor(&mut self, pos: usize) {
        self.cursors.set_single(Selection::cursor(pos));
        self.sync_cursor_from_primary();
    }

    /// Clear all cursors and reset to a single cursor at the primary position.
    pub fn exit_multi_cursor_mode(&mut self) {
        let primary_pos = self.cursors.primary().head;
        self.clear_to_single_cursor(primary_pos);
    }

    /// Add a new cursor at the given character position.
    pub fn add_cursor(&mut self, pos: usize) {
        self.cursors.add(Selection::cursor(pos));
        self.sync_cursor_from_primary();
    }

    /// Add a new selection (for Ctrl+D next occurrence).
    pub fn add_selection(&mut self, anchor: usize, head: usize) {
        self.cursors.add(Selection::new(anchor, head));
        self.sync_cursor_from_primary();
    }

    /// Set the primary cursor/selection (for single cursor operations).
    pub fn set_cursor(&mut self, pos: usize) {
        self.cursors.set_single(Selection::cursor(pos));
        self.sync_cursor_from_primary();
    }

    /// Set the primary selection (for single selection operations).
    pub fn set_selection(&mut self, anchor: usize, head: usize) {
        self.cursors.set_single(Selection::new(anchor, head));
        self.sync_cursor_from_primary();
    }

    /// Update cursor state from egui's TextEdit cursor range.
    ///
    /// This syncs the multi-cursor state with egui's single-cursor model.
    /// When multi-cursor editing is active, this only updates the primary cursor.
    pub fn update_cursor_from_egui(&mut self, primary: usize, secondary: usize) {
        if self.cursors.is_single() {
            // Single cursor mode: sync from egui
            if primary == secondary {
                self.cursors.set_single(Selection::cursor(primary));
            } else {
                // egui uses primary as cursor position, secondary as anchor
                self.cursors.set_single(Selection::new(secondary, primary));
            }
        } else {
            // Multi-cursor mode: only update primary cursor, preserve others
            let primary_sel = self.cursors.primary_mut();
            if primary == secondary {
                primary_sel.anchor = primary;
                primary_sel.head = primary;
            } else {
                primary_sel.anchor = secondary;
                primary_sel.head = primary;
            }
        }
        self.sync_cursor_from_primary();
    }

    /// Find the next occurrence of the given text after the specified position.
    /// Returns (start, end) character indices if found.
    pub fn find_next_occurrence(
        &self,
        search_text: &str,
        after_pos: usize,
    ) -> Option<(usize, usize)> {
        if search_text.is_empty() {
            return None;
        }

        // Search from after_pos to end
        if let Some(rel_pos) = self.content[after_pos..].find(search_text) {
            let start = after_pos + rel_pos;
            let end = start + search_text.len();
            return Some((start, end));
        }

        // Wrap around: search from beginning to after_pos
        if let Some(rel_pos) = self.content[..after_pos].find(search_text) {
            let end = rel_pos + search_text.len();
            return Some((rel_pos, end));
        }

        None
    }

    /// Get the text under the primary cursor (word at cursor if no selection).
    pub fn get_primary_selection_text(&self) -> Option<String> {
        let primary = self.cursors.primary();

        if primary.is_selection() {
            // Return selected text
            let (start, end) = primary.range();
            if end <= self.content.len() {
                return Some(self.content[start..end].to_string());
            }
        } else {
            // No selection: find word at cursor
            return self.word_at_position(primary.head);
        }

        None
    }

    /// Get the word at the given character position.
    fn word_at_position(&self, pos: usize) -> Option<String> {
        if self.content.is_empty() || pos > self.content.len() {
            return None;
        }

        let chars: Vec<char> = self.content.chars().collect();
        let char_pos = pos.min(chars.len().saturating_sub(1));

        // Find word boundaries
        let is_word_char = |c: char| c.is_alphanumeric() || c == '_';

        // Check if we're on a word character
        if char_pos < chars.len() && !is_word_char(chars[char_pos]) {
            return None;
        }

        // Find start of word
        let mut start = char_pos;
        while start > 0 && is_word_char(chars[start - 1]) {
            start -= 1;
        }

        // Find end of word
        let mut end = char_pos;
        while end < chars.len() && is_word_char(chars[end]) {
            end += 1;
        }

        if start < end {
            Some(chars[start..end].iter().collect())
        } else {
            None
        }
    }

    /// Get the byte range of the word at the given character position.
    pub fn word_range_at_position(&self, pos: usize) -> Option<(usize, usize)> {
        if self.content.is_empty() || pos > self.content.len() {
            return None;
        }

        let chars: Vec<char> = self.content.chars().collect();
        let char_pos = pos.min(chars.len().saturating_sub(1));

        let is_word_char = |c: char| c.is_alphanumeric() || c == '_';

        // Check if we're on a word character
        if char_pos < chars.len() && !is_word_char(chars[char_pos]) {
            return None;
        }

        // Find start of word
        let mut start = char_pos;
        while start > 0 && is_word_char(chars[start - 1]) {
            start -= 1;
        }

        // Find end of word
        let mut end = char_pos;
        while end < chars.len() && is_word_char(chars[end]) {
            end += 1;
        }

        if start < end {
            Some((start, end))
        } else {
            None
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Transient Highlight (Search Result Navigation)
    // ─────────────────────────────────────────────────────────────────────────

    /// Set a transient highlight for search result navigation.
    ///
    /// This highlight is temporary and will be cleared on scroll, edit, or click.
    /// The programmatic scroll that positions the match is ignored.
    pub fn set_transient_highlight(&mut self, start: usize, end: usize) {
        self.transient_highlight.set(start, end);
    }

    /// Clear the transient highlight.
    pub fn clear_transient_highlight(&mut self) {
        self.transient_highlight.clear();
    }

    /// Check if a transient highlight is active.
    pub fn has_transient_highlight(&self) -> bool {
        self.transient_highlight.is_active()
    }

    /// Get the transient highlight range if active.
    pub fn transient_highlight_range(&self) -> Option<(usize, usize)> {
        self.transient_highlight.range()
    }

    /// Notify that a scroll event occurred.
    ///
    /// This will clear the transient highlight unless it's the first scroll
    /// after the highlight was set (the programmatic scroll to position the match).
    pub fn on_scroll_event(&mut self) {
        self.transient_highlight.on_scroll();
    }

    /// Notify that an edit event occurred. Clears the transient highlight.
    pub fn on_edit_event(&mut self) {
        self.transient_highlight.on_edit();
    }

    /// Notify that a click event occurred. Clears the transient highlight.
    pub fn on_click_event(&mut self) {
        self.transient_highlight.on_click();
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Code Folding
    // ─────────────────────────────────────────────────────────────────────────

    /// Update fold regions for this tab using the detection algorithm.
    ///
    /// This should be called when content changes significantly or when
    /// folding settings change. The current collapsed states are preserved
    /// where possible.
    pub fn update_folds(
        &mut self,
        fold_headings: bool,
        fold_code_blocks: bool,
        fold_lists: bool,
        fold_indentation: bool,
    ) {
        use crate::editor::folding::detect_fold_regions;

        // Remember currently collapsed fold positions
        let collapsed_lines: std::collections::HashSet<usize> = self
            .fold_state
            .regions()
            .iter()
            .filter(|r| r.collapsed)
            .map(|r| r.start_line)
            .collect();

        // Detect new fold regions
        let mut new_state = detect_fold_regions(
            &self.content,
            self.file_type,
            fold_headings,
            fold_code_blocks,
            fold_lists,
            fold_indentation,
        );

        // Restore collapsed state for matching start lines
        for region in new_state.regions_mut() {
            if collapsed_lines.contains(&region.start_line) {
                region.collapsed = true;
            }
        }

        self.fold_state = new_state;
    }

    /// Mark fold state as needing recomputation.
    pub fn mark_folds_dirty(&mut self) {
        self.fold_state.mark_dirty();
    }

    /// Check if fold state needs recomputation.
    pub fn folds_dirty(&self) -> bool {
        self.fold_state.is_dirty()
    }

    /// Toggle the fold at the given line.
    ///
    /// Returns true if a fold was toggled.
    pub fn toggle_fold_at_line(&mut self, line: usize) -> bool {
        self.fold_state.toggle_at_line(line)
    }

    /// Check if a line is hidden by a fold.
    pub fn is_line_folded(&self, line: usize) -> bool {
        self.fold_state.is_line_hidden(line)
    }

    /// Reveal a line by expanding any fold that hides it.
    pub fn reveal_line(&mut self, line: usize) -> bool {
        self.fold_state.reveal_line(line)
    }

    /// Get lines that should show fold indicators.
    ///
    /// Returns (line, is_collapsed) for each fold start line.
    pub fn fold_indicator_lines(&self) -> Vec<(usize, bool)> {
        self.fold_state.fold_indicator_lines()
    }

    /// Fold all regions.
    pub fn fold_all(&mut self) {
        self.fold_state.fold_all();
    }

    /// Unfold all regions.
    pub fn unfold_all(&mut self) {
        self.fold_state.unfold_all();
    }

    /// Fold all headings.
    pub fn fold_all_headings(&mut self) {
        self.fold_state
            .fold_all_of_kind(|k| matches!(k, FoldKind::Heading(_)));
    }

    /// Fold all code blocks.
    pub fn fold_all_code_blocks(&mut self) {
        self.fold_state
            .fold_all_of_kind(|k| matches!(k, FoldKind::CodeBlock));
    }

    /// Get the number of collapsed folds.
    pub fn collapsed_fold_count(&self) -> usize {
        self.fold_state.collapsed_count()
    }

    /// Get total hidden line count from folds.
    pub fn hidden_line_count(&self) -> usize {
        self.fold_state.hidden_line_count()
    }
}

impl Default for Tab {
    fn default() -> Self {
        Self::new(0) // Defaults to Raw view mode and Markdown file type
    }
}

