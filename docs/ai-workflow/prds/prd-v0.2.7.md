# PRD: Ferrite v0.2.7 - Performance, Features & Polish

## Overview

v0.2.7 focuses on features deferred from v0.2.6 to allow focus on the custom text editor, plus checking for updates and quality improvements.

**Theme:** Performance, linking, editing modes, updates, and polish.

**Scope:** Bug fixes, markdown linking (wikilinks, backlinks), optional Vim mode, manual check-for-updates, large file performance, flowchart refactoring, executable code blocks (opt-in), and GitHub-style callouts.

---

## P0 - Critical Bug Fixes

### #76 - CJK Rendering After Restart with Explicit Preference

**Problem:** When "Which CJK font to prioritize" is set to a non-Auto value and the app restarts, Chinese can render as tofu in restored tabs. Root cause: we only lazy-load CJK for the active tab and don't preload the user's preferred font at startup.

**Fix:**
- Preload the single preferred CJK font at startup when preference is explicit
- Same approach as Auto + system locale (preload at app init)
- Ensures restored documents render correctly regardless of which tab is active

**Location:** CJK font loading logic (likely `src/editor/ferrite/` or theme/font module)

**Test:**
1. Set CJK font preference to explicit value (e.g., Noto Sans SC)
2. Restart app with restored session containing Chinese text
3. Switch between tabs → Chinese should render correctly in all tabs

---

### General Bug Fixes

**Scope:** Additional issues reported post-v0.2.6.1 release. To be populated from GitHub issues as they emerge.

---

## P1 - Markdown Linking

### #1 - Wikilinks Support

**Feature:** `[[wikilinks]]` syntax — standard Obsidian/wiki-style internal links.

**Requirements:**
- Parse `[[target]]` and `[[target|display text]]` syntax
- Resolve target relative to current file or workspace root
- **Resolution tie-breaker** (ambiguous `[[note]]` when ProjectA/note.md and ProjectB/note.md exist): Same-folder-first, then shortest path; if still ambiguous, prompt user to choose
- **Spaces in filenames:** Support `[[My Note]]` (Obsidian convention); internally resolve to file path — no `%20` required in source
- Click-to-navigate in rendered/split view
- Support in outline/backlinks

**Implementation notes:**
- Extend markdown parser (pulldown-cmark or custom in `src/markdown/`)
- Store link targets for resolution; handle broken links gracefully
- Use existing `Tab.content` / FerriteEditor buffer for resolution

**Test:**
1. Create `note-a.md` with `[[note-b]]` and `[[note-b|Link to B]]`
2. Rendered view: click link → opens note-b.md
3. Display text variant shows "Link to B" as link label
4. `[[My Document]]` with spaces → resolves to `My Document.md`

---

### Backlinks Panel

**Feature:** Show documents that link to the current file.

**Requirements:**
- New panel (or extend outline) listing files containing `[[current_file]]` or `[text](current_file.md)`
- Update when switching tabs or on file change
- Click entry → navigate to that file (and ideally scroll to link)

**Implementation notes:**
- **Small workspaces (≤50 files):** Scan on tab switch or debounced
- **Large workspaces (>50 files):** Build lightweight in-memory graph on workspace load: `HashMap<filename, [referencing_files]>`; update incrementally on save. Avoid O(N) scan on every tab switch.
- UI: `ui/` — could extend `outline_panel.rs` or add `backlinks_panel.rs`

**Test:**
1. File A links to B, C links to B
2. Open B → backlinks panel shows A and C
3. Click A → navigates to A

---

## P2 - Editing Modes

### Vim Mode (Optional)

**Feature:** Optional Vim-style modal editing: Normal / Insert / Visual modes.

**Requirements:**
- Setting to enable/disable (default: disabled)
- Normal mode: hjkl movement, dd (delete line), yy (yank), p (paste), / (search), etc.
- Insert mode: type normally, Esc to Normal
- Visual mode: v for char-wise, V for line-wise selection
- **Status bar indicator:** Show `[NORMAL]` or `[INSERT]` (or block vs line cursor) so users know which mode is active — avoids "keyboard broken" confusion
- Essential subset first; expand in later releases

**Implementation notes:**
- Integrate with FerriteEditor in `src/editor/ferrite/`
- Modal state machine; key handling differs by mode
- **Key conflict handling:** Vim keys (e.g. Ctrl+C) conflict with standard shortcuts. In egui, handle focus and key consumption priority so Vim mode consumes keys in editor when active; standard shortcuts take precedence when not in Vim or when modal doesn't use the key
- Consider `editor/vim.rs` module or similar

**Test:**
1. Enable Vim mode in settings
2. Open file → Normal mode; status bar shows `[NORMAL]`; hjkl moves cursor
3. i → Insert mode; status bar shows `[INSERT]`; type; Esc → Normal
4. v → Visual; select; y → yank; p → paste

---

## P3 - Check for Updates

### Manual Check for Updates

**Feature:** Settings panel button that checks GitHub and prompts to install if an update is found.

**Requirements:**
- "Check for Updates" button in Settings (e.g., About/General section)
- Fetches latest release from GitHub API (e.g., `https://api.github.com/repos/OlaProeis/Ferrite/releases/latest`)
- Compare version with current; if newer, show dialog with download link
- **Manual trigger only** — no automatic background checking (offline-first)

**Implementation notes:**
- Add to `src/ui/settings.rs`
- Use `reqwest` or similar for HTTP (blocking or async via workers)
- Version comparison: semver or simple string compare
- Handle network errors gracefully (toast)

**Test:**
1. Click "Check for Updates" → if up to date, show "You're on the latest version"
2. If update available, show version + download link
3. Offline / API error → user-friendly message

---

## P4 - Large File Performance

### Large File Detection & Warning

**Feature:** Auto-detect files > 10MB on open; show warning toast.

**Requirements:**
- On file open: if size > 10MB, show toast: "Large file (X MB). Performance may be affected."
- Non-blocking; user can proceed
- Configurable threshold in settings (optional, v0.2.8+)

**Implementation notes:**
- Check file size in open flow (`app.rs` or file loading)
- Use `std::fs::metadata().len()` before loading content

**Test:** Open 15MB file → toast appears; file still opens.

---

### Lazy CSV Row Parsing

**Feature:** Parse CSV rows on-demand using byte-offset index for massive CSVs.

**Requirements:**
- For large CSVs, build byte-offset index of row boundaries
- Parse rows only when visible (virtual scrolling)
- Avoid loading entire CSV into memory as parsed grid

**Implementation notes:**
- CSV viewer in `src/` (locate existing CSV handling)
- Index: `Vec<u64>` of byte offsets per row
- Parse row N: `buffer[offsets[n]..offsets[n+1]]` with CSV parsing

**Test:** Open 100MB CSV → memory stays low; scrolling is smooth.

---

## P5 - Refactoring & Quality

### Flowchart Refactoring

**Scope:** Modularize `src/markdown/mermaid/flowchart.rs` (~3,200+ lines).

**Requirements:**
- Split into logical modules (e.g., nodes, edges, layout, rendering)
- Preserve behavior; no functional changes
- Improve maintainability for future Mermaid enhancements
- **Rendering cache:** egui is immediate-mode — recomputing layout every frame is expensive. Cache the rendered flowchart as a texture; only recompute when diagram content changes. Avoid full layout+draw on each frame.

**Proposed structure (example):**
- `flowchart/nodes.rs` — node shapes, labels
- `flowchart/edges.rs` — edge routing, arrow types
- `flowchart/layout.rs` — positioning, rank assignment
- `flowchart/render.rs` — egui drawing
- `flowchart/mod.rs` — public API, glue

**Test:** All existing flowchart tests pass; manual Mermaid flowchart rendering unchanged.

---

### Window Controls & Icon Polish

**Feature:** Native-feel window controls for macOS; further icon polish.

**Requirements:**
- macOS: traffic-light style window controls (close, minimize, maximize) in window frame
- Consistent icon quality across light/dark themes
- Follow platform HIG where applicable

**Implementation notes:**
- `src/ui/window.rs` — window decoration
- `eframe`/`egui` window options for native vs custom traffic lights

**Test:** macOS build shows native-style window controls; icons look correct in both themes.

---

## P6 - Executable Code Blocks

**Security note:** Code execution is **opt-in and disabled by default**.

### Run Button on Code Blocks

**Feature:** Add `▶ Run` button to fenced code blocks.

**Requirements:**
- Render small "Run" button on eligible code blocks (shell, bash, python)
- Click → execute via `std::process::Command`
- Output displayed inline or in collapsible section below block

### Execution Support

- **Shell / Bash:** Execute via system shell. **Windows:** `sh`/`bash` fail unless WSL or Git Bash in PATH — default to `powershell` or `cmd`, or detect if `bash.exe` exists. **Unix:** use `sh` or `bash`.
- **Python:** Detect `python` / `python3` and run with system interpreter
- **CWD (Current Working Directory):** Run script in the **file's directory** (user expectation for relative paths). Fallback to app root only if file has no path (e.g. unsaved).
- **Timeout:** Kill long-running scripts after configurable timeout (default: 30s)
- **Security warning:** First-run dialog explaining execution risks; user must acknowledge before first run

**Implementation notes:**
- New module e.g. `src/code_execution/` or under `src/markdown/`
- Settings: enable/disable, timeout, interpreter paths
- Use `std::process::Command`; timeout via `std::thread::spawn` + kill
- Store "has seen warning" in settings or local state

**Test:**
1. Default: Run button hidden or disabled until opt-in
2. Enable → Run button appears on ```bash and ```python blocks
3. Run simple script → output shown; CWD is file's directory (relative paths work)
4. Run infinite loop → killed after 30s
5. First run → security dialog shown once
6. Windows: script runs via powershell/cmd (or detected bash) without WSL

---

## P7 - Content Blocks / Callouts

### GitHub-Style Callouts

**Feature:** Support GitHub-style admonition blocks.

**Syntax:**
- `> [!NOTE]` — Note
- `> [!TIP]` — Tip
- `> [!WARNING]` — Warning
- `> [!CAUTION]` — Caution
- `> [!IMPORTANT]` — Important

**Requirements:**
- Parse `> [!TYPE]` and optional `> [!TYPE] Custom Title`
- Styled rendering: color-coded blocks, icons
- Collapsible: `> [!NOTE]-` for collapsed-by-default

**Implementation notes:**
- Extend blockquote parsing in `src/markdown/`
- Map type to color + icon; render in `markdown/widgets.rs` or similar
- Collapsed state: store in AST or view state; toggle on click

**Test:**
1. `> [!NOTE]\n> Content` → blue note block with icon
2. `> [!WARNING] Custom Title\n> Content` → orange block, "Custom Title"
3. `> [!NOTE]-` → collapsed by default; click to expand

---

## Deferred (Documented, Not in v0.2.7)

- **Bidirectional scroll sync** — Editor-Preview scroll sync in Split view (requires viewport-based line tracking)
- **IME candidate box positioning** (#15) — Chinese/Japanese IME candidate window offset
- **Click-to-edit cursor drift** — Cursor offset on mixed-format lines in rendered view

---

## Logical Dependency Chain

Suggested order for task breakdown:

1. **Bug fixes (P0)** — Unblock users immediately
2. **Check for Updates (P3)** — Small, self-contained, high visibility
3. **Large file detection (P4)** — Quick win, low risk
4. **Callouts (P7)** — Pure parser/rendering change; lower risk, warms up on parser codebase before linking logic. Do before Wikilinks.
5. **Wikilinks (P1)** — Foundation for backlinks; parser + interaction
6. **Backlinks panel (P1)** — Depends on wikilinks
7. **Flowchart refactoring (P5)** — Improves maintainability
8. **Window controls (P5)** — Platform polish
9. **Lazy CSV (P4)** — Larger refactor
10. **Vim mode (P2)** — Larger feature, editor integration
11. **Executable code blocks (P6)** — New subsystem, security-sensitive

---

## Technical Architecture Notes

| Component        | Location                      | Notes                                      |
|-----------------|-------------------------------|--------------------------------------------|
| Markdown parser | `src/markdown/parser.rs`      | Extend for wikilinks, callouts             |
| Rendering       | `src/markdown/widgets.rs`, `editor.rs` | Callouts, run button, link handling |
| Editor          | `src/editor/ferrite/`        | Vim mode, large file behavior              |
| Settings        | `src/config/settings.rs`     | New options: Vim, code execution, updates  |
| UI panels       | `src/ui/`                    | Backlinks, settings button                 |
| Mermaid         | `src/markdown/mermaid/`      | Flowchart refactor                         |

---

## Risks and Mitigations

| Risk                    | Mitigation                                              |
|-------------------------|---------------------------------------------------------|
| Code execution security | Opt-in, first-run warning, timeout, no network/filesystem by default |
| Vim mode scope creep    | Define minimal MVP (hjkl, i, Esc, dd, yy, p, v, /)      |
| Flowchart regressions   | Preserve all tests; manual Mermaid checklist            |
| Wikilinks ambiguity     | Tie-breaker rule (same-folder-first, shortest path); prompt if ambiguous |
| Vim key conflicts       | Vim consumes keys when active; egui focus/priority handling              |
| Code exec on Windows    | Default to powershell/cmd; detect bash if present                        |

---

## Testing Checklist (Pre-Release)

- [ ] P0: CJK renders after restart with explicit preference
- [ ] P1: Wikilinks parse, render, navigate; backlinks panel works
- [ ] P2: Vim mode toggles, basic commands work
- [ ] P3: Check for Updates works (current + outdated cases)
- [ ] P4: Large file toast; lazy CSV for big files
- [ ] P5: Flowchart tests pass; window controls on macOS
- [ ] P6: Code execution opt-in, timeout, security dialog
- [ ] P7: All callout types render; collapsible works
- [ ] No regressions in editor, terminal, productivity hub
- [ ] Cross-platform smoke test (Windows, macOS, Linux if available)

---

## Release Notes Draft

### v0.2.7 - Performance, Features & Polish

**Bug Fixes:**
- Fixed CJK font rendering in restored tabs when using explicit font preference (#76)

**New Features:**
- Wikilinks (`[[link]]`, `[[link|text]]`) with click-to-navigate
- Backlinks panel showing documents that link to the current file
- Optional Vim mode (Normal/Insert/Visual)
- Check for Updates button in Settings (manual only)
- Large file detection (>10MB) with warning toast
- Lazy CSV parsing for massive spreadsheets
- GitHub-style callouts (`> [!NOTE]`, `> [!TIP]`, etc.) with optional collapse
- Executable code blocks (opt-in): Run button for shell/Python with timeout
- Native window controls on macOS

**Improvements:**
- Flowchart module refactored for maintainability
- Icon and window polish

---

**Document Version:** 1.1  
**Created:** February 7, 2026  
**Updated:** February 7, 2026 — Incorporated external review (resolution rules, Vim status bar, Windows exec, CWD, flowchart cache, dependency order)  
**Status:** Draft — Ready for Review  
**Next Step:** Review with team; then run `task-master parse-prd` to generate tasks.
