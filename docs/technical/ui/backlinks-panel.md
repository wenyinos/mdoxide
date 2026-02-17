# Backlinks Panel

## Overview

The backlinks panel shows which files in the workspace link to the currently active file. It detects both `[[wikilinks]]` and standard `[text](file.md)` markdown links, providing a reverse-link graph for navigating between related documents.

The panel is accessible as a "Links" tab within the existing outline panel (alongside Outline, Statistics, and Hub tabs).

## Key Files

| File | Purpose |
|------|---------|
| `src/ui/backlinks_panel.rs` | `BacklinksPanel` UI component — renders the backlink list, handles click-to-navigate |
| `src/state.rs` | `BacklinkIndex`, `BacklinkEntry` — in-memory reverse-link index with build/update/scan methods |
| `src/ui/outline_panel.rs` | Hosts the "Links" tab in `OutlinePanelTab` enum, passes through to `BacklinksPanel` |
| `src/app/navigation.rs` | `refresh_backlinks()` — smart refresh strategy based on workspace size |
| `src/app/file_ops.rs` | Triggers incremental index update on file save, clears index on workspace close |
| `src/app/mod.rs` | `backlinks_panel` field on `FerriteApp`, tab-switch detection via `last_active_tab_for_backlinks` |

## Implementation Details

### Index Strategy (Adaptive)

The indexing approach scales with workspace size:

| Workspace Size | Strategy | Trigger |
|---|---|---|
| **≤50 files** | On-demand scan via `BacklinkIndex::scan_on_demand()` | Every tab switch |
| **>50 files** | Full `HashMap<filename, Vec<BacklinkEntry>>` cached in `AppState.backlink_index` | Built on first access, updated incrementally on file save |
| **Single-file mode** | Scans all markdown files in the current file's parent directory | Every tab switch |

### Link Detection

`extract_links_from_content()` in `state.rs` performs lightweight regex-free text scanning:

- **Wikilinks**: `[[target]]` and `[[target|display text]]` — extracts the target portion
- **Standard links**: `[text](file.md)` — only matches local `.md`/`.markdown` files (ignores `http://`, `https://`, and `#anchor` links)
- **Filename normalization**: `normalize_filename()` strips `.md`/`.markdown` extensions and lowercases for case-insensitive matching

### Reactivity

Backlinks refresh on two events:

1. **Tab switch** — detected by comparing `last_active_tab_for_backlinks` with the current active tab index
2. **File save** — `handle_save_file()` and `handle_save_as_file()` set `backlinks_need_refresh = true` and call `backlink_index.update_file()` for incremental updates

### UI Integration

The backlinks panel is rendered as a tab inside the existing `OutlinePanel`:

- `OutlinePanelTab::Backlinks` — new enum variant
- Tab bar shows "🔗 Links" between Statistics and Hub
- `BacklinksPanelOutput.navigate_to` propagated through `OutlinePanelOutput.backlink_navigate_to` → file open in `app/mod.rs`
- Click on a backlink entry opens that file as a tab via `state.open_file()`

## Dependencies Used

No new crate dependencies. Uses only standard library (`HashMap`, `PathBuf`, `fs::read_to_string`).

## Usage

1. Open a workspace containing markdown files with `[[wikilinks]]` or `[text](file.md)` links
2. Open the outline panel (toggle via ribbon or keyboard shortcut)
3. Click the "🔗 Links" tab
4. The panel shows all files that link to the currently active file
5. Click any backlink entry to navigate to that file

## Tests

11 unit tests in `src/state.rs`:

- `test_normalize_filename` — case-insensitive extension stripping
- `test_extract_wikilinks_from_content` — `[[target]]` and `[[target|display]]` extraction
- `test_extract_standard_links_from_content` — `[text](file.md)` extraction
- `test_extract_links_ignores_urls` — HTTP/HTTPS URLs filtered out
- `test_extract_links_ignores_anchors` — `#anchor` links filtered out
- `test_extract_mixed_links` — combined wikilink + standard link extraction
- `test_extract_unclosed_wikilink` — malformed `[[` handled gracefully
- `test_extract_empty_wikilink` — empty `[[]]` produces no link
- `test_backlink_index_get_and_build` — full index build from test files
- `test_backlink_index_update_file` — incremental update after file modification
- `test_backlink_scan_on_demand` — on-demand scanning for small workspaces
