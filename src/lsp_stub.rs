//! No-op stubs for the LSP module when the `lsp` feature is disabled.
//!
//! Provides the same public API as `src/lsp/` so the rest of the codebase
//! compiles unchanged — all functions are inert, all collections empty.
//! Remove this file when re-enabling LSP (add `lsp` to default features).

#![allow(dead_code, unused_imports)]

use std::path::{Path, PathBuf};

// ── Sub-modules mirroring the real lsp crate structure ──────────────────────

pub mod state {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum DiagnosticSeverity {
        Error = 1,
        Warning = 2,
        Information = 3,
        Hint = 4,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct DiagnosticEntry {
        pub start_line: usize,
        pub start_col: usize,
        pub end_line: usize,
        pub end_col: usize,
        pub severity: DiagnosticSeverity,
        pub message: String,
        pub source: Option<String>,
    }

    #[derive(Debug, Clone, Default)]
    pub struct DiagnosticMap;

    impl DiagnosticMap {
        pub fn new() -> Self {
            Self
        }
        pub fn set(&mut self, _path: std::path::PathBuf, _diags: Vec<DiagnosticEntry>) {}
        pub fn get(&self, _path: &std::path::Path) -> Option<&[DiagnosticEntry]> {
            None
        }
        pub fn counts(&self) -> (usize, usize) {
            (0, 0)
        }
        pub fn clear(&mut self) {}
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum ServerStatus {
        Disconnected,
        Starting,
        Initializing,
        Ready,
        Error(String),
    }

    impl Default for ServerStatus {
        fn default() -> Self {
            Self::Disconnected
        }
    }

    impl ServerStatus {
        pub fn short_label(&self) -> String {
            "Disabled".to_string()
        }
    }
}

pub mod detection {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct LspServerSpec {
        pub program: String,
        pub args: Vec<String>,
    }
}

// ── Re-exports matching the real module ─────────────────────────────────────

pub use detection::LspServerSpec;
pub use state::{DiagnosticEntry, DiagnosticMap, DiagnosticSeverity, ServerStatus};

// ── LspManager (no-op) ─────────────────────────────────────────────────────

#[derive(Debug)]
pub struct LspManager;

impl LspManager {
    pub fn new() -> Self {
        Self
    }
    pub fn start_server(
        &self,
        _server_key: impl Into<String>,
        _spec: LspServerSpec,
        _workspace_root: Option<PathBuf>,
    ) {
    }
    pub fn stop_server(&self, _server_key: impl Into<String>) {}
    pub fn stop_all_servers(&self) {}
    pub fn did_open(
        &self,
        _server_key: impl Into<String>,
        _uri: String,
        _language_id: String,
        _version: i64,
        _text: String,
    ) {
    }
    pub fn did_change(
        &self,
        _server_key: impl Into<String>,
        _uri: String,
        _version: i64,
        _text: String,
    ) {
    }
    pub fn did_close(&self, _server_key: impl Into<String>, _uri: String) {}
    pub fn poll_events(&mut self) -> Vec<LspManagerEvent> {
        Vec::new()
    }
}

impl Default for LspManager {
    fn default() -> Self {
        Self::new()
    }
}

// ── LspManagerEvent (kept for type compatibility) ───────────────────────────

#[derive(Debug, Clone)]
pub enum LspManagerEvent {
    StatusChanged {
        server_key: String,
        status: state::ServerStatus,
    },
    SpawnFailed {
        server_key: String,
        program: String,
        error: String,
    },
    Diagnostics {
        server_key: String,
        path: PathBuf,
        diagnostics: Vec<state::DiagnosticEntry>,
    },
}

// ── Free functions (no-op / passthrough) ────────────────────────────────────

pub fn overrides_fingerprint(_map: &std::collections::HashMap<String, String>) -> u64 {
    0
}

pub fn normalize_lsp_path(path: &Path) -> PathBuf {
    path.to_path_buf()
}

pub fn path_to_uri(_path: &Path) -> String {
    String::new()
}

pub fn language_id_for_path(_path: &Path) -> &'static str {
    "plaintext"
}

pub fn detect_lsp_server_for_path(_path: &Path) -> Option<LspServerSpec> {
    None
}

pub fn install_hint(_program: &str) -> &'static str {
    ""
}

pub fn detect_servers_for_workspace(_root: &Path) -> Vec<(String, LspServerSpec)> {
    Vec::new()
}
