//! Search match management for FerriteEditor.
//!
//! This module contains:
//! - Search match getters (`search_matches`, `search_match_count`, etc.)
//! - Search match setters (`set_search_matches`, `clear_search_matches`)
//! - Current match navigation (`current_search_match`, `set_current_search_match`)

use super::editor::{FerriteEditor, SearchMatch};
use super::highlights::MAX_DISPLAYED_MATCHES;

impl FerriteEditor {
    // ─────────────────────────────────────────────────────────────────────────────
    // Search Highlights (Phase 2)
    // ─────────────────────────────────────────────────────────────────────────────

    /// Returns the current search matches.
    #[must_use]
    pub fn search_matches(&self) -> &[SearchMatch] {
        &self.search_matches
    }

    /// Returns the total number of search matches (may exceed displayed limit).
    #[must_use]
    pub fn search_match_count(&self) -> usize {
        self.search_matches.len()
    }

    /// Returns the number of matches being displayed (capped at MAX_DISPLAYED_MATCHES).
    #[must_use]
    pub fn displayed_match_count(&self) -> usize {
        self.search_matches.len().min(MAX_DISPLAYED_MATCHES)
    }

    /// Returns whether there are more matches than can be displayed.
    #[must_use]
    pub fn has_more_matches(&self) -> bool {
        self.search_matches.len() > MAX_DISPLAYED_MATCHES
    }

    /// Returns the index of the current search match.
    #[must_use]
    pub fn current_search_match(&self) -> usize {
        self.current_search_match
    }

    /// Sets the search matches and optionally scrolls to the current match.
    ///
    /// # Arguments
    /// * `matches` - Vector of (start_byte, end_byte) positions for all matches
    /// * `current_match` - Index of the current/focused match
    /// * `scroll_to_match` - Whether to scroll to make the current match visible
    ///
    /// # Performance
    /// Pre-computes line numbers for each match using rope's O(log N) byte_to_line method.
    /// This avoids expensive per-frame line number calculations.
    pub fn set_search_matches(
        &mut self,
        matches: Vec<(usize, usize)>,
        current_match: usize,
        scroll_to_match: bool,
    ) {
        // Pre-compute line numbers for all matches
        self.search_matches = matches
            .into_iter()
            .map(|(start_byte, end_byte)| SearchMatch {
                start_byte,
                end_byte,
                line: self.byte_pos_to_line(start_byte),
            })
            .collect();

        self.current_search_match = current_match.min(self.search_matches.len().saturating_sub(1));
        self.scroll_to_search_match = scroll_to_match;
    }

    /// Clears all search matches.
    pub fn clear_search_matches(&mut self) {
        self.search_matches.clear();
        self.current_search_match = 0;
        self.scroll_to_search_match = false;
    }

    /// Sets the current search match index.
    ///
    /// # Arguments
    /// * `index` - Index of the match to focus (clamped to valid range)
    /// * `scroll_to_match` - Whether to scroll to make the match visible
    pub fn set_current_search_match(&mut self, index: usize, scroll_to_match: bool) {
        self.current_search_match = index.min(self.search_matches.len().saturating_sub(1));
        self.scroll_to_search_match = scroll_to_match;
    }
}
