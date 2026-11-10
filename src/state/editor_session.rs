//! Editor-session state: selections, multi-cursor and code folding.
//!
//! Extracted from `state.rs` as part of P1-04. These are pure, independently
//! testable view-model types shared by `Tab` and the editor widgets.

use super::char_index_to_line_col;

// ─────────────────────────────────────────────────────────────────────────────
// Multi-Cursor Support
// ─────────────────────────────────────────────────────────────────────────────

/// A selection or cursor position in the editor.
///
/// A Selection represents either:
/// - A cursor with no selection (when `anchor == head`)
/// - A text selection (when `anchor != head`)
///
/// The anchor is the fixed end of the selection (where selection started),
/// and the head is the moving end (current cursor position).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
    /// The fixed end of the selection (where selection started), as a character index.
    pub anchor: usize,
    /// The moving end of the selection (current cursor position), as a character index.
    pub head: usize,
    /// Preferred visual column for vertical movement (preserved during up/down navigation).
    /// This is in visual columns, accounting for tabs and wide characters.
    pub preferred_column: Option<usize>,
}

impl Selection {
    /// Create a new cursor with no selection at the given character index.
    pub fn cursor(pos: usize) -> Self {
        Self {
            anchor: pos,
            head: pos,
            preferred_column: None,
        }
    }

    /// Create a new selection from anchor to head.
    pub fn new(anchor: usize, head: usize) -> Self {
        Self {
            anchor,
            head,
            preferred_column: None,
        }
    }

    /// Check if this is a cursor with no selection.
    pub fn is_cursor(&self) -> bool {
        self.anchor == self.head
    }

    /// Check if this is a selection (has a range).
    pub fn is_selection(&self) -> bool {
        self.anchor != self.head
    }

    /// Get the start of the selection (smaller index).
    pub fn start(&self) -> usize {
        self.anchor.min(self.head)
    }

    /// Get the end of the selection (larger index).
    pub fn end(&self) -> usize {
        self.anchor.max(self.head)
    }

    /// Get the selection range as (start, end) tuple.
    pub fn range(&self) -> (usize, usize) {
        (self.start(), self.end())
    }

    /// Get the length of the selection.
    pub fn len(&self) -> usize {
        self.end() - self.start()
    }

    /// Check if the selection is empty (cursor with no selection).
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Check if this selection contains or overlaps with a position.
    pub fn contains(&self, pos: usize) -> bool {
        pos >= self.start() && pos <= self.end()
    }

    /// Check if this selection overlaps with another selection.
    pub fn overlaps(&self, other: &Selection) -> bool {
        self.start() < other.end() && other.start() < self.end()
    }

    /// Merge this selection with another overlapping selection.
    pub fn merge(&self, other: &Selection) -> Selection {
        Selection {
            anchor: self.start().min(other.start()),
            head: self.end().max(other.end()),
            preferred_column: self.preferred_column.or(other.preferred_column),
        }
    }

    /// Move the cursor/selection by an offset.
    pub fn offset(self, delta: isize) -> Selection {
        let new_anchor = ((self.anchor as isize) + delta).max(0) as usize;
        let new_head = ((self.head as isize) + delta).max(0) as usize;
        Selection {
            anchor: new_anchor,
            head: new_head,
            preferred_column: self.preferred_column,
        }
    }

    /// Collapse the selection to a cursor at the head position.
    pub fn collapse_to_head(self) -> Selection {
        Selection::cursor(self.head)
    }

    /// Collapse the selection to a cursor at the start position.
    pub fn collapse_to_start(self) -> Selection {
        Selection::cursor(self.start())
    }

    /// Collapse the selection to a cursor at the end position.
    pub fn collapse_to_end(self) -> Selection {
        Selection::cursor(self.end())
    }
}

impl Default for Selection {
    fn default() -> Self {
        Self::cursor(0)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Transient Highlight (Search Result Navigation)
// ─────────────────────────────────────────────────────────────────────────────

/// A temporary highlight for search result navigation.
///
/// This highlight is applied when the user clicks a search-in-files result,
/// and is automatically cleared on scroll, edit, or any mouse click.
/// It is independent of text selection and multi-cursor state.
#[derive(Debug, Clone, Default)]
pub struct TransientHighlight {
    /// The character range to highlight (start, end).
    /// None if no highlight is active.
    range: Option<(usize, usize)>,
    /// Guard flag to ignore the programmatic scroll that positions the match.
    /// Set to true when the highlight is first applied, cleared after one scroll event.
    ignore_next_scroll: bool,
}

impl TransientHighlight {
    /// Create a new transient highlight (initially inactive).
    pub fn new() -> Self {
        Self {
            range: None,
            ignore_next_scroll: false,
        }
    }

    /// Set the highlight range.
    ///
    /// This also sets the guard flag to ignore the programmatic scroll.
    pub fn set(&mut self, start: usize, end: usize) {
        self.range = Some((start, end));
        self.ignore_next_scroll = true;
    }

    /// Clear the highlight.
    pub fn clear(&mut self) {
        self.range = None;
        self.ignore_next_scroll = false;
    }

    /// Check if a highlight is active.
    pub fn is_active(&self) -> bool {
        self.range.is_some()
    }

    /// Get the highlight range if active.
    pub fn range(&self) -> Option<(usize, usize)> {
        self.range
    }

    /// Handle a scroll event.
    ///
    /// If this is the first scroll after applying the highlight (the programmatic
    /// scroll to position the match), ignore it. Otherwise, clear the highlight.
    ///
    /// Returns true if the highlight was cleared.
    pub fn on_scroll(&mut self) -> bool {
        if self.range.is_none() {
            return false;
        }

        if self.ignore_next_scroll {
            self.ignore_next_scroll = false;
            return false;
        }

        self.clear();
        true
    }

    /// Handle an edit event. Clears the highlight.
    ///
    /// Returns true if the highlight was cleared.
    pub fn on_edit(&mut self) -> bool {
        if self.range.is_some() {
            self.clear();
            true
        } else {
            false
        }
    }

    /// Handle a mouse click event. Clears the highlight.
    ///
    /// Returns true if the highlight was cleared.
    pub fn on_click(&mut self) -> bool {
        if self.range.is_some() {
            self.clear();
            true
        } else {
            false
        }
    }
}

/// Multi-cursor state - a collection of selections/cursors.
///
/// Invariants:
/// - Always contains at least one selection
/// - Selections are sorted by start position
/// - Selections do not overlap (merged if they would)
#[derive(Debug, Clone, Default)]
pub struct MultiCursor {
    /// All active selections/cursors (sorted, non-overlapping).
    selections: Vec<Selection>,
    /// Index of the primary selection (for status bar display, scroll anchoring).
    primary_index: usize,
}

impl MultiCursor {
    /// Create a new multi-cursor with a single cursor at position 0.
    pub fn new() -> Self {
        Self {
            selections: vec![Selection::cursor(0)],
            primary_index: 0,
        }
    }

    /// Create a multi-cursor with a single cursor at the given position.
    pub fn single(pos: usize) -> Self {
        Self {
            selections: vec![Selection::cursor(pos)],
            primary_index: 0,
        }
    }

    /// Create a multi-cursor with a single selection.
    pub fn from_selection(selection: Selection) -> Self {
        Self {
            selections: vec![selection],
            primary_index: 0,
        }
    }

    /// Get all selections.
    pub fn selections(&self) -> &[Selection] {
        &self.selections
    }

    /// Get the number of cursors/selections.
    pub fn len(&self) -> usize {
        self.selections.len()
    }

    /// Check if there's only a single cursor/selection.
    pub fn is_single(&self) -> bool {
        self.selections.len() == 1
    }

    /// Check if this is empty (should never be true due to invariants).
    pub fn is_empty(&self) -> bool {
        self.selections.is_empty()
    }

    /// Get the primary selection (for status bar, scroll anchoring).
    pub fn primary(&self) -> &Selection {
        self.selections
            .get(self.primary_index)
            .unwrap_or(&self.selections[0])
    }

    /// Get a mutable reference to the primary selection.
    pub fn primary_mut(&mut self) -> &mut Selection {
        let idx = self
            .primary_index
            .min(self.selections.len().saturating_sub(1));
        &mut self.selections[idx]
    }

    /// Get the primary index.
    pub fn primary_index(&self) -> usize {
        self.primary_index
    }

    /// Set the primary selection by index.
    pub fn set_primary(&mut self, index: usize) {
        if index < self.selections.len() {
            self.primary_index = index;
        }
    }

    /// Add a new selection, maintaining invariants.
    pub fn add(&mut self, selection: Selection) {
        self.selections.push(selection);
        self.normalize();
    }

    /// Replace all selections with a single one.
    pub fn set_single(&mut self, selection: Selection) {
        self.selections.clear();
        self.selections.push(selection);
        self.primary_index = 0;
    }

    /// Clear to a single cursor at position 0.
    pub fn clear(&mut self) {
        self.selections.clear();
        self.selections.push(Selection::cursor(0));
        self.primary_index = 0;
    }

    /// Normalize selections: sort and merge overlapping.
    fn normalize(&mut self) {
        if self.selections.is_empty() {
            self.selections.push(Selection::cursor(0));
            self.primary_index = 0;
            return;
        }

        // Sort by start position
        self.selections.sort_by_key(|s| s.start());

        // Merge overlapping selections
        let mut merged: Vec<Selection> = Vec::with_capacity(self.selections.len());
        for sel in self.selections.drain(..) {
            if let Some(last) = merged.last_mut() {
                if last.overlaps(&sel) || last.end() == sel.start() {
                    *last = last.merge(&sel);
                    continue;
                }
            }
            merged.push(sel);
        }

        self.selections = merged;

        // Ensure primary_index is valid
        if self.primary_index >= self.selections.len() {
            self.primary_index = self.selections.len().saturating_sub(1);
        }
    }

    /// Apply an offset adjustment to all selections after a given position.
    /// Used after insertions/deletions to keep cursor positions valid.
    pub fn adjust_after(&mut self, pos: usize, delta: isize) {
        for sel in &mut self.selections {
            if sel.anchor >= pos {
                sel.anchor = ((sel.anchor as isize) + delta).max(0) as usize;
            }
            if sel.head >= pos {
                sel.head = ((sel.head as isize) + delta).max(0) as usize;
            }
        }
        self.normalize();
    }

    /// Get legacy cursor position (line, column) from primary selection.
    /// Used for backwards compatibility with status bar, etc.
    pub fn cursor_position(&self, text: &str) -> (usize, usize) {
        let pos = self.primary().head;
        char_index_to_line_col(text, pos)
    }

    /// Get legacy selection range from primary selection.
    /// Returns None if primary is a cursor with no selection.
    pub fn selection_range(&self) -> Option<(usize, usize)> {
        let primary = self.primary();
        if primary.is_selection() {
            Some(primary.range())
        } else {
            None
        }
    }

    /// Iterate over all selections.
    pub fn iter(&self) -> impl Iterator<Item = &Selection> {
        self.selections.iter()
    }

    /// Iterate mutably over all selections.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Selection> {
        self.selections.iter_mut()
    }
}
// ─────────────────────────────────────────────────────────────────────────────
// Code Folding
// ─────────────────────────────────────────────────────────────────────────────

/// The kind/type of a foldable region.
///
/// Different fold kinds have different detection rules and may be
/// toggled on/off independently in settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FoldKind {
    /// Markdown heading (## Section) - folds until next heading of same/higher level
    Heading(u8), // level 1-6
    /// Fenced code block (```...```)
    CodeBlock,
    /// List hierarchy (nested list items)
    List,
    /// Indentation-based region (for JSON/YAML/structured files)
    Indentation,
}

impl FoldKind {
    /// Get a display name for this fold kind.
    pub fn display_name(&self) -> &'static str {
        match self {
            FoldKind::Heading(_) => "Heading",
            FoldKind::CodeBlock => "Code Block",
            FoldKind::List => "List",
            FoldKind::Indentation => "Indentation",
        }
    }

    /// Get an icon for this fold kind.
    pub fn icon(&self) -> &'static str {
        match self {
            FoldKind::Heading(_) => "§",
            FoldKind::CodeBlock => "{ }",
            FoldKind::List => "•",
            FoldKind::Indentation => "⤵",
        }
    }
}

/// A unique identifier for a fold region.
pub type FoldId = u32;

/// A foldable region in a document.
///
/// Represents a contiguous range of lines that can be collapsed/expanded.
#[derive(Debug, Clone, PartialEq)]
pub struct FoldRegion {
    /// Unique identifier for this fold region
    pub id: FoldId,
    /// Starting line (0-indexed, inclusive)
    pub start_line: usize,
    /// Ending line (0-indexed, inclusive)
    pub end_line: usize,
    /// The kind of fold region
    pub kind: FoldKind,
    /// Whether this region is currently collapsed
    pub collapsed: bool,
    /// Preview text to show when collapsed (e.g., first line content)
    pub preview_text: String,
}

impl FoldRegion {
    /// Create a new fold region.
    pub fn new(id: FoldId, start_line: usize, end_line: usize, kind: FoldKind) -> Self {
        Self {
            id,
            start_line,
            end_line,
            kind,
            collapsed: false,
            preview_text: String::new(),
        }
    }

    /// Create a new fold region with preview text.
    pub fn with_preview(
        id: FoldId,
        start_line: usize,
        end_line: usize,
        kind: FoldKind,
        preview: String,
    ) -> Self {
        Self {
            id,
            start_line,
            end_line,
            kind,
            collapsed: false,
            preview_text: preview,
        }
    }

    /// Get the number of lines in this fold region.
    pub fn line_count(&self) -> usize {
        self.end_line.saturating_sub(self.start_line) + 1
    }

    /// Get the number of hidden lines when collapsed.
    pub fn hidden_line_count(&self) -> usize {
        if self.collapsed {
            self.end_line.saturating_sub(self.start_line)
        } else {
            0
        }
    }

    /// Check if a line is within this fold region.
    pub fn contains_line(&self, line: usize) -> bool {
        line >= self.start_line && line <= self.end_line
    }

    /// Check if a line is hidden by this fold (collapsed and not the start line).
    pub fn hides_line(&self, line: usize) -> bool {
        self.collapsed && line > self.start_line && line <= self.end_line
    }

    /// Toggle the collapsed state.
    pub fn toggle(&mut self) {
        self.collapsed = !self.collapsed;
    }

    /// Adjust line numbers after an edit.
    ///
    /// `edit_line` is where the edit occurred, `delta` is the number of lines added (positive)
    /// or removed (negative).
    ///
    /// Returns `true` if the region is still valid, `false` if it should be removed.
    pub fn adjust_for_edit(&mut self, edit_line: usize, delta: isize) -> bool {
        // If edit is after this region, no change needed
        if edit_line > self.end_line {
            return true;
        }

        // If edit is within the region, adjust end line
        if edit_line >= self.start_line && edit_line <= self.end_line {
            let new_end = (self.end_line as isize) + delta;
            if new_end < (self.start_line as isize) {
                // Region collapsed to invalid state
                return false;
            }
            self.end_line = new_end as usize;
            return true;
        }

        // Edit is before this region, shift both lines
        let new_start = (self.start_line as isize) + delta;
        let new_end = (self.end_line as isize) + delta;

        if new_start < 0 || new_end < new_start {
            return false;
        }

        self.start_line = new_start as usize;
        self.end_line = new_end as usize;
        true
    }
}

/// State manager for all fold regions in a document.
///
/// Maintains an ordered list of fold regions and provides efficient
/// queries for rendering and interaction.
#[derive(Debug, Clone, Default)]
pub struct FoldState {
    /// All fold regions, sorted by start_line
    regions: Vec<FoldRegion>,
    /// Counter for generating unique fold IDs
    next_id: FoldId,
    /// Whether fold state needs recomputation
    dirty: bool,
}

impl FoldState {
    /// Create a new empty fold state.
    pub fn new() -> Self {
        Self {
            regions: Vec::new(),
            next_id: 1,
            dirty: true,
        }
    }

    /// Get all fold regions.
    pub fn regions(&self) -> &[FoldRegion] {
        &self.regions
    }

    /// Get mutable access to all fold regions.
    pub fn regions_mut(&mut self) -> &mut Vec<FoldRegion> {
        &mut self.regions
    }

    /// Check if there are any fold regions.
    pub fn is_empty(&self) -> bool {
        self.regions.is_empty()
    }

    /// Get the number of fold regions.
    pub fn len(&self) -> usize {
        self.regions.len()
    }

    /// Check if fold state needs recomputation.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Mark fold state as needing recomputation.
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Mark fold state as clean (just recomputed).
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    /// Generate a new unique fold ID.
    pub fn next_id(&mut self) -> FoldId {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        id
    }

    /// Clear all fold regions.
    pub fn clear(&mut self) {
        self.regions.clear();
        self.dirty = true;
    }

    /// Replace all fold regions with new ones.
    pub fn set_regions(&mut self, regions: Vec<FoldRegion>) {
        self.regions = regions;
        self.sort_regions();
        self.dirty = false;
    }

    /// Add a fold region, maintaining sort order.
    pub fn add_region(&mut self, region: FoldRegion) {
        self.regions.push(region);
        self.sort_regions();
    }

    /// Sort regions by start line.
    fn sort_regions(&mut self) {
        self.regions.sort_by_key(|r| r.start_line);
    }

    /// Get the fold region that starts on a given line.
    pub fn region_at_line(&self, line: usize) -> Option<&FoldRegion> {
        self.regions.iter().find(|r| r.start_line == line)
    }

    /// Get mutable access to the fold region that starts on a given line.
    pub fn region_at_line_mut(&mut self, line: usize) -> Option<&mut FoldRegion> {
        self.regions.iter_mut().find(|r| r.start_line == line)
    }

    /// Get the fold region by ID.
    pub fn region_by_id(&self, id: FoldId) -> Option<&FoldRegion> {
        self.regions.iter().find(|r| r.id == id)
    }

    /// Get mutable access to a fold region by ID.
    pub fn region_by_id_mut(&mut self, id: FoldId) -> Option<&mut FoldRegion> {
        self.regions.iter_mut().find(|r| r.id == id)
    }

    /// Toggle the fold state at a given line.
    ///
    /// Returns `true` if a fold was toggled.
    pub fn toggle_at_line(&mut self, line: usize) -> bool {
        if let Some(region) = self.region_at_line_mut(line) {
            region.toggle();
            true
        } else {
            false
        }
    }

    /// Check if a line is hidden by any collapsed fold.
    pub fn is_line_hidden(&self, line: usize) -> bool {
        self.regions.iter().any(|r| r.hides_line(line))
    }

    /// Get the fold region that hides a given line.
    pub fn fold_hiding_line(&self, line: usize) -> Option<&FoldRegion> {
        self.regions.iter().find(|r| r.hides_line(line))
    }

    /// Expand any fold that contains the given line (to reveal it).
    ///
    /// Returns `true` if any fold was expanded.
    pub fn reveal_line(&mut self, line: usize) -> bool {
        let mut revealed = false;
        for region in &mut self.regions {
            if region.hides_line(line) {
                region.collapsed = false;
                revealed = true;
            }
        }
        revealed
    }

    /// Fold all regions of a specific kind.
    pub fn fold_all_of_kind(&mut self, kind_matches: impl Fn(&FoldKind) -> bool) {
        for region in &mut self.regions {
            if kind_matches(&region.kind) {
                region.collapsed = true;
            }
        }
    }

    /// Unfold all regions.
    pub fn unfold_all(&mut self) {
        for region in &mut self.regions {
            region.collapsed = false;
        }
    }

    /// Fold all regions.
    pub fn fold_all(&mut self) {
        for region in &mut self.regions {
            region.collapsed = true;
        }
    }

    /// Get the total number of hidden lines.
    pub fn hidden_line_count(&self) -> usize {
        self.regions.iter().map(|r| r.hidden_line_count()).sum()
    }

    /// Get all lines that have fold indicators (start lines of regions).
    pub fn fold_indicator_lines(&self) -> Vec<(usize, bool)> {
        self.regions
            .iter()
            .map(|r| (r.start_line, r.collapsed))
            .collect()
    }

    /// Map a visual line (accounting for folds) to the actual document line.
    ///
    /// Visual lines skip over hidden (folded) content.
    pub fn visual_to_document_line(&self, visual_line: usize) -> usize {
        let mut doc_line = 0;
        let mut vis_line = 0;

        while vis_line < visual_line {
            if !self.is_line_hidden(doc_line) {
                vis_line += 1;
            }
            doc_line += 1;
        }

        // Skip any hidden lines at the target position
        while self.is_line_hidden(doc_line) {
            doc_line += 1;
        }

        doc_line
    }

    /// Map a document line to the visual line (accounting for folds).
    pub fn document_to_visual_line(&self, doc_line: usize) -> usize {
        let mut vis_line = 0;
        for line in 0..doc_line {
            if !self.is_line_hidden(line) {
                vis_line += 1;
            }
        }
        vis_line
    }

    /// Adjust all fold regions after a document edit.
    ///
    /// `edit_line` is where the edit occurred, `delta` is the number of lines
    /// added (positive) or removed (negative).
    pub fn adjust_for_edit(&mut self, edit_line: usize, delta: isize) {
        self.regions
            .retain_mut(|r| r.adjust_for_edit(edit_line, delta));
        self.dirty = true;
    }

    /// Get the number of collapsed folds.
    pub fn collapsed_count(&self) -> usize {
        self.regions.iter().filter(|r| r.collapsed).count()
    }
}
