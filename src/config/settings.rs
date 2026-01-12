//! User settings and preferences for Ferrite
//!
//! This module defines the `Settings` struct that holds all user-configurable
//! options, with serde support for JSON persistence.

// Allow dead code - this module contains complete API with methods for UI display
// labels and settings that may not all be used yet but provide consistent API
#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ─────────────────────────────────────────────────────────────────────────────
// Maximum Line Width Configuration
// ─────────────────────────────────────────────────────────────────────────────

/// Maximum line width setting for the editor.
///
/// Controls the maximum width of text content in the editor. When enabled,
/// text is constrained to the specified width and centered in the viewport.
/// This applies to Raw, Rendered, Split, and Zen mode views.
/// Reference: GitHub Issue #15
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum MaxLineWidth {
    /// No width limit (current behavior)
    #[default]
    Off,
    /// 80 characters (traditional terminal width)
    #[serde(rename = "80")]
    Col80,
    /// 100 characters (comfortable reading width)
    #[serde(rename = "100")]
    Col100,
    /// 120 characters (wide monitors)
    #[serde(rename = "120")]
    Col120,
    /// Custom pixel width
    #[serde(rename = "custom")]
    Custom(u32),
}

impl MaxLineWidth {
    /// Get the display name for UI.
    pub fn display_name(&self) -> &'static str {
        match self {
            MaxLineWidth::Off => "Off",
            MaxLineWidth::Col80 => "80 characters",
            MaxLineWidth::Col100 => "100 characters",
            MaxLineWidth::Col120 => "120 characters",
            MaxLineWidth::Custom(_) => "Custom",
        }
    }

    /// Get a description of the setting.
    pub fn description(&self) -> &'static str {
        match self {
            MaxLineWidth::Off => "No width limit",
            MaxLineWidth::Col80 => "Traditional terminal width",
            MaxLineWidth::Col100 => "Comfortable reading width",
            MaxLineWidth::Col120 => "Wide monitor width",
            MaxLineWidth::Custom(_) => "Custom pixel width",
        }
    }

    /// Get all preset width options (excludes Custom).
    pub fn presets() -> &'static [MaxLineWidth] {
        &[
            MaxLineWidth::Off,
            MaxLineWidth::Col80,
            MaxLineWidth::Col100,
            MaxLineWidth::Col120,
        ]
    }

    /// Convert to pixel width given a font's approximate character width.
    ///
    /// Returns `None` for `Off` (no limit).
    pub fn to_pixels(&self, char_width: f32) -> Option<f32> {
        match self {
            MaxLineWidth::Off => None,
            MaxLineWidth::Col80 => Some(char_width * 80.0),
            MaxLineWidth::Col100 => Some(char_width * 100.0),
            MaxLineWidth::Col120 => Some(char_width * 120.0),
            MaxLineWidth::Custom(px) => Some(*px as f32),
        }
    }

    /// Check if this is a custom width setting.
    pub fn is_custom(&self) -> bool {
        matches!(self, MaxLineWidth::Custom(_))
    }

    /// Get the custom pixel value, if any.
    pub fn custom_value(&self) -> Option<u32> {
        if let MaxLineWidth::Custom(px) = self {
            Some(*px)
        } else {
            None
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Log Level Configuration
// ─────────────────────────────────────────────────────────────────────────────

/// Available log levels for controlling runtime log filtering.
///
/// Controls the verbosity of log output. Default is `Warn`.
/// Reference: GitHub Issue #11
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    /// Most verbose - shows all debug messages
    Debug,
    /// Informational messages and above
    Info,
    /// Warnings and errors only (default)
    #[default]
    Warn,
    /// Errors only
    Error,
    /// Disable all logging
    Off,
}

impl LogLevel {
    /// Get the display name for the log level.
    pub fn display_name(&self) -> &'static str {
        match self {
            LogLevel::Debug => "Debug",
            LogLevel::Info => "Info",
            LogLevel::Warn => "Warn",
            LogLevel::Error => "Error",
            LogLevel::Off => "Off",
        }
    }

    /// Get a description of the log level.
    pub fn description(&self) -> &'static str {
        match self {
            LogLevel::Debug => "Most verbose, shows all debug messages",
            LogLevel::Info => "Informational messages and above",
            LogLevel::Warn => "Warnings and errors only (default)",
            LogLevel::Error => "Errors only",
            LogLevel::Off => "Disable all logging",
        }
    }

    /// Get all available log levels.
    pub fn all() -> &'static [LogLevel] {
        &[
            LogLevel::Debug,
            LogLevel::Info,
            LogLevel::Warn,
            LogLevel::Error,
            LogLevel::Off,
        ]
    }

    /// Convert to log::LevelFilter for env_logger initialization.
    pub fn to_level_filter(&self) -> log::LevelFilter {
        match self {
            LogLevel::Debug => log::LevelFilter::Debug,
            LogLevel::Info => log::LevelFilter::Info,
            LogLevel::Warn => log::LevelFilter::Warn,
            LogLevel::Error => log::LevelFilter::Error,
            LogLevel::Off => log::LevelFilter::Off,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Theme Configuration
// ─────────────────────────────────────────────────────────────────────────────

/// Available color themes for the editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    Light,
    Dark,
    System,
}

// ─────────────────────────────────────────────────────────────────────────────
// Font Family Configuration
// ─────────────────────────────────────────────────────────────────────────────

/// Available font families for the editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum EditorFont {
    /// Inter - Modern, clean UI font (default)
    #[default]
    Inter,
    /// JetBrains Mono - Monospace font, good for code-heavy documents
    JetBrainsMono,
}

impl EditorFont {
    /// Get the display name for the font.
    pub fn display_name(&self) -> &'static str {
        match self {
            EditorFont::Inter => "Inter",
            EditorFont::JetBrainsMono => "JetBrains Mono",
        }
    }

    /// Get a description of the font.
    pub fn description(&self) -> &'static str {
        match self {
            EditorFont::Inter => "Modern, clean proportional font",
            EditorFont::JetBrainsMono => "Monospace font for code",
        }
    }

    /// Get all available fonts.
    pub fn all() -> &'static [EditorFont] {
        &[EditorFont::Inter, EditorFont::JetBrainsMono]
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// View Mode Configuration
// ─────────────────────────────────────────────────────────────────────────────

/// Editor view modes for markdown editing.
///
/// Three modes are available:
/// - `Raw`: Plain markdown text editing using a standard text editor
/// - `Rendered`: WYSIWYG editing with rendered markdown elements
/// - `Split`: Side-by-side split view with raw editor on left and preview on right
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ViewMode {
    /// Raw markdown text editing (plain TextEdit)
    #[default]
    Raw,
    /// WYSIWYG rendered editing (MarkdownEditor)
    Rendered,
    /// Split view: raw editor (left) + rendered preview (right)
    Split,
}

impl ViewMode {
    /// Cycle through view modes: Raw → Split → Rendered → Raw
    pub fn toggle(&self) -> Self {
        match self {
            ViewMode::Raw => ViewMode::Split,
            ViewMode::Split => ViewMode::Rendered,
            ViewMode::Rendered => ViewMode::Raw,
        }
    }

    /// Get a display label for the mode.
    pub fn label(&self) -> &'static str {
        match self {
            ViewMode::Raw => "Raw",
            ViewMode::Rendered => "Rendered",
            ViewMode::Split => "Split",
        }
    }

    /// Get an icon/symbol for the mode.
    #[allow(dead_code)]
    pub fn icon(&self) -> &'static str {
        match self {
            ViewMode::Raw => "📝",
            ViewMode::Rendered => "👁",
            ViewMode::Split => "⫿",
        }
    }

    /// Check if this mode shows the raw editor.
    pub fn shows_raw(&self) -> bool {
        matches!(self, ViewMode::Raw | ViewMode::Split)
    }

    /// Check if this mode shows the rendered preview.
    pub fn shows_rendered(&self) -> bool {
        matches!(self, ViewMode::Rendered | ViewMode::Split)
    }

    /// Get all available view modes.
    pub fn all() -> &'static [ViewMode] {
        &[ViewMode::Raw, ViewMode::Rendered, ViewMode::Split]
    }

    /// Get a description of the view mode.
    pub fn description(&self) -> &'static str {
        match self {
            ViewMode::Raw => "Plain markdown text editing",
            ViewMode::Rendered => "WYSIWYG rendered editing",
            ViewMode::Split => "Raw editor + rendered preview side by side",
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Outline Panel Configuration
// ─────────────────────────────────────────────────────────────────────────────

/// Which side of the editor the outline panel should appear on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum OutlinePanelSide {
    /// Outline panel on the left side
    Left,
    /// Outline panel on the right side (default)
    #[default]
    Right,
}

impl OutlinePanelSide {
    /// Toggle between left and right.
    #[allow(dead_code)]
    pub fn toggle(&self) -> Self {
        match self {
            OutlinePanelSide::Left => OutlinePanelSide::Right,
            OutlinePanelSide::Right => OutlinePanelSide::Left,
        }
    }

    /// Get display label.
    #[allow(dead_code)]
    pub fn label(&self) -> &'static str {
        match self {
            OutlinePanelSide::Left => "Left",
            OutlinePanelSide::Right => "Right",
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Window Size Configuration
// ─────────────────────────────────────────────────────────────────────────────

/// Window dimensions and position.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WindowSize {
    /// Window width in pixels
    pub width: f32,
    /// Window height in pixels
    pub height: f32,
    /// Window X position (optional, for restoring position)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<f32>,
    /// Window Y position (optional, for restoring position)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<f32>,
    /// Whether the window was maximized
    #[serde(default)]
    pub maximized: bool,
}

impl Default for WindowSize {
    fn default() -> Self {
        Self {
            width: 1200.0,
            height: 800.0,
            x: None,
            y: None,
            maximized: false,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tab Information
// ─────────────────────────────────────────────────────────────────────────────

/// Information about an open tab for session restoration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TabInfo {
    /// Path to the file (None for unsaved/new files)
    pub path: Option<PathBuf>,
    /// Whether this tab has unsaved changes (used for recovery)
    #[serde(default)]
    pub modified: bool,
    /// Cursor position (line, column)
    #[serde(default)]
    pub cursor_position: (usize, usize),
    /// Scroll position
    #[serde(default)]
    pub scroll_offset: f32,
    /// View mode for this tab (raw, rendered, or split)
    #[serde(default)]
    pub view_mode: ViewMode,
    /// Split view ratio (0.0 to 1.0, where ratio is the proportion for the left pane)
    /// Default is 0.5 (50/50 split). Only used when view_mode is Split.
    #[serde(default = "default_split_ratio")]
    pub split_ratio: f32,
}

/// Default split ratio for TabInfo (50/50 split)
fn default_split_ratio() -> f32 {
    0.5
}

impl Default for TabInfo {
    fn default() -> Self {
        Self {
            path: None,
            modified: false,
            cursor_position: (0, 0),
            scroll_offset: 0.0,
            view_mode: ViewMode::Raw, // New documents default to raw mode
            split_ratio: 0.5,         // Default to 50/50 split
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Main Settings Struct
// ─────────────────────────────────────────────────────────────────────────────

/// User preferences and application settings.
///
/// This struct is serialized to JSON and persisted to the user's config directory.
/// All fields have sensible defaults via the `Default` trait and `#[serde(default)]`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    // ─────────────────────────────────────────────────────────────────────────
    // Appearance
    // ─────────────────────────────────────────────────────────────────────────
    /// Color theme (light, dark, or system)
    pub theme: Theme,

    /// Editor view mode (editor only, preview only, or split view)
    pub view_mode: ViewMode,

    /// Whether to show line numbers in the editor
    pub show_line_numbers: bool,

    /// Font size for the editor (in points)
    pub font_size: f32,

    /// Font family for the editor
    pub font_family: EditorFont,

    // ─────────────────────────────────────────────────────────────────────────
    // Editor Behavior
    // ─────────────────────────────────────────────────────────────────────────
    /// Whether to enable word wrap
    pub word_wrap: bool,

    /// Tab size (number of spaces)
    pub tab_size: u8,

    /// Whether to use spaces instead of tabs
    pub use_spaces: bool,

    /// Default auto-save state for new tabs/documents
    /// When true, new documents will have auto-save enabled by default
    pub auto_save_enabled_default: bool,

    /// Auto-save delay in milliseconds after last edit before triggering save
    /// Uses temp-file based strategy to avoid overwriting main file prematurely
    /// Default is 15000ms (15 seconds)
    pub auto_save_delay_ms: u32,

    // ─────────────────────────────────────────────────────────────────────────
    // Session & History
    // ─────────────────────────────────────────────────────────────────────────
    /// Recently opened files (most recent first)
    pub recent_files: Vec<PathBuf>,

    /// Maximum number of recent files to remember
    pub max_recent_files: usize,

    /// Last open tabs for session restoration
    pub last_open_tabs: Vec<TabInfo>,

    /// Index of the active tab (for session restoration)
    pub active_tab_index: usize,

    // ─────────────────────────────────────────────────────────────────────────
    // Window State
    // ─────────────────────────────────────────────────────────────────────────
    /// Window size and position
    pub window_size: WindowSize,

    /// Split ratio for the editor/preview panes (0.0 to 1.0)
    pub split_ratio: f32,

    // ─────────────────────────────────────────────────────────────────────────
    // Syntax Highlighting
    // ─────────────────────────────────────────────────────────────────────────
    /// Syntax highlighting theme name
    pub syntax_theme: String,

    // ─────────────────────────────────────────────────────────────────────────
    // Outline Panel
    // ─────────────────────────────────────────────────────────────────────────
    /// Whether the outline panel is visible
    pub outline_enabled: bool,

    /// Which side of the editor the outline panel appears on
    pub outline_side: OutlinePanelSide,

    /// Width of the outline panel in pixels
    pub outline_width: f32,

    // ─────────────────────────────────────────────────────────────────────────
    // Sync Scrolling
    // ─────────────────────────────────────────────────────────────────────────
    /// Whether synchronized scrolling between Raw and Rendered views is enabled
    pub sync_scroll_enabled: bool,

    // ─────────────────────────────────────────────────────────────────────────
    // Export Settings
    // ─────────────────────────────────────────────────────────────────────────
    /// Last directory used for HTML export
    pub last_export_directory: Option<std::path::PathBuf>,

    /// Whether to open exported files after export
    pub open_after_export: bool,

    /// Whether to embed images as base64 in exports
    pub export_embed_images: bool,

    // ─────────────────────────────────────────────────────────────────────────
    // Workspace Settings
    // ─────────────────────────────────────────────────────────────────────────
    /// Recently opened workspaces (folders), most recent first
    pub recent_workspaces: Vec<PathBuf>,

    /// Maximum number of recent workspaces to remember
    pub max_recent_workspaces: usize,

    // ─────────────────────────────────────────────────────────────────────────
    // Zen Mode Settings
    // ─────────────────────────────────────────────────────────────────────────
    /// Maximum column width for Zen Mode (in characters, approx 70-90)
    pub zen_max_column_width: f32,

    /// Whether Zen Mode was enabled in the last session (for restore)
    pub zen_mode_enabled: bool,

    // ─────────────────────────────────────────────────────────────────────────
    // Code Folding Settings
    // ─────────────────────────────────────────────────────────────────────────
    /// Whether code folding is enabled globally
    pub folding_enabled: bool,

    /// Whether to show fold indicators in the gutter
    pub folding_show_indicators: bool,

    /// Whether to fold Markdown headings
    pub fold_headings: bool,

    /// Whether to fold fenced code blocks
    pub fold_code_blocks: bool,

    /// Whether to fold list hierarchies
    pub fold_lists: bool,

    /// Whether to use indentation-based folding for JSON/YAML
    pub fold_indentation: bool,

    // ─────────────────────────────────────────────────────────────────────────
    // Live Pipeline Settings
    // ─────────────────────────────────────────────────────────────────────────
    /// Whether the Live Pipeline feature is enabled (for JSON/YAML files)
    pub pipeline_enabled: bool,

    /// Debounce delay in milliseconds before auto-executing pipeline command
    pub pipeline_debounce_ms: u32,

    /// Maximum output size in bytes (to prevent memory issues)
    pub pipeline_max_output_bytes: u32,

    /// Maximum runtime in milliseconds before killing the process
    pub pipeline_max_runtime_ms: u32,

    /// Height of the pipeline panel in pixels
    pub pipeline_panel_height: f32,

    /// Recent pipeline commands (persisted across sessions)
    pub pipeline_recent_commands: Vec<String>,

    // ─────────────────────────────────────────────────────────────────────────
    // Minimap Settings
    // ─────────────────────────────────────────────────────────────────────────
    /// Whether the minimap is enabled
    pub minimap_enabled: bool,

    /// Width of the minimap in pixels
    pub minimap_width: f32,

    // ─────────────────────────────────────────────────────────────────────────
    // Bracket Matching Settings
    // ─────────────────────────────────────────────────────────────────────────
    /// Whether to highlight matching brackets and emphasis pairs when cursor is adjacent
    /// Supports (), [], {}, <>, and markdown emphasis ** and __
    pub highlight_matching_pairs: bool,

    /// Whether to automatically insert closing brackets and quotes when typing openers.
    /// When enabled:
    /// - Typing `(`, `[`, `{`, `"`, `'`, or `` ` `` inserts the closing pair
    /// - If text is selected, wraps the selection with the pair
    /// - Typing a closer when the next character is the same closer skips over it
    pub auto_close_brackets: bool,

    // ─────────────────────────────────────────────────────────────────────────
    // Syntax Highlighting Settings
    // ─────────────────────────────────────────────────────────────────────────
    /// Whether to enable syntax highlighting for source code files in raw editor mode
    /// Supports Rust, Python, JavaScript, TypeScript, and many other languages
    pub syntax_highlighting_enabled: bool,

    // ─────────────────────────────────────────────────────────────────────────
    // Maximum Line Width Settings
    // ─────────────────────────────────────────────────────────────────────────
    /// Maximum line width for text content in the editor.
    /// When enabled, constrains text width and centers the text column.
    /// Applies to Raw, Rendered, Split, and Zen mode views.
    /// Reference: GitHub Issue #15
    pub max_line_width: MaxLineWidth,

    // ─────────────────────────────────────────────────────────────────────────
    // Logging Settings
    // ─────────────────────────────────────────────────────────────────────────
    /// Log level for controlling runtime log verbosity.
    /// Default is Warn. Can be overridden via --log-level CLI flag.
    /// Reference: GitHub Issue #11
    pub log_level: LogLevel,

    // ─────────────────────────────────────────────────────────────────────────
    // Default View Mode
    // ─────────────────────────────────────────────────────────────────────────
    /// Default view mode for new tabs.
    /// Controls whether new tabs open in Raw, Rendered, or Split view.
    /// Existing tabs retain their stored view mode (not overridden by this setting).
    /// Reference: GitHub Issue #3
    pub default_view_mode: ViewMode,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            // Appearance
            theme: Theme::default(),
            view_mode: ViewMode::default(),
            show_line_numbers: true,
            font_size: 14.0,
            font_family: EditorFont::default(),

            // Editor Behavior
            word_wrap: true,
            tab_size: 4,
            use_spaces: true,
            auto_save_enabled_default: false,
            auto_save_delay_ms: 15000, // 15 seconds default

            // Session & History
            recent_files: Vec::new(),
            max_recent_files: 10,
            last_open_tabs: Vec::new(),
            active_tab_index: 0,

            // Window State
            window_size: WindowSize::default(),
            split_ratio: 0.5,

            // Syntax Highlighting
            syntax_theme: String::from("base16-ocean.dark"),

            // Outline Panel
            outline_enabled: false, // Hidden by default
            outline_side: OutlinePanelSide::default(),
            outline_width: 200.0,

            // Sync Scrolling
            sync_scroll_enabled: true, // Enabled by default

            // Export Settings
            last_export_directory: None,
            open_after_export: false,
            export_embed_images: true, // Standalone files by default

            // Workspace Settings
            recent_workspaces: Vec::new(),
            max_recent_workspaces: 10,

            // Zen Mode Settings
            zen_max_column_width: 80.0, // ~80 characters default
            zen_mode_enabled: false,

            // Code Folding Settings
            folding_enabled: true,           // Folding enabled by default
            folding_show_indicators: false,  // Hide fold indicators by default (they don't collapse yet)
            fold_headings: true,             // Fold headings by default
            fold_code_blocks: true,          // Fold code blocks by default
            fold_lists: true,                // Fold lists by default
            fold_indentation: true,          // Indentation folding for JSON/YAML

            // Live Pipeline Settings
            pipeline_enabled: true,          // Feature enabled by default
            pipeline_debounce_ms: 500,       // 500ms debounce
            pipeline_max_output_bytes: 1024 * 1024, // 1 MB max output
            pipeline_max_runtime_ms: 30000,  // 30 seconds max runtime
            pipeline_panel_height: 200.0,    // Default panel height
            pipeline_recent_commands: Vec::new(),

            // Minimap Settings
            minimap_enabled: true,           // Minimap enabled by default
            minimap_width: 80.0,             // Default minimap width

            // Bracket Matching Settings
            highlight_matching_pairs: true,  // Bracket matching enabled by default
            auto_close_brackets: true,       // Auto-close brackets enabled by default

            // Syntax Highlighting Settings
            syntax_highlighting_enabled: true, // Syntax highlighting enabled by default

            // Maximum Line Width Settings
            max_line_width: MaxLineWidth::default(), // Off by default (no limit)

            // Logging Settings
            log_level: LogLevel::default(), // Default to Warn level

            // Default View Mode
            default_view_mode: ViewMode::default(), // Default to Raw mode
        }
    }
}

impl Settings {
    /// Add a file to the recent files list.
    ///
    /// If the file already exists in the list, it's moved to the front.
    /// The list is trimmed to `max_recent_files`.
    pub fn add_recent_file(&mut self, path: PathBuf) {
        // Remove if already exists
        self.recent_files.retain(|p| p != &path);
        // Add to front
        self.recent_files.insert(0, path);
        // Trim to max
        self.recent_files.truncate(self.max_recent_files);
    }

    /// Add a workspace (folder) to the recent workspaces list.
    ///
    /// If the workspace already exists in the list, it's moved to the front.
    /// The list is trimmed to `max_recent_workspaces`.
    pub fn add_recent_workspace(&mut self, path: PathBuf) {
        // Remove if already exists
        self.recent_workspaces.retain(|p| p != &path);
        // Add to front
        self.recent_workspaces.insert(0, path);
        // Trim to max
        self.recent_workspaces.truncate(self.max_recent_workspaces);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Validation Constants and Sanitization
    // ─────────────────────────────────────────────────────────────────────────

    /// Minimum allowed font size.
    pub const MIN_FONT_SIZE: f32 = 8.0;
    /// Maximum allowed font size.
    pub const MAX_FONT_SIZE: f32 = 72.0;
    /// Minimum allowed tab size.
    pub const MIN_TAB_SIZE: u8 = 1;
    /// Maximum allowed tab size.
    pub const MAX_TAB_SIZE: u8 = 8;
    /// Minimum window dimension.
    pub const MIN_WINDOW_SIZE: f32 = 200.0;
    /// Maximum window dimension.
    pub const MAX_WINDOW_SIZE: f32 = 10000.0;
    /// Minimum outline panel width.
    pub const MIN_OUTLINE_WIDTH: f32 = 120.0;
    /// Maximum outline panel width.
    pub const MAX_OUTLINE_WIDTH: f32 = 500.0;
    /// Minimum Zen Mode column width (characters).
    pub const MIN_ZEN_COLUMN_WIDTH: f32 = 50.0;
    /// Maximum Zen Mode column width (characters).
    pub const MAX_ZEN_COLUMN_WIDTH: f32 = 120.0;
    /// Minimum pipeline debounce in milliseconds.
    pub const MIN_PIPELINE_DEBOUNCE_MS: u32 = 100;
    /// Maximum pipeline debounce in milliseconds.
    pub const MAX_PIPELINE_DEBOUNCE_MS: u32 = 5000;
    /// Minimum pipeline output size in bytes (1 KB).
    pub const MIN_PIPELINE_OUTPUT_BYTES: u32 = 1024;
    /// Maximum pipeline output size in bytes (10 MB).
    pub const MAX_PIPELINE_OUTPUT_BYTES: u32 = 10 * 1024 * 1024;
    /// Minimum pipeline runtime in milliseconds (1 second).
    pub const MIN_PIPELINE_RUNTIME_MS: u32 = 1000;
    /// Maximum pipeline runtime in milliseconds (5 minutes).
    pub const MAX_PIPELINE_RUNTIME_MS: u32 = 300000;
    /// Minimum pipeline panel height.
    pub const MIN_PIPELINE_PANEL_HEIGHT: f32 = 100.0;
    /// Maximum pipeline panel height.
    pub const MAX_PIPELINE_PANEL_HEIGHT: f32 = 500.0;
    /// Maximum number of recent pipeline commands.
    pub const MAX_PIPELINE_RECENT_COMMANDS: usize = 20;
    /// Minimum minimap width.
    pub const MIN_MINIMAP_WIDTH: f32 = 40.0;
    /// Maximum minimap width.
    pub const MAX_MINIMAP_WIDTH: f32 = 150.0;
    /// Minimum custom line width in pixels.
    pub const MIN_CUSTOM_LINE_WIDTH: u32 = 400;
    /// Maximum custom line width in pixels.
    pub const MAX_CUSTOM_LINE_WIDTH: u32 = 2000;

    /// Sanitize settings by clamping values to valid ranges.
    ///
    /// This is useful after loading settings from a file that might have
    /// been manually edited with invalid values.
    pub fn sanitize(&mut self) {
        // Clamp font size
        self.font_size = self
            .font_size
            .clamp(Self::MIN_FONT_SIZE, Self::MAX_FONT_SIZE);

        // Clamp tab size
        self.tab_size = self.tab_size.clamp(Self::MIN_TAB_SIZE, Self::MAX_TAB_SIZE);

        // Clamp window size
        self.window_size.width = self
            .window_size
            .width
            .clamp(Self::MIN_WINDOW_SIZE, Self::MAX_WINDOW_SIZE);
        self.window_size.height = self
            .window_size
            .height
            .clamp(Self::MIN_WINDOW_SIZE, Self::MAX_WINDOW_SIZE);

        // Clamp split ratio
        self.split_ratio = self.split_ratio.clamp(0.0, 1.0);

        // Ensure max_recent_files is reasonable
        if self.max_recent_files == 0 {
            self.max_recent_files = 10;
        } else if self.max_recent_files > 100 {
            self.max_recent_files = 100;
        }

        // Trim recent files to max
        self.recent_files.truncate(self.max_recent_files);

        // Ensure auto-save delay is reasonable (minimum 5 seconds, max 5 minutes)
        self.auto_save_delay_ms = self.auto_save_delay_ms.clamp(5000, 300000);

        // Ensure active_tab_index is valid
        if !self.last_open_tabs.is_empty() && self.active_tab_index >= self.last_open_tabs.len() {
            self.active_tab_index = self.last_open_tabs.len() - 1;
        }

        // Clamp outline width
        self.outline_width = self
            .outline_width
            .clamp(Self::MIN_OUTLINE_WIDTH, Self::MAX_OUTLINE_WIDTH);

        // Clamp Zen Mode column width
        self.zen_max_column_width = self
            .zen_max_column_width
            .clamp(Self::MIN_ZEN_COLUMN_WIDTH, Self::MAX_ZEN_COLUMN_WIDTH);

        // Clamp pipeline settings
        self.pipeline_debounce_ms = self
            .pipeline_debounce_ms
            .clamp(Self::MIN_PIPELINE_DEBOUNCE_MS, Self::MAX_PIPELINE_DEBOUNCE_MS);
        self.pipeline_max_output_bytes = self
            .pipeline_max_output_bytes
            .clamp(Self::MIN_PIPELINE_OUTPUT_BYTES, Self::MAX_PIPELINE_OUTPUT_BYTES);
        self.pipeline_max_runtime_ms = self
            .pipeline_max_runtime_ms
            .clamp(Self::MIN_PIPELINE_RUNTIME_MS, Self::MAX_PIPELINE_RUNTIME_MS);
        self.pipeline_panel_height = self
            .pipeline_panel_height
            .clamp(Self::MIN_PIPELINE_PANEL_HEIGHT, Self::MAX_PIPELINE_PANEL_HEIGHT);
        self.pipeline_recent_commands
            .truncate(Self::MAX_PIPELINE_RECENT_COMMANDS);

        // Clamp minimap width
        self.minimap_width = self
            .minimap_width
            .clamp(Self::MIN_MINIMAP_WIDTH, Self::MAX_MINIMAP_WIDTH);

        // Clamp custom line width if set
        if let MaxLineWidth::Custom(px) = self.max_line_width {
            let clamped = px.clamp(Self::MIN_CUSTOM_LINE_WIDTH, Self::MAX_CUSTOM_LINE_WIDTH);
            self.max_line_width = MaxLineWidth::Custom(clamped);
        }
    }

    /// Load settings and sanitize them to ensure validity.
    ///
    /// This is a convenience method that deserializes and then sanitizes.
    pub fn from_json_sanitized(json: &str) -> Result<Self, serde_json::Error> {
        let mut settings: Self = serde_json::from_str(json)?;
        settings.sanitize();
        Ok(settings)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();

        assert_eq!(settings.theme, Theme::Light);
        assert_eq!(settings.view_mode, ViewMode::Raw);
        assert!(settings.show_line_numbers);
        assert_eq!(settings.font_size, 14.0);
        assert!(settings.recent_files.is_empty());
        assert_eq!(settings.max_recent_files, 10);
        assert_eq!(settings.window_size.width, 1200.0);
        assert_eq!(settings.window_size.height, 800.0);
        assert_eq!(settings.split_ratio, 0.5);
    }

    #[test]
    fn test_add_recent_file() {
        let mut settings = Settings::default();
        settings.max_recent_files = 3;

        settings.add_recent_file(PathBuf::from("/file1.md"));
        settings.add_recent_file(PathBuf::from("/file2.md"));
        settings.add_recent_file(PathBuf::from("/file3.md"));

        assert_eq!(settings.recent_files.len(), 3);
        assert_eq!(settings.recent_files[0], PathBuf::from("/file3.md"));
        assert_eq!(settings.recent_files[2], PathBuf::from("/file1.md"));

        // Add existing file - should move to front
        settings.add_recent_file(PathBuf::from("/file1.md"));
        assert_eq!(settings.recent_files[0], PathBuf::from("/file1.md"));
        assert_eq!(settings.recent_files.len(), 3);

        // Add new file - should trim oldest
        settings.add_recent_file(PathBuf::from("/file4.md"));
        assert_eq!(settings.recent_files.len(), 3);
        assert_eq!(settings.recent_files[0], PathBuf::from("/file4.md"));
        assert!(!settings.recent_files.contains(&PathBuf::from("/file2.md")));
    }

    #[test]
    fn test_theme_serialization() {
        assert_eq!(serde_json::to_string(&Theme::Light).unwrap(), "\"light\"");
        assert_eq!(serde_json::to_string(&Theme::Dark).unwrap(), "\"dark\"");
        assert_eq!(serde_json::to_string(&Theme::System).unwrap(), "\"system\"");
    }

    #[test]
    fn test_theme_deserialization() {
        assert_eq!(
            serde_json::from_str::<Theme>("\"light\"").unwrap(),
            Theme::Light
        );
        assert_eq!(
            serde_json::from_str::<Theme>("\"dark\"").unwrap(),
            Theme::Dark
        );
        assert_eq!(
            serde_json::from_str::<Theme>("\"system\"").unwrap(),
            Theme::System
        );
    }

    #[test]
    fn test_view_mode_serialization() {
        assert_eq!(serde_json::to_string(&ViewMode::Raw).unwrap(), "\"raw\"");
        assert_eq!(
            serde_json::to_string(&ViewMode::Rendered).unwrap(),
            "\"rendered\""
        );
        assert_eq!(
            serde_json::to_string(&ViewMode::Split).unwrap(),
            "\"split\""
        );
    }

    #[test]
    fn test_view_mode_toggle() {
        // Raw → Split → Rendered → Raw
        assert_eq!(ViewMode::Raw.toggle(), ViewMode::Split);
        assert_eq!(ViewMode::Split.toggle(), ViewMode::Rendered);
        assert_eq!(ViewMode::Rendered.toggle(), ViewMode::Raw);
    }

    #[test]
    fn test_view_mode_labels() {
        assert_eq!(ViewMode::Raw.label(), "Raw");
        assert_eq!(ViewMode::Rendered.label(), "Rendered");
        assert_eq!(ViewMode::Split.label(), "Split");
        assert_eq!(ViewMode::Raw.icon(), "📝");
        assert_eq!(ViewMode::Rendered.icon(), "👁");
        assert_eq!(ViewMode::Split.icon(), "⫿");
    }

    #[test]
    fn test_view_mode_shows_raw_rendered() {
        assert!(ViewMode::Raw.shows_raw());
        assert!(!ViewMode::Raw.shows_rendered());
        assert!(!ViewMode::Rendered.shows_raw());
        assert!(ViewMode::Rendered.shows_rendered());
        assert!(ViewMode::Split.shows_raw());
        assert!(ViewMode::Split.shows_rendered());
    }

    #[test]
    fn test_view_mode_all() {
        let all = ViewMode::all();
        assert_eq!(all.len(), 3);
        assert!(all.contains(&ViewMode::Raw));
        assert!(all.contains(&ViewMode::Rendered));
        assert!(all.contains(&ViewMode::Split));
    }

    #[test]
    fn test_view_mode_description() {
        assert!(!ViewMode::Raw.description().is_empty());
        assert!(!ViewMode::Rendered.description().is_empty());
        assert!(!ViewMode::Split.description().is_empty());
        // Ensure descriptions are different
        assert_ne!(ViewMode::Raw.description(), ViewMode::Rendered.description());
        assert_ne!(ViewMode::Raw.description(), ViewMode::Split.description());
    }

    #[test]
    fn test_settings_default_view_mode() {
        let settings = Settings::default();
        assert_eq!(settings.default_view_mode, ViewMode::Raw);
    }

    #[test]
    fn test_settings_backward_compatibility_default_view_mode() {
        // Old JSON without default_view_mode field should default to Raw
        let json = r#"{"theme": "dark"}"#;
        let settings: Settings = serde_json::from_str(json).unwrap();
        assert_eq!(settings.default_view_mode, ViewMode::Raw);
    }

    #[test]
    fn test_settings_serialize_default_view_mode() {
        let mut settings = Settings::default();
        settings.default_view_mode = ViewMode::Split;
        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("\"default_view_mode\":\"split\""));
    }

    #[test]
    fn test_settings_deserialize_default_view_mode() {
        let json = r#"{"default_view_mode": "rendered"}"#;
        let settings: Settings = serde_json::from_str(json).unwrap();
        assert_eq!(settings.default_view_mode, ViewMode::Rendered);
    }

    #[test]
    fn test_settings_serialization_roundtrip() {
        let original = Settings::default();
        let json = serde_json::to_string_pretty(&original).unwrap();
        let deserialized: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_settings_deserialize_with_defaults() {
        // Minimal JSON - should fill in defaults
        let json = r#"{"theme": "dark"}"#;
        let settings: Settings = serde_json::from_str(json).unwrap();

        assert_eq!(settings.theme, Theme::Dark);
        // All other fields should have defaults
        assert_eq!(settings.view_mode, ViewMode::Raw);
        assert!(settings.show_line_numbers);
        assert_eq!(settings.font_size, 14.0);
    }

    #[test]
    fn test_settings_deserialize_empty_json() {
        // Empty JSON object - should use all defaults
        let json = "{}";
        let settings: Settings = serde_json::from_str(json).unwrap();
        assert_eq!(settings, Settings::default());
    }

    #[test]
    fn test_window_size_default() {
        let size = WindowSize::default();
        assert_eq!(size.width, 1200.0);
        assert_eq!(size.height, 800.0);
        assert!(size.x.is_none());
        assert!(size.y.is_none());
        assert!(!size.maximized);
    }

    #[test]
    fn test_tab_info_default() {
        let tab = TabInfo::default();
        assert!(tab.path.is_none());
        assert!(!tab.modified);
        assert_eq!(tab.cursor_position, (0, 0));
        assert_eq!(tab.scroll_offset, 0.0);
    }

    #[test]
    fn test_tab_info_serialization() {
        let tab = TabInfo {
            path: Some(PathBuf::from("/test.md")),
            modified: true,
            cursor_position: (10, 5),
            scroll_offset: 100.0,
            view_mode: ViewMode::Rendered,
            split_ratio: 0.6,
        };

        let json = serde_json::to_string(&tab).unwrap();
        let deserialized: TabInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(tab, deserialized);
    }

    #[test]
    fn test_tab_info_default_split_ratio() {
        let tab = TabInfo::default();
        assert_eq!(tab.split_ratio, 0.5); // Default to 50/50 split
    }

    #[test]
    fn test_tab_info_backward_compatibility_split_ratio() {
        // Old JSON without split_ratio field should default to 0.5
        let json = r#"{"path": "/test.md", "modified": false, "cursor_position": [0, 0], "scroll_offset": 0.0, "view_mode": "raw"}"#;
        let tab: TabInfo = serde_json::from_str(json).unwrap();
        assert_eq!(tab.split_ratio, 0.5);
    }

    #[test]
    fn test_tab_info_default_view_mode() {
        let tab = TabInfo::default();
        assert_eq!(tab.view_mode, ViewMode::Raw); // Default to raw mode
    }

    #[test]
    fn test_tab_info_backward_compatibility() {
        // Old JSON without view_mode field should default to Raw
        let json = r#"{"path": "/test.md", "modified": false, "cursor_position": [0, 0], "scroll_offset": 0.0}"#;
        let tab: TabInfo = serde_json::from_str(json).unwrap();
        assert_eq!(tab.view_mode, ViewMode::Raw);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // LogLevel tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_log_level_default() {
        assert_eq!(LogLevel::default(), LogLevel::Warn);
    }

    #[test]
    fn test_log_level_serialization() {
        assert_eq!(serde_json::to_string(&LogLevel::Debug).unwrap(), "\"debug\"");
        assert_eq!(serde_json::to_string(&LogLevel::Info).unwrap(), "\"info\"");
        assert_eq!(serde_json::to_string(&LogLevel::Warn).unwrap(), "\"warn\"");
        assert_eq!(serde_json::to_string(&LogLevel::Error).unwrap(), "\"error\"");
        assert_eq!(serde_json::to_string(&LogLevel::Off).unwrap(), "\"off\"");
    }

    #[test]
    fn test_log_level_deserialization() {
        assert_eq!(
            serde_json::from_str::<LogLevel>("\"debug\"").unwrap(),
            LogLevel::Debug
        );
        assert_eq!(
            serde_json::from_str::<LogLevel>("\"info\"").unwrap(),
            LogLevel::Info
        );
        assert_eq!(
            serde_json::from_str::<LogLevel>("\"warn\"").unwrap(),
            LogLevel::Warn
        );
        assert_eq!(
            serde_json::from_str::<LogLevel>("\"error\"").unwrap(),
            LogLevel::Error
        );
        assert_eq!(
            serde_json::from_str::<LogLevel>("\"off\"").unwrap(),
            LogLevel::Off
        );
    }

    #[test]
    fn test_log_level_display_name() {
        assert_eq!(LogLevel::Debug.display_name(), "Debug");
        assert_eq!(LogLevel::Info.display_name(), "Info");
        assert_eq!(LogLevel::Warn.display_name(), "Warn");
        assert_eq!(LogLevel::Error.display_name(), "Error");
        assert_eq!(LogLevel::Off.display_name(), "Off");
    }

    #[test]
    fn test_log_level_all() {
        let all = LogLevel::all();
        assert_eq!(all.len(), 5);
        assert!(all.contains(&LogLevel::Debug));
        assert!(all.contains(&LogLevel::Info));
        assert!(all.contains(&LogLevel::Warn));
        assert!(all.contains(&LogLevel::Error));
        assert!(all.contains(&LogLevel::Off));
    }

    #[test]
    fn test_log_level_to_level_filter() {
        assert_eq!(LogLevel::Debug.to_level_filter(), log::LevelFilter::Debug);
        assert_eq!(LogLevel::Info.to_level_filter(), log::LevelFilter::Info);
        assert_eq!(LogLevel::Warn.to_level_filter(), log::LevelFilter::Warn);
        assert_eq!(LogLevel::Error.to_level_filter(), log::LevelFilter::Error);
        assert_eq!(LogLevel::Off.to_level_filter(), log::LevelFilter::Off);
    }

    #[test]
    fn test_settings_log_level_default() {
        let settings = Settings::default();
        assert_eq!(settings.log_level, LogLevel::Warn);
    }

    #[test]
    fn test_settings_backward_compatibility_log_level() {
        // Old JSON without log_level field should default to Warn
        let json = r#"{"theme": "dark"}"#;
        let settings: Settings = serde_json::from_str(json).unwrap();
        assert_eq!(settings.log_level, LogLevel::Warn);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Sanitization tests
    // ─────────────────────────────────────────────────────────────────────────
    #[test]
    fn test_sanitize_font_size() {
        let mut settings = Settings::default();
        settings.font_size = 4.0;
        settings.sanitize();
        assert_eq!(settings.font_size, Settings::MIN_FONT_SIZE);

        settings.font_size = 100.0;
        settings.sanitize();
        assert_eq!(settings.font_size, Settings::MAX_FONT_SIZE);
    }

    #[test]
    fn test_sanitize_tab_size() {
        let mut settings = Settings::default();
        settings.tab_size = 0;
        settings.sanitize();
        assert_eq!(settings.tab_size, Settings::MIN_TAB_SIZE);

        settings.tab_size = 20;
        settings.sanitize();
        assert_eq!(settings.tab_size, Settings::MAX_TAB_SIZE);
    }

    #[test]
    fn test_sanitize_split_ratio() {
        let mut settings = Settings::default();
        settings.split_ratio = -0.5;
        settings.sanitize();
        assert_eq!(settings.split_ratio, 0.0);

        settings.split_ratio = 1.5;
        settings.sanitize();
        assert_eq!(settings.split_ratio, 1.0);
    }

    #[test]
    fn test_sanitize_recent_files() {
        let mut settings = Settings::default();
        settings.max_recent_files = 2;
        settings.recent_files = vec![
            PathBuf::from("/file1.md"),
            PathBuf::from("/file2.md"),
            PathBuf::from("/file3.md"),
        ];
        settings.sanitize();
        assert_eq!(settings.recent_files.len(), 2);
    }

    #[test]
    fn test_sanitize_active_tab_index() {
        let mut settings = Settings::default();
        settings.last_open_tabs = vec![TabInfo::default()];
        settings.active_tab_index = 5;
        settings.sanitize();
        assert_eq!(settings.active_tab_index, 0);
    }

    #[test]
    fn test_from_json_sanitized() {
        let json = r#"{"font_size": 4.0, "split_ratio": 2.0}"#;
        let settings = Settings::from_json_sanitized(json).unwrap();
        assert_eq!(settings.font_size, Settings::MIN_FONT_SIZE);
        assert_eq!(settings.split_ratio, 1.0);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Code Folding Settings tests (GitHub Issue #12)
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_folding_show_indicators_default_false() {
        // Issue #12: Fold indicators are hidden by default because
        // they don't actually collapse yet (visual only)
        let settings = Settings::default();
        assert!(!settings.folding_show_indicators);
        // But folding detection is still enabled
        assert!(settings.folding_enabled);
    }

    #[test]
    fn test_folding_show_indicators_backward_compatibility() {
        // Old settings without folding_show_indicators should get the new default (false)
        let json = r#"{"theme": "dark"}"#;
        let settings: Settings = serde_json::from_str(json).unwrap();
        assert!(!settings.folding_show_indicators);
    }

    #[test]
    fn test_folding_show_indicators_explicit_true() {
        // Users can still enable it via settings
        let json = r#"{"folding_show_indicators": true}"#;
        let settings: Settings = serde_json::from_str(json).unwrap();
        assert!(settings.folding_show_indicators);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Auto-close Brackets tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_auto_close_brackets_default_true() {
        let settings = Settings::default();
        assert!(settings.auto_close_brackets);
    }

    #[test]
    fn test_auto_close_brackets_backward_compatibility() {
        // Old settings without auto_close_brackets should get the new default (true)
        let json = r#"{"theme": "dark"}"#;
        let settings: Settings = serde_json::from_str(json).unwrap();
        assert!(settings.auto_close_brackets);
    }

    #[test]
    fn test_auto_close_brackets_explicit_false() {
        // Users can disable it via settings
        let json = r#"{"auto_close_brackets": false}"#;
        let settings: Settings = serde_json::from_str(json).unwrap();
        assert!(!settings.auto_close_brackets);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // MaxLineWidth tests (GitHub Issue #15)
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_max_line_width_default() {
        assert_eq!(MaxLineWidth::default(), MaxLineWidth::Off);
    }

    #[test]
    fn test_max_line_width_serialization() {
        assert_eq!(serde_json::to_string(&MaxLineWidth::Off).unwrap(), "\"off\"");
        assert_eq!(serde_json::to_string(&MaxLineWidth::Col80).unwrap(), "\"80\"");
        assert_eq!(serde_json::to_string(&MaxLineWidth::Col100).unwrap(), "\"100\"");
        assert_eq!(serde_json::to_string(&MaxLineWidth::Col120).unwrap(), "\"120\"");
        assert_eq!(
            serde_json::to_string(&MaxLineWidth::Custom(600)).unwrap(),
            "{\"custom\":600}"
        );
    }

    #[test]
    fn test_max_line_width_deserialization() {
        assert_eq!(
            serde_json::from_str::<MaxLineWidth>("\"off\"").unwrap(),
            MaxLineWidth::Off
        );
        assert_eq!(
            serde_json::from_str::<MaxLineWidth>("\"80\"").unwrap(),
            MaxLineWidth::Col80
        );
        assert_eq!(
            serde_json::from_str::<MaxLineWidth>("\"100\"").unwrap(),
            MaxLineWidth::Col100
        );
        assert_eq!(
            serde_json::from_str::<MaxLineWidth>("\"120\"").unwrap(),
            MaxLineWidth::Col120
        );
        assert_eq!(
            serde_json::from_str::<MaxLineWidth>("{\"custom\":600}").unwrap(),
            MaxLineWidth::Custom(600)
        );
    }

    #[test]
    fn test_max_line_width_to_pixels() {
        let char_width = 10.0; // Example char width
        assert_eq!(MaxLineWidth::Off.to_pixels(char_width), None);
        assert_eq!(MaxLineWidth::Col80.to_pixels(char_width), Some(800.0));
        assert_eq!(MaxLineWidth::Col100.to_pixels(char_width), Some(1000.0));
        assert_eq!(MaxLineWidth::Col120.to_pixels(char_width), Some(1200.0));
        assert_eq!(MaxLineWidth::Custom(600).to_pixels(char_width), Some(600.0));
    }

    #[test]
    fn test_max_line_width_is_custom() {
        assert!(!MaxLineWidth::Off.is_custom());
        assert!(!MaxLineWidth::Col80.is_custom());
        assert!(!MaxLineWidth::Col100.is_custom());
        assert!(!MaxLineWidth::Col120.is_custom());
        assert!(MaxLineWidth::Custom(600).is_custom());
    }

    #[test]
    fn test_max_line_width_custom_value() {
        assert_eq!(MaxLineWidth::Off.custom_value(), None);
        assert_eq!(MaxLineWidth::Col80.custom_value(), None);
        assert_eq!(MaxLineWidth::Custom(600).custom_value(), Some(600));
    }

    #[test]
    fn test_settings_max_line_width_default() {
        let settings = Settings::default();
        assert_eq!(settings.max_line_width, MaxLineWidth::Off);
    }

    #[test]
    fn test_settings_backward_compatibility_max_line_width() {
        // Old JSON without max_line_width field should default to Off
        let json = r#"{"theme": "dark"}"#;
        let settings: Settings = serde_json::from_str(json).unwrap();
        assert_eq!(settings.max_line_width, MaxLineWidth::Off);
    }

    #[test]
    fn test_settings_serialize_max_line_width() {
        let mut settings = Settings::default();
        settings.max_line_width = MaxLineWidth::Col80;
        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("\"max_line_width\":\"80\""));
    }

    #[test]
    fn test_settings_deserialize_max_line_width() {
        let json = r#"{"max_line_width": "100"}"#;
        let settings: Settings = serde_json::from_str(json).unwrap();
        assert_eq!(settings.max_line_width, MaxLineWidth::Col100);
    }

    #[test]
    fn test_settings_sanitize_custom_line_width() {
        // Test clamping of custom line width
        let mut settings = Settings::default();
        
        // Below minimum
        settings.max_line_width = MaxLineWidth::Custom(100);
        settings.sanitize();
        assert_eq!(settings.max_line_width, MaxLineWidth::Custom(Settings::MIN_CUSTOM_LINE_WIDTH));
        
        // Above maximum
        settings.max_line_width = MaxLineWidth::Custom(5000);
        settings.sanitize();
        assert_eq!(settings.max_line_width, MaxLineWidth::Custom(Settings::MAX_CUSTOM_LINE_WIDTH));
        
        // Valid value unchanged
        settings.max_line_width = MaxLineWidth::Custom(800);
        settings.sanitize();
        assert_eq!(settings.max_line_width, MaxLineWidth::Custom(800));
    }
}
