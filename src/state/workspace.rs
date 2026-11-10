//! Workspace-domain state: backlink index and link extraction.
//!
//! Extracted from `state.rs` as part of P1-04. The file tree/index/watcher
//! live in `crate::workspaces`; this module holds the derived state that
//! `AppState` caches for the active workspace.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ─────────────────────────────────────────────────────────────────────────────
// Backlink Index
// ─────────────────────────────────────────────────────────────────────────────

/// A single backlink entry: a file that references the target file.
#[derive(Debug, Clone)]
pub struct BacklinkEntry {
    /// Absolute path to the file that contains the link
    pub source_path: PathBuf,
    /// Display name for the source file (filename without extension)
    pub display_name: String,
}

/// In-memory index mapping file names to the files that reference them.
///
/// For small workspaces (≤50 files), backlinks are scanned on demand when
/// the active tab changes. For larger workspaces, the index is built once
/// on workspace load and updated incrementally on file save events.
#[derive(Debug, Default)]
pub struct BacklinkIndex {
    /// Map from lowercase filename (without extension) → list of files that link to it.
    /// The key is normalized (lowercase, no extension) to match wikilink resolution.
    index: HashMap<String, Vec<BacklinkEntry>>,
    /// Number of files in the workspace when the index was last built.
    /// Used to decide between on-demand scanning vs cached index.
    pub file_count: usize,
    /// Whether the full index has been built (for large workspaces).
    pub is_built: bool,
}

impl BacklinkIndex {
    /// Create a new empty backlink index.
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear the entire index.
    pub fn clear(&mut self) {
        self.index.clear();
        self.file_count = 0;
        self.is_built = false;
    }

    /// Get backlinks for a given filename.
    ///
    /// The filename is normalized to lowercase without extension for matching.
    pub fn get_backlinks(&self, filename: &str) -> Vec<BacklinkEntry> {
        let key = normalize_filename(filename);
        self.index.get(&key).cloned().unwrap_or_default()
    }

    /// Build the full index by scanning all workspace files.
    ///
    /// This reads every markdown file in the workspace and extracts wikilinks
    /// and standard markdown links, building a reverse mapping.
    pub fn build_from_files(&mut self, files: &[PathBuf]) {
        self.index.clear();
        self.file_count = files.len();

        let md_files: Vec<&PathBuf> = files
            .iter()
            .filter(|f| {
                f.extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.eq_ignore_ascii_case("md") || e.eq_ignore_ascii_case("markdown"))
                    .unwrap_or(false)
            })
            .collect();

        for file_path in &md_files {
            if let Ok(content) = std::fs::read_to_string(file_path) {
                let links = extract_links_from_content(&content);
                let source_display = file_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                for link_target in links {
                    let key = normalize_filename(&link_target);
                    let entry = BacklinkEntry {
                        source_path: file_path.to_path_buf(),
                        display_name: source_display.clone(),
                    };
                    self.index.entry(key).or_default().push(entry);
                }
            }
        }

        self.is_built = true;
        log::debug!(
            "Backlink index built: {} files scanned, {} targets indexed",
            md_files.len(),
            self.index.len()
        );
    }

    /// Incrementally update the index for a single file that was saved.
    ///
    /// Removes all old entries from this source file, then re-scans it.
    pub fn update_file(&mut self, file_path: &Path) {
        // Remove old entries from this source
        for entries in self.index.values_mut() {
            entries.retain(|e| e.source_path != file_path);
        }
        // Remove empty keys
        self.index.retain(|_, v| !v.is_empty());

        // Re-scan the file
        if let Ok(content) = std::fs::read_to_string(file_path) {
            let links = extract_links_from_content(&content);
            let source_display = file_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

            for link_target in links {
                let key = normalize_filename(&link_target);
                let entry = BacklinkEntry {
                    source_path: file_path.to_path_buf(),
                    display_name: source_display.clone(),
                };
                self.index.entry(key).or_default().push(entry);
            }
        }
    }

    /// Scan a subset of files on demand (for small workspaces or single-file mode).
    ///
    /// Returns backlinks for the given target filename by scanning the provided files.
    pub fn scan_on_demand(
        target_filename: &str,
        files: &[PathBuf],
        target_path: Option<&Path>,
    ) -> Vec<BacklinkEntry> {
        let target_key = normalize_filename(target_filename);
        let mut results = Vec::new();

        for file_path in files {
            // Skip non-markdown files
            let is_md = file_path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case("md") || e.eq_ignore_ascii_case("markdown"))
                .unwrap_or(false);
            if !is_md {
                continue;
            }

            // Skip the target file itself
            if let Some(tp) = target_path {
                if file_path == tp {
                    continue;
                }
            }

            if let Ok(content) = std::fs::read_to_string(file_path) {
                let links = extract_links_from_content(&content);
                for link in &links {
                    if normalize_filename(link) == target_key {
                        let display = file_path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("unknown")
                            .to_string();
                        results.push(BacklinkEntry {
                            source_path: file_path.clone(),
                            display_name: display,
                        });
                        break; // One match per file is enough
                    }
                }
            }
        }

        results
    }
}

/// Normalize a filename for backlink matching.
/// Removes `.md`/`.markdown` extension (case-insensitive) and converts to lowercase.
pub(crate) fn normalize_filename(name: &str) -> String {
    let name = name.trim();
    let lower = name.to_lowercase();
    let without_ext = lower
        .strip_suffix(".md")
        .or_else(|| lower.strip_suffix(".markdown"))
        .unwrap_or(&lower);
    without_ext.to_string()
}

/// Extract all link targets from markdown content.
///
/// Detects:
/// - `[[target]]` wikilinks
/// - `[[target|display]]` wikilinks with display text
/// - `[text](target.md)` standard markdown links to local .md files
pub(crate) fn extract_links_from_content(content: &str) -> Vec<String> {
    let mut targets = Vec::new();

    // Extract wikilinks: [[target]] and [[target|display]]
    let mut remaining = content;
    while let Some(open) = remaining.find("[[") {
        let after_open = &remaining[open + 2..];
        if let Some(close) = after_open.find("]]") {
            let inner = &after_open[..close];
            // Don't allow newlines inside wikilinks
            if !inner.contains('\n') && !inner.is_empty() {
                // Extract target (before | if present)
                let target = if let Some(pipe) = inner.find('|') {
                    inner[..pipe].trim()
                } else {
                    inner.trim()
                };
                if !target.is_empty() {
                    targets.push(target.to_string());
                }
            }
            remaining = &after_open[close + 2..];
        } else {
            remaining = after_open;
        }
    }

    // Extract standard markdown links: [text](target.md)
    // Only capture local .md file references (not http/https URLs)
    remaining = content;
    while let Some(open_paren) = remaining.find("](") {
        let after_paren = &remaining[open_paren + 2..];
        if let Some(close_paren) = after_paren.find(')') {
            let url = after_paren[..close_paren].trim();
            // Only match local .md links (not URLs)
            if !url.starts_with("http://")
                && !url.starts_with("https://")
                && !url.starts_with('#')
                && (url.ends_with(".md") || url.ends_with(".markdown"))
            {
                // Extract just the filename from the path
                let filename = Path::new(url)
                    .file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or(url);
                targets.push(filename.to_string());
            }
            remaining = &after_paren[close_paren + 1..];
        } else {
            break;
        }
    }

    targets
}
