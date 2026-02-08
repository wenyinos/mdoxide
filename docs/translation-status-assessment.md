# Translation Status Assessment (Post v0.2.6.1)

**Purpose:** List user-facing strings that are **not** currently using the translation system (`t!("key")` / `locales/en.yaml`). No code changes—assessment only. Use this to add keys to `locales/en.yaml` and replace hardcoded strings with `t!(...)` for Weblate.

**Reference:** i18n uses `rust_i18n::t!`, keys in `locales/en.yaml`; see [Internationalization](technical/config/i18n.md).

---

## 1. Terminal Panel (`src/ui/terminal_panel.rs`)

All of the following are hardcoded (terminal was added in v0.2.6.1 and has no `terminal.*` keys in `en.yaml`).

### Labels / messages
| String | Context |
|--------|--------|
| `"Terminal not found"` | Shown when terminal widget is missing |
| `"New Terminal Here"` | Context menu |
| `"Maximize Active Pane"` | Context menu |
| `"Pop out terminal"` | Context menu |
| `"Split Horizontal"` | Context menu |
| `"Split Vertical"` | Context menu |
| `"Rename"` | Context menu |
| `"Close"` | Context menu |
| `"Close Pane"` | Context menu |
| `"Close Others"` | Context menu |
| `"Scatter to Windows"` | Context menu |
| `"Watching: {}"` | Watch mode label (path) |
| `"Stop Watching"` | Context menu |
| `"Watch Workspace Root"` | Context menu |
| `"Export Output as HTML..."` | Context menu |
| `"(No workspace root set)"` | Layout/workspace UI |
| `"No macros saved."` | Macros menu |
| `"Close terminal panel"` | Tooltip on close button |
| `"This terminal has a running process."` | Close confirmation dialog |
| `"Closing it will terminate the process."` | Close confirmation dialog |
| `"Close Terminal"` | Button in close confirmation |
| `"Cancel"` | Button in close confirmation |
| `"No terminal. Click + to create one."` | Empty state message |
| `"Restore ⤢"` | Button in maximized pane UI |

### Window / dialog titles
| String | Context |
|--------|--------|
| `"Close Terminal?"` | Confirmation window title |

### Shell / layout buttons (toolbar / menus)
| String | Context |
|--------|--------|
| `"PowerShell"` | New terminal shell option |
| `"CMD"` | New terminal shell option |
| `"WSL"` | New terminal shell option |
| `"2 Columns"` | Layout preset |
| `"2 Rows"` | Layout preset |
| `"2x2 Grid"` | Layout preset |
| `"Save Current Layout..."` | Layout menu |
| `"Load Layout..."` | Layout menu |
| `"Scatter All Tabs"` | Workspaces menu |
| `"Save Workspace..."` | Workspaces menu |
| `"Load Workspace..."` | Workspaces menu |
| `"Save as Workspace Layout"` | Button (when saving to .ferrite) |
| `"Layouts"` | Menu button |
| `"Workspaces"` | Menu button |
| `"Watch Mode"` | Menu button |
| `"Macros"` | Menu button |

### Error / recovery hints (user-facing from `report_error` / `recovery_hint`)
| String | Context |
|--------|--------|
| `"Terminal: {} — {}"` | Toast format (operation + hint) |
| `"Check that the shell is installed and the path is correct"` | Recovery hint |
| `"PTY allocation failed. Try closing other terminals or restarting the app"` | Recovery hint |
| `"Permission denied. Check that you have access to run the shell"` | Recovery hint |
| `"Terminal I/O failed. Try closing and reopening the terminal"` | Recovery hint |
| `"Failed to modify terminal layout"` | Recovery hint |
| `"An unexpected error occurred. Try restarting the terminal"` | Recovery hint |
| `"Terminal '{}' process exited"` | Toast when terminal process exits |

### Internal / fallback (still visible in UI)
| String | Context |
|--------|--------|
| `"Tab {}"` | Default tab name when saving layout |
| `"Workspace"` | Default layout/workspace name |
| `"Terminal"` | Fallback title when terminal has no title |
| `"Terminal Layout"` | File dialog filter name |
| `"Ferrite Workspace"` | File dialog filter name |
| `"Custom Layout"` | Name when saving custom layout |

---

## 2. Settings – Terminal section (`src/ui/settings.rs`)

There is an explicit `// TODO: i18n` next to the Terminal heading. All terminal-related settings are hardcoded.

### Heading
| String | Context |
|--------|--------|
| `"Terminal"` | Section heading (TODO: i18n) |

### Checkboxes and tooltips
| String | Context |
|--------|--------|
| `"Auto-load Layout"` | Checkbox label |
| `"Automatically load 'terminal_layout.json' from project root"` | Tooltip |
| `"Enable Sound on Prompt"` | Checkbox label |
| `"Play a notification sound when terminal detects a prompt (waiting for input)"` | Tooltip |
| `"Focus Terminal on Prompt"` | Checkbox label |
| `"Automatically focus a terminal when it transitions from running to waiting for input"` | Tooltip |
| `"Automatically copy text to clipboard when selecting with mouse"` | Tooltip (terminal selection) |

### Other
| String | Context |
|--------|--------|
| `"Monitor {}:"` | Monitor label (format with index) |
| `"({}x{} at {},{})"` | Monitor geometry (format) |
| `"{}px"` | Font size / scrollback (numeric) |
| `"{:.0}%"` | Opacity (numeric) |

---

## 3. Productivity Panel (`src/ui/productivity_panel.rs`)

Productivity Hub was added in v0.2.6.1; there are no `productivity.*` or `tasks.*` / `pomodoro.*` / `notes.*` keys in `en.yaml`.

### Window / headings
| String | Context |
|--------|--------|
| `"Productivity Hub"` | Floating window title |
| `"Tasks"` | Section heading |
| `"Pomodoro Timer"` | Section heading |
| `"Quick Notes"` | Section heading |

### Tasks section
| String | Context |
|--------|--------|
| `"Open a workspace to enable task and note persistence"` | Message when no workspace |
| `"{}/{} completed"` | Progress label (completed/total) |
| `"Type task or - [ ] task..."` | Hint text |
| `"Add"` | Button |
| `"Tip: Use - [ ] for checkbox, ! or !! for priority"` | Hint text |
| `"Move up"` | Tooltip (^ button) |
| `"Move down"` | Tooltip (v button) |
| `"No tasks yet"` | Empty state |

### Pomodoro section
| String | Context |
|--------|--------|
| `"Work: {}"` | Label (time) |
| `"Break: {}"` | Label (time) |
| `"Ready"` | Idle state |
| `"Cycles: {}"` | Label (count) |
| `"Stop"` | Button |
| `"Start Work (25m)"` | Button |
| `"Start Break (5m)"` | Button |

### Quick Notes section
| String | Context |
|--------|--------|
| `"Name:"` | Label |
| `"Note:"` | Label |
| `"Ok"` | Button (confirm rename) |
| `"Confirm rename"` | Tooltip |
| `"X"` | Button (cancel rename) |
| `"Cancel rename"` | Tooltip |
| `"+"` | Button (new note) |
| `"New note"` | Tooltip |
| `"Rn"` | Button (rename note) |
| `"Rename note"` | Tooltip |
| `"Confirm?"` | Button (confirm delete) |
| `"Click to confirm deletion"` | Tooltip |
| `"Delete note"` | Tooltip (🗑 button) |
| `"Type your notes here..."` | Hint text |
| `"Dock"` | Button (dock into outline) |
| `"Dock into outline panel"` | Tooltip |

### Validation / errors (user-facing)
| String | Context |
|--------|--------|
| `"Note name cannot be empty"` | Validation (rename) |
| `"A note with that name already exists"` | Validation (rename) |

---

## 4. File watcher (v0.2.6.1) – `src/app/file_ops.rs`

Toasts for externally modified files (reload / unsaved warning). No keys in `en.yaml` for these.

| String | Context |
|--------|--------|
| `"Reloaded: {}"` | Toast (single file reloaded; filename) |
| `"{} files reloaded from disk"` | Toast (multiple files) |
| `"File changed externally (unsaved changes): {}"` | Toast (single file, has unsaved changes; filename) |
| `"{} files changed externally (have unsaved changes)"` | Toast (multiple files with unsaved changes) |

---

## 5. Navigation and toasts (`src/app/navigation.rs`, `src/app/keyboard.rs`)

All of these are passed to `show_toast()` and are hardcoded.

| String | Context |
|--------|--------|
| `"Undo ({} remaining)"` | Toast after undo |
| `"Nothing to undo"` | Toast when undo stack empty |
| `"Redo ({} remaining)"` | Toast after redo |
| `"Nothing to redo"` | Toast when redo stack empty |
| `"Outline panel shown"` | Toast |
| `"Outline panel hidden"` | Toast |
| `"Loaded workspace terminal layout"` | Toast |
| `"Terminal panel shown"` | Toast |
| `"Terminal panel hidden"` | Toast |
| `"Zen Mode enabled"` | Toast |
| `"Zen Mode disabled"` | Toast |
| `"Fullscreen mode (F10 or Esc to exit)"` | Toast |
| `"Exited fullscreen mode"` | Toast (navigation + keyboard) |
| `"Pipeline feature is disabled"` | Toast |
| `"Pipeline panel hidden in Zen Mode"` | Toast |
| `"Pipeline only available for JSON/YAML (current: {})"` | Toast (file type) |
| `"Pipeline panel opened"` | Toast |
| `"Pipeline panel closed"` | Toast |

---

## 6. Formatting / document actions (`src/app/formatting.rs`)

| String | Context |
|--------|--------|
| `"TOC only available for Markdown files"` | Toast |
| `"No document to format"` | Toast |
| `"Not a structured data file"` | Toast (format + validate) |
| `"Document formatted"` | Toast |
| `"Format failed: {}"` | Toast (error) |
| `"Parse error: {}"` | Toast |
| `"No document to validate"` | Toast |
| `"✔ Valid {} syntax"` | Toast (file type) |
| `"✗ {}"` | Toast (validation error) |

---

## 7. Find & Replace (`src/app/find_replace.rs`)

| String | Context |
|--------|--------|
| `"Replaced"` | Toast (single replace) |
| `"Replaced {} occurrence{}"` | Toast (replace all; pluralization) |

---

## 8. Export (`src/app/export.rs`)

| String | Context |
|--------|--------|
| `"No document to export"` | Toast |
| `"Exported to {}"` | Toast (path) |
| `"Export failed: {}"` | Toast |
| `"No document to copy"` | Toast |
| `"HTML copied to clipboard"` | Toast |
| `"Copy failed: {}"` | Toast |

---

## 9. File operations & workspace (`src/app/file_ops.rs`)

| String | Context |
|--------|--------|
| `"Opened {} files"` | Toast |
| `"Saved: {}"` | Toast (path) |
| `"Failed to save file:\n{}"` | Error modal |
| `"Opened workspace: {}"` | Toast (folder name) |
| `"Failed to open workspace:\n{}"` | Error modal |
| `"Workspace closed"` | Toast |
| `"Open a folder first (📁 button)"` | Toast |
| `"Open a folder first to use quick open"` | Toast |
| `"Open a folder first to use search in files"` | Toast |
| `"Failed to open file:\n{}"` | Error modal (multiple call sites) |
| `"File tree refreshed"` | Toast |
| `"Created: {}"` | Toast (name) |
| `"Failed to create file:\n{}"` | Error modal |
| `"Failed to create folder:\n{}"` | Error modal |
| `"Renamed to: {}"` | Toast |
| `"Failed to rename:\n{}"` | Error modal |
| `"Deleted: {}"` | Toast |
| `"Failed to delete:\n{}"` | Error modal |
| `"Failed to add image:\n{}"` | Error modal |
| `"Failed to open explorer:\n{}"` | Error modal |
| `"Failed to write file:\n{}"` | Error modal |

---

## 10. Dialogs & status bar (`src/app/dialogs.rs`, `src/app/status_bar.rs`)

| String | Context |
|--------|--------|
| `"Settings reset to defaults"` | Toast (dialogs.rs) |
| `"Opened in background: {}"` | Toast (status_bar; filename) |
| `"Failed to open file:\n{}"` | Error (status_bar) |
| `"Opened workspace: {}"` | Toast (status_bar; duplicate of file_ops) |
| `"Failed to open workspace:\n{}"` | Error (status_bar) |
| `"File Encoding"` | Status bar encoding popup heading |
| `"Failed to change encoding: {}"` | Toast |
| `"Encoding changed to {}"` | Toast |
| `"📄 Recent Files"` | Recent files popup section (status_bar) |
| `"📁 Recent Folders"` | Recent folders popup section (status_bar) |
| `"{}\n\nClick: Open\nShift+Click: Open in background"` | Tooltip (recent item; path) |

---

## 11. Outline panel (`src/ui/outline_panel.rs`)

| String | Context |
|--------|--------|
| `"Detach to floating window"` | Tooltip (detach productivity) |

---

## 12. Markdown / widgets (`src/markdown/widgets.rs`)

Tooltips that may or may not already have keys under `widgets.*`:

| String | Context |
|--------|--------|
| `"Decrease level"` | List item tooltip |
| `"Increase level"` | List item tooltip |
| `"Remove item"` | List item (×) tooltip |
| `"Delete row"` | Table row tooltip |
| `"Add a new row"` | Table tooltip |
| `"Add a new column"` | Table tooltip |

*Note:* `en.yaml` has `widgets.table.add_row` / `add_column` (e.g. "+ Row", "+ Column") but not these exact tooltip strings.

---

## 13. Echo / AI placeholder (`src/app/mod.rs`)

| String | Context |
|--------|--------|
| `"Echo Demo (AI Panel Placeholder)"` | Window title |
| `"This demonstrates async workers. Type a message:"` | Label |
| `"Responses (100ms delay):"` | Label |
| `"This panel will be replaced with AI chat in Phase 8."` | Label |
| `"Demonstrates: lazy worker spawn, mpsc communication, non-blocking UI."` | Label |

---

## 14. File dialogs (`src/files/dialogs.rs`, `src/app/export.rs`)

Filter names and dialog titles (user-visible in file picker):

| String | Context |
|--------|--------|
| `"Open Workspace Folder"` | Dialog title (dialogs.rs) |
| `"Supported Files"` | Filter (dialogs) |
| `"Markdown Files"` | Filter |
| `"Text Files"` | Filter |
| `"JSON Files"` | Filter |
| `"YAML Files"` | Filter |
| `"TOML Files"` | Filter |
| `"CSV/TSV Files"` | Filter |
| `"All Files"` | Filter |
| `"HTML Files"` | Filter (export; html, htm) |

*Terminal-specific filters already listed in §1.*

---

## Summary by area

| Area | Approx. count | Notes |
|------|----------------|------|
| Terminal panel | 50+ | All UI, errors, toasts, filters |
| Settings (Terminal) | ~10 | Section + checkboxes + tooltips |
| Productivity panel | 35+ | Tasks, Pomodoro, Quick Notes, validation |
| File watcher | 4 | Reload / unsaved toasts |
| Navigation toasts | 17 | Undo/redo, outline, terminal, zen, fullscreen, pipeline |
| Formatting | 9 | TOC, format, validate toasts |
| Find & Replace | 2 | Replace toasts |
| Export | 6 | Export/copy toasts |
| File ops & workspace | 20+ | Open/save/workspace/refresh/create/rename/delete/errors |
| Status bar & dialogs | ~10 | Encoding, recent files, reset settings |
| Outline panel | 1 | Detach tooltip |
| Markdown widgets | 6 | List/table tooltips |
| Echo/AI placeholder | 5 | Demo window |
| File dialog filters | 10+ | Open/save/export filter names |

**Total:** Roughly **180+** distinct user-facing strings not currently using translation keys. The largest gaps are **Terminal**, **Productivity Hub**, **navigation/formatting/export/file toasts**, and **Settings Terminal section**.

---

## Suggested key namespaces for `locales/en.yaml`

- `terminal.*` – all terminal panel and terminal settings strings.
- `productivity.*` or `tasks.*`, `pomodoro.*`, `notes.*` – Productivity Hub.
- `notification.*` – extend for file watcher, undo/redo, outline, terminal, zen, fullscreen, pipeline, format, find, export, file ops toasts (and any other notifications).
- `dialog.*` or `error.*` – error modals and confirmations (e.g. close terminal, failed to save).
- `file_dialog.*` or `dialog.file.*` – file picker titles and filter names.
- Keep existing `settings.*` and add `settings.terminal.*` for the Terminal section.

After adding keys to `en.yaml`, replace each string in code with `t!("key")` or `t!("key", param = value)` for parameterized messages, then run extraction/sync for Weblate as per your workflow.
