//! UI-domain state: dialogs, panels, transient interaction flags.
//!
//! Extracted from `state.rs` as the second increment of P1-04, following the
//! document/tab split. `AppState` still owns the instance, but the shape of
//! UI state no longer lives in the aggregation module.

use std::path::PathBuf;

// ─────────────────────────────────────────────────────────────────────────────
// UI State
// ─────────────────────────────────────────────────────────────────────────────

/// Payload stashed while the code-execution consent dialog is open.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingCodeRun {
    pub code: String,
    pub language: String,
    pub cwd: Option<PathBuf>,
    pub timeout_secs: u32,
    pub block_id: egui::Id,
}

/// UI-related state flags.
#[derive(Debug, Clone, Default)]
pub struct UiState {
    /// Whether the settings panel is open
    pub show_settings: bool,
    /// Whether the file browser/open dialog is active
    pub show_file_dialog: bool,
    /// Whether the "save as" dialog is active
    pub show_save_as_dialog: bool,
    /// Whether the about dialog is open
    pub show_about: bool,
    /// Whether a confirmation dialog is open (e.g., unsaved changes)
    pub show_confirm_dialog: bool,
    /// Message for the confirmation dialog
    pub confirm_dialog_message: String,
    /// Pending action after confirmation
    pub pending_action: Option<PendingAction>,
    /// Status bar message (deprecated, use toast_message instead)
    pub status_message: Option<String>,
    /// Whether the find/replace panel is open
    pub show_find_replace: bool,
    /// Find/replace state
    pub find_state: crate::editor::FindState,
    /// Whether to scroll to the current match (set when navigating)
    pub scroll_to_match: bool,
    /// Whether a find search is pending (debounced)
    pub find_search_pending: bool,
    /// When the find search was last requested (for debouncing)
    pub find_search_requested_at: Option<std::time::Instant>,
    /// Whether to show error modal
    pub show_error_modal: bool,
    /// Error message for modal
    pub error_message: String,
    /// Whether to show the portal error dialog (Linux xdg-desktop-portal missing)
    pub show_portal_error_dialog: bool,
    /// Portal error message with installation instructions
    pub portal_error_message: String,
    /// Portal error command to copy (install command)
    pub portal_error_command: String,
    /// Temporary toast message (shown in center of status bar)
    pub toast_message: Option<String>,
    /// When the toast message should expire (as seconds since app start)
    pub toast_expires_at: Option<f64>,
    /// Whether the recent files popup is open
    pub show_recent_files_popup: bool,
    /// Whether the recent folders popup is open
    pub show_recent_folders_popup: bool,
    /// Whether Zen Mode is enabled (distraction-free writing)
    pub zen_mode: bool,
    /// Go to Line dialog state (None = closed)
    pub go_to_line_dialog: Option<crate::ui::GoToLineDialog>,
    /// Current Vim mode label for status bar display (None = Vim disabled).
    pub vim_mode_indicator: Option<&'static str>,
    /// Whether the HTML export options dialog is open.
    pub show_html_export_dialog: bool,
    /// Whether the PDF export options dialog is open.
    pub show_pdf_export_dialog: bool,
    /// First-run / consent modal for running fenced code from markdown preview.
    pub show_code_execution_consent_dialog: bool,
    /// Queued run for **Enable & run** after consent.
    pub pending_code_run: Option<PendingCodeRun>,
    /// When true, focus the Cancel button once when the consent dialog opens.
    pub code_execution_consent_focus_cancel: bool,
    /// Rename a pathless document tab: `(tab_index, text buffer)`.
    pub rename_untitled_tab: Option<(usize, String)>,
}

/// Whether a save confirmation applies to closing one tab or exiting the app.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SavePromptContext {
    /// Closing a single tab (tab strip, Ctrl+W, etc.)
    TabClose,
    /// App exit and other checks via [`AppState::has_unsaved_changes`]
    AppExit,
}

/// Actions that may need confirmation before execution.
#[derive(Debug, Clone, PartialEq)]
pub enum PendingAction {
    /// Close a specific tab
    CloseTab(usize),
    /// Close all tabs
    CloseAllTabs,
    /// Exit the application
    Exit,
    /// Open a new file (replacing current)
    OpenFile(PathBuf),
    /// Create a new document
    NewDocument,
}
