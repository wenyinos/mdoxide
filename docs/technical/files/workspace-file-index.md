# Workspace File Index

## Overview

Background full-workspace file index used by **Quick File Switcher** (Ctrl+P) and **Search in Files** (Ctrl+Shift+F). Independent of the lazy-loaded sidebar file tree, so files in collapsed folders are discoverable without expanding them first.

## Key Files

| File | Purpose |
|------|---------|
| `src/workspaces/file_index.rs` | `WorkspaceFileIndex`, background walk, `collect_all_files` / `collect_markdown_files` |
| `src/ui/file_index_progress.rs` | Animated progress bar in quick switcher and search panel |
| `src/app/file_ops.rs` | `sync_workspace_file_index`, `workspace_files_for_search`, invalidation on tree refresh |
| `src/app/mod.rs` | Poll index each frame; repaint while indexing |
| `src/app/central_panel.rs` | Pass indexed file list + progress into switcher/search UI |

## Why a Separate Index?

The sidebar **file tree** uses **lazy loading** (`DirectoryNotLoaded`) so opening large folders does not freeze the UI. `Workspace::all_files()` only walks loaded tree nodes, so search and quick open previously missed files under unexpanded folders.

The file index runs a **`walkdir`** pass on a background thread with the same hidden-folder rules as the tree (`node_modules`, `.git`, dot entries except allowlisted names like `.env`). Backlinks reuse the same walk via `collect_markdown_files`.

## Lifecycle

1. **Workspace open** — `WorkspaceFileIndex::sync` starts a background thread for the workspace root.
2. **Incremental updates** — Batches of paths (~128 files) and progress ticks (~250 entries) are sent to the UI thread; results grow while scanning.
3. **Complete** — Full file list is kept in memory until the workspace closes or the index is invalidated.
4. **Invalidate** — File create/delete/rename (watcher), manual tree refresh, or file operations that change structure trigger a rebuild on the next `sync`.
5. **Workspace close** — Index reset.

## UI Progress

While indexing, Ctrl+P and Ctrl+Shift+F show an **animated progress bar** and **“Indexing… N files found”** (`workspace.indexing` in locales). The bar hides when indexing finishes. The app repaints every ~100 ms during indexing so the count updates on large trees.

## File List Selection

`FerriteApp::workspace_files_for_search`:

| State | File list |
|-------|-----------|
| Indexing, no batches yet | Tree fallback (`Workspace::all_files()` — root + expanded paths) |
| Indexing, partial batches | Accumulated index paths |
| Complete | Full index |

Quick switcher still merges **recent files** on top of the search pool.

## Hidden Paths

Walk filtering uses `file_tree::entry_hidden` (same as lazy scan). The **workspace root directory itself is never skipped**, even when its folder name starts with `.` (e.g. temp or dot-prefixed project roots).

## Tests

```bash
cargo test workspaces::file_index::
```

Unit tests cover nested folders and `node_modules` exclusion.
