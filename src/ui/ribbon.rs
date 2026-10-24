//! Application-menu actions for MDOxide.
//!
//! This module used to host the Ferrite-style ribbon toolbar. The chrome now
//! follows MarkText's frameless title bar with a hamburger menu, so only the
//! action vocabulary remains: the menu in `app::title_bar` emits these values
//! and `MDOxideApp::handle_ribbon_action` dispatches them.

use crate::markdown::formatting::MarkdownFormatCommand;

/// Actions that can be triggered from the application menu.
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)] // Some variants are only reachable from keyboard shortcuts
pub enum RibbonAction {
    // File operations
    /// Create a new file/tab
    New,
    /// Open file dialog
    Open,
    /// Open folder/workspace dialog
    OpenWorkspace,
    /// Close current workspace (return to single-file mode)
    CloseWorkspace,
    /// Save current file
    Save,
    /// Save As dialog
    SaveAs,
    /// Toggle auto-save for current document (kept for keyboard shortcut handling)
    ToggleAutoSave,

    // Workspace operations (only visible in workspace mode)
    /// Search in files across workspace (Ctrl+Shift+F)
    SearchInFiles,
    /// Quick file switcher / file palette (Ctrl+P)
    QuickFileSwitcher,

    // Edit operations
    /// Undo last change
    Undo,
    /// Redo last undone change
    Redo,

    // Formatting operations (Markdown)
    /// Apply a markdown formatting command
    Format(MarkdownFormatCommand),

    // Markdown document operations
    /// Insert or update Table of Contents
    InsertToc,

    // Structured data operations (JSON/YAML/TOML)
    /// Format/pretty-print the structured data document
    FormatDocument,
    /// Validate syntax of the structured data document
    ValidateSyntax,
    /// Toggle Live Pipeline panel (JSON/YAML only)
    TogglePipeline,

    // View operations (kept for keyboard shortcut handling)
    /// Toggle between Raw and Rendered view
    ToggleViewMode,
    /// Toggle line numbers visibility
    ToggleLineNumbers,
    /// Toggle sync scrolling between Raw and Rendered views
    ToggleSyncScroll,

    // Tools
    /// Open Find/Replace dialog (placeholder)
    FindReplace,
    /// Toggle outline panel
    ToggleOutline,

    // Export operations
    /// Export current document as HTML file
    ExportHtml,
    /// Copy rendered HTML to clipboard
    CopyAsHtml,
    /// Export current document as a PDF file (opens the options dialog).
    ExportPdf,
    /// Print preview via in-app PDF viewer (same renderer as Export PDF).
    PrintPreview,

    // Settings (kept for keyboard shortcut handling)
    /// Cycle through themes
    CycleTheme,
    /// Open settings panel
    OpenSettings,

    // Zen Mode (kept for keyboard shortcut handling)
    /// Toggle Zen Mode (distraction-free writing)
    ToggleZenMode,

    // Terminal
    /// Toggle terminal panel visibility
    ToggleTerminal,

    // Productivity
    /// Toggle productivity hub visibility
    ToggleProductivity,

    // Frontmatter
    /// Toggle the frontmatter panel (kept for keyboard shortcut handling)
    ToggleFrontmatter,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ribbon_action_equality() {
        assert_eq!(RibbonAction::New, RibbonAction::New);
        assert_ne!(RibbonAction::New, RibbonAction::Open);
    }
}
