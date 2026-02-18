# Vim Mode

## Overview

Optional modal editing mode that adds Vim-style keybindings to FerriteEditor. Disabled by default to preserve standard editing behavior. When enabled, provides Normal/Insert/Visual/Visual Line modes with essential Vim commands.

## Key Files

- `src/config/settings.rs` - `vim_mode: bool` setting (default: `false`)
- `src/editor/ferrite/vim.rs` - `VimState` struct, `VimMode` enum, `handle_key()` dispatcher
- `src/editor/ferrite/editor.rs` - Event loop interception, `set_vim_mode()`/`vim_mode()` methods
- `src/editor/widget.rs` - `vim_mode` builder method, `vim_mode_label` on `EditorOutput`
- `src/app/central_panel.rs` - Propagates setting to EditorWidget, surfaces mode to UiState
- `src/app/status_bar.rs` - Renders `[NORMAL]`/`[INSERT]`/`[VISUAL]`/`[V-LINE]` indicator
- `src/ui/settings.rs` - Vim Mode checkbox in Editor settings section
- `src/state.rs` - `vim_mode_indicator: Option<&'static str>` on `UiState`

## Architecture

### Data Flow

```
Settings.vim_mode ──► EditorWidget.vim_mode() ──► FerriteEditor.set_vim_mode()
                                                        │
                                                   VimState.handle_key()
                                                        │
                                              VimMode (label: &'static str)
                                                        │
                                              EditorOutput.vim_mode_label
                                                        │
                                              UiState.vim_mode_indicator
                                                        │
                                              status_bar.rs (renders indicator)
```

### Modal State Machine

`VimState` manages the current mode and pending operations:

- **Normal**: Default mode. Keys are interpreted as commands (motions, operators, mode switches).
- **Insert**: Text input mode. All keys pass through to normal editor handling. `Esc` returns to Normal.
- **Visual**: Character-wise selection. Motions extend selection. `d`/`y` operate on selection.
- **Visual Line**: Line-wise selection. Similar to Visual but selects full lines.

### Event Loop Integration

In `FerriteEditor::ui()`, when `vim_mode_enabled` is true:

1. `Event::Key` events are intercepted by `VimState::handle_key()` before normal processing.
2. Returns `VimKeyResult::Handled(result)` if Vim consumed the key, `Passthrough` if not.
3. `Event::Text` events are suppressed in Normal/Visual modes via `should_insert_text()`.
4. Standard egui shortcuts (Ctrl+C, Ctrl+V, etc.) are not intercepted by Vim.

## Implemented Commands

### Normal Mode

| Key | Action |
|-----|--------|
| `h`/`j`/`k`/`l` | Left/down/up/right movement |
| `w`/`b`/`e` | Word forward/backward/end |
| `0`/`$` | Line start/end |
| `gg`/`G` | File start/end |
| `i`/`a` | Insert before/after cursor |
| `I`/`A` | Insert at line start/end |
| `o`/`O` | Open line below/above |
| `x` | Delete character |
| `dd` | Delete line |
| `yy` | Yank line |
| `D` | Delete to end of line |
| `C` | Change to end of line |
| `p`/`P` | Paste after/before |
| `v`/`V` | Enter Visual/Visual Line mode |
| `u` | Undo |
| `Ctrl+R` | Redo |
| `{count}{motion}` | Repeat count (e.g., `3j` = move down 3) |

### Visual/Visual Line Mode

| Key | Action |
|-----|--------|
| Motions | Extend selection |
| `d` | Delete selection |
| `y` | Yank selection |
| `Esc` | Return to Normal |

## Dependencies Used

No additional crates. Built entirely on existing `TextBuffer`, `Cursor`, and `Selection` types.

## Usage

1. Open Settings (gear icon or Ctrl+,)
2. In the Editor section, check "Vim Mode"
3. Status bar shows `[NORMAL]` when active
4. Press `i` to enter Insert mode, `Esc` to return to Normal
5. Disable the checkbox to return to standard editing
