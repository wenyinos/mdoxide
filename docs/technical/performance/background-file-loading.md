# Background File Loading

## Overview

Files larger than 5 MB are loaded in a background thread with a progress indicator, preventing the UI from freezing during I/O. Smaller files continue to use synchronous loading (fast enough to be imperceptible).

## Architecture

```
open_file_smart()
    ├── < 5 MB  → synchronous std::fs::read (existing path)
    └── ≥ 5 MB  → AppState::open_file_loading() → Tab::new_loading()
                    + spawn_file_loader() background thread
                        ↓ FileLoadMsg::Progress (per 1 MB chunk)
                        ↓ FileLoadMsg::Complete | FileLoadMsg::Error
                        ↓ (via std::sync::mpsc channel)
                    poll_file_load_messages() in update()
                        → Tab::finish_loading() | Tab::fail_loading()
```

### Key Types

| Type | Location | Purpose |
|------|----------|---------|
| `TabContent` | `state.rs` | Enum: `Loading(LoadingProgress)`, `Ready`, `Error(String)` |
| `LoadingProgress` | `state.rs` | Tracks path, bytes_loaded, total_size |
| `FileLoadMsg` | `app/types.rs` | Channel message: Progress, Complete, Error |

### Fields on `MDOxideApp`

| Field | Type | Purpose |
|-------|------|---------|
| `file_load_tx` | `mpsc::Sender<FileLoadMsg>` | Cloned into each background thread |
| `file_load_rx` | `mpsc::Receiver<FileLoadMsg>` | Polled each frame in `update()` |
| `loading_tasks` | `HashMap<usize, JoinHandle<()>>` | Active loaders keyed by tab ID |

### Tab Lifecycle

1. `Tab::new_loading(id, path, total_size)` — creates placeholder with `TabContent::Loading`
2. Background thread reads in 1 MB chunks, sends `FileLoadMsg::Progress`
3. On complete, sends `FileLoadMsg::Complete { bytes }` 
4. `poll_file_load_messages()` calls `tab.finish_loading(bytes, ...)` → decodes encoding, sets content, transitions to `TabContent::Ready`
5. On error or binary detection, `tab.fail_loading(error)` → `TabContent::Error`

### Cancellation

When a loading tab is closed, `cleanup_tab_state()` removes the `JoinHandle` from `loading_tasks`. The background thread continues briefly but its channel sends will fail silently (receiver dropped).

## UI

- **Loading state**: Centered spinner + progress bar showing MB loaded / total + percentage
- **Error state**: Warning icon + error message
- **Tab title**: `⏳ filename` during load, `⚠ filename` on error

## Constants

| Constant | Value | Location |
|----------|-------|----------|
| `BACKGROUND_LOAD_THRESHOLD` | 5 MB | `app/file_ops.rs` |
| `LOAD_CHUNK_SIZE` | 1 MB | `app/file_ops.rs` |

## Entry Points

`open_file_smart()` on `MDOxideApp` (in `file_ops.rs`) is the unified entry point. It's called from:
- File > Open dialog
- File tree click
- Drag and drop
- CLI arguments
- Quick switcher (Ctrl+P)
- Recent files
- Secondary instance protocol

Internal navigation paths (search, wikilinks, backlinks) continue to use the synchronous `AppState::open_file()` since those files are typically small markdown documents.
