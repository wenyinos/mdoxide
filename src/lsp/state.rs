//! Per-language-server status and diagnostic state tracked by the LSP client.

use std::collections::HashMap;
use std::path::PathBuf;

/// LSP diagnostic severity (mirrors LSP spec values 1–4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DiagnosticSeverity {
    Error = 1,
    Warning = 2,
    Information = 3,
    Hint = 4,
}

impl DiagnosticSeverity {
    pub fn from_lsp(value: u64) -> Self {
        match value {
            1 => Self::Error,
            2 => Self::Warning,
            3 => Self::Information,
            _ => Self::Hint,
        }
    }
}

/// A single diagnostic entry projected from LSP `Diagnostic`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticEntry {
    /// 0-indexed start line.
    pub start_line: usize,
    /// 0-indexed start column (UTF-16 offset as per LSP, but we treat as char col for rendering).
    pub start_col: usize,
    /// 0-indexed end line.
    pub end_line: usize,
    /// 0-indexed end column.
    pub end_col: usize,
    /// Severity.
    pub severity: DiagnosticSeverity,
    /// Human-readable message.
    pub message: String,
    /// Optional source string (e.g. "rustc", "clippy").
    pub source: Option<String>,
}

/// Per-file diagnostic list, keyed by file path.
///
/// Updated when `textDocument/publishDiagnostics` arrives; the UI reads this
/// each frame for visible-line squiggles and hover tooltips.
#[derive(Debug, Clone, Default)]
pub struct DiagnosticMap {
    inner: HashMap<PathBuf, Vec<DiagnosticEntry>>,
}

impl DiagnosticMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Replace diagnostics for a single file (empty vec clears them).
    /// Path is normalized to ensure consistent lookup across URI and filesystem representations.
    pub fn set(&mut self, path: PathBuf, diagnostics: Vec<DiagnosticEntry>) {
        let key = super::normalize_lsp_path(&path);
        if diagnostics.is_empty() {
            self.inner.remove(&key);
        } else {
            self.inner.insert(key, diagnostics);
        }
    }

    /// Get diagnostics for a file, if any.
    /// Path is normalized before lookup.
    pub fn get(&self, path: &std::path::Path) -> Option<&[DiagnosticEntry]> {
        let key = super::normalize_lsp_path(path);
        self.inner.get(&key).map(|v| v.as_slice())
    }

    /// Return diagnostics that touch the given 0-indexed line range `[start, end)`.
    pub fn for_line_range(
        &self,
        path: &std::path::Path,
        start_line: usize,
        end_line: usize,
    ) -> Vec<&DiagnosticEntry> {
        let key = super::normalize_lsp_path(path);
        match self.inner.get(&key) {
            Some(diags) => diags
                .iter()
                .filter(|d| d.start_line < end_line && d.end_line >= start_line)
                .collect(),
            None => Vec::new(),
        }
    }

    /// Total error + warning counts across all files (for status bar).
    pub fn counts(&self) -> (usize, usize) {
        let mut errors = 0usize;
        let mut warnings = 0usize;
        for diags in self.inner.values() {
            for d in diags {
                match d.severity {
                    DiagnosticSeverity::Error => errors += 1,
                    DiagnosticSeverity::Warning => warnings += 1,
                    _ => {}
                }
            }
        }
        (errors, warnings)
    }

    /// Remove all diagnostics (e.g. when LSP is disabled or workspace closes).
    pub fn clear(&mut self) {
        self.inner.clear();
    }
}

/// High-level connection state for a single language server process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerStatus {
    /// No server running for this key.
    Disconnected,
    /// Process spawn requested; child not yet confirmed.
    Starting,
    /// Child running; LSP `initialize` / handshake not yet complete (reserved for future use).
    Initializing,
    /// Server process is up; handshake may still be added in a later task.
    Ready,
    /// Last operation failed (spawn error, protocol error, etc.).
    Error(String),
}

impl Default for ServerStatus {
    fn default() -> Self {
        Self::Disconnected
    }
}

impl ServerStatus {
    /// Short label for the status bar (user-facing).
    pub fn short_label(&self) -> String {
        match self {
            ServerStatus::Disconnected => "Disconnected".to_string(),
            ServerStatus::Starting => "Starting…".to_string(),
            ServerStatus::Initializing => "Initializing…".to_string(),
            ServerStatus::Ready => "Ready".to_string(),
            ServerStatus::Error(e) => {
                let el = e.to_lowercase();
                if el.contains("not found")
                    || el.contains("no such file")
                    || el.contains("cannot find")
                {
                    "Not found".to_string()
                } else if el == "server crashed" {
                    "Crashed".to_string()
                } else {
                    "Error".to_string()
                }
            }
        }
    }
}
