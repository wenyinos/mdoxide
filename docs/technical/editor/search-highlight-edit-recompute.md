# Search Highlight Recomputation After Document Edits

## Problem

When the user edits a document while the Ctrl+F find panel is open, search highlight rectangles drift out of alignment with actual match positions. The yellow highlight boxes shift horizontally and vertically, no longer overlaying the correct text.

## Root Cause

`FindState.matches` stores search results as byte-position pairs `(start_byte, end_byte)` computed against the document text at search time. When the user inserts or deletes text (e.g., pressing Enter), byte positions of all subsequent matches shift, but `find_matches()` was only called when:

1. The search term changed (debounced via `dialogs.rs`)
2. The find panel was first opened (`find_replace.rs`)
3. After replace operations

No recomputation was triggered by document edits, so stale byte offsets were passed to the highlight renderer each frame.

## Fix

In `src/app/central_panel.rs`, after each editor's content-change detection (`editor_output.changed == true`), if the find panel is active with a non-empty search term, `find_matches()` is re-run against the current `tab.content`.

### Affected View Modes

| View Mode | Detection | Location |
|-----------|-----------|----------|
| Raw | `editor_output.changed` from `EditorWidget` | After tab borrow ends (~line 857) |
| Split | `editor_output.changed` from left raw editor OR `md_editor_output.changed` from right rendered pane | After both panes render (~line 1740) |
| Rendered | `editor_output.changed` from `MarkdownEditor` | After tab borrow ends (~line 1960) |

### Pattern

Each view mode uses a flag variable to capture the change signal inside the mutable `tab` borrow scope, then recomputes after the borrow ends:

```rust
let mut content_changed_in_editor = false;
if let Some(tab) = self.state.active_tab_mut() {
    // ... editor rendering ...
    if editor_output.changed {
        content_changed_in_editor = true;
    }
}
// After tab borrow ends:
if content_changed_in_editor
    && self.state.ui.show_find_replace
    && !self.state.ui.find_state.search_term.is_empty()
{
    if let Some(content) = self.state.active_tab().map(|t| t.content.clone()) {
        self.state.ui.find_state.find_matches(&content);
    }
}
```

## Performance

- **Only runs on actual edits** — gated by `editor_output.changed`, not per-frame
- **O(N) text clone + regex search** — same cost as the existing debounced search path
- `find_matches()` uses compiled regex internally, which is efficient for typical document sizes
- For 100K+ line files, the search is still user-initiated (typing causes the edit), keeping within the O(N) user-initiated tier from the performance rules

## Key Files

| File | Role |
|------|------|
| `src/app/central_panel.rs` | Recomputation trigger after edit detection in all 3 view modes |
| `src/editor/find_replace.rs` | `FindState::find_matches()` — the search engine |
| `src/editor/widget.rs` | `SearchHighlights` struct passed to `FerriteEditor` |
| `src/editor/ferrite/search.rs` | `set_search_matches()` — converts byte ranges to `SearchMatch` with pre-computed line numbers |
| `src/editor/ferrite/highlights.rs` | `render_search_highlights()` — paints highlight rectangles |
