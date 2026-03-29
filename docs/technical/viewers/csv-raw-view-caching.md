# CSV Raw View Caching

## Overview

Eliminates per-frame heap allocation in `CsvViewer::show_raw_view()` by caching the raw text string and rebuilding only when the content hash changes.

## Problem

`show_raw_view()` called `self.content.to_string()` every frame to pass to `egui::TextEdit::multiline()`. For a 10 MB CSV file at 60 fps, this caused ~600 MB/s of allocation churn and visible lag.

## Key Files

- `src/markdown/csv_viewer.rs` — `CsvViewerState` (cached fields), `show_raw_view()` (hash-guarded rebuild), `blake3_content_hash()` helper

## Implementation

Two fields added to `CsvViewerState`:

| Field | Type | Purpose |
|-------|------|---------|
| `raw_view_text` | `String` | Cached copy of content for `TextEdit` |
| `raw_view_hash` | `u64` | Blake3 hash (truncated) of content when text was last built |

`show_raw_view()` computes `blake3_content_hash(content)` and compares with `raw_view_hash`. The `.to_string()` allocation only happens when the hash differs (content changed).

### Hashing

Uses `blake3::hash()` truncated to `u64` via `from_le_bytes` — consistent with the Markdown AST cache pattern (Task 4). Unlike the table-view's `hash_content_bytes()` which samples head/tail for speed, this hashes the full content so mid-file edits are always detected.

### Cache Invalidation

`raw_view_text` and `raw_view_hash` are cleared to empty/zero by:
- `invalidate_cache()`
- `set_delimiter()`
- `clear_delimiter_override()`

## Performance

| Metric | Before | After |
|--------|--------|-------|
| Per-frame allocation (10 MB CSV) | ~10 MB | 0 bytes |
| Per-frame work | `to_string()` + dealloc | blake3 hash compare (u64 ==) |
| One-time cost on content change | — | `to_string()` + blake3 hash |
