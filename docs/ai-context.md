# Ferrite - AI Context

Rust (edition 2021) + egui 0.28 markdown editor. Immediate-mode GUI — no retained widget state, UI rebuilds each frame.

## Architecture

| Module | Purpose |
|--------|---------|
| `app/` | Main application (~15 modules: keyboard, file_ops, formatting, navigation, etc.) |
| `state.rs` | All application state (`AppState`, `Tab`, `TabKind`, `SpecialTabKind`, `FileType`) |
| `editor/ferrite/` | Custom rope-based editor (`ropey`) for large files (buffer, cursor, history, view, rendering) |
| `editor/widget.rs` | Editor widget wrapper, integrates FerriteEditor via egui memory |
| `markdown/editor.rs` | WYSIWYG rendered editing |
| `markdown/parser.rs` | Comrak markdown parsing, AST operations |
| `markdown/mermaid/` | Native mermaid rendering (11 diagram types); flowchart is modular (`flowchart/{types,parser,layout/,render/,utils}`) |
| `markdown/csv_viewer.rs` | CSV/TSV table viewer with lazy byte-offset row parsing |
| `markdown/tree_viewer.rs` | JSON/YAML/TOML hierarchical tree viewer |
| `terminal/` | Integrated terminal emulator (PTY via `portable-pty`, VTE ANSI parser, screen buffer, themes, split layouts) |
| `ui/` | UI panels (ribbon, settings, file_tree, outline, search, terminal_panel, productivity_panel, frontmatter_panel, welcome) |
| `config/` | Settings persistence, session/crash recovery, text expansion snippets |
| `theme/` | Light/dark theme management (ThemeManager, light.rs, dark.rs) |
| `export/` | HTML export with themed CSS, clipboard operations |
| `preview/` | Sync scrolling between Raw and Rendered views |
| `vcs/git.rs` | Git integration (status tracking, branch display, auto-refresh via `git2`) |
| `workspaces/` | Folder mode (file tree, file watcher, workspace settings, persistence) |
| `workers/` | Async worker infrastructure (feature-gated `async-workers`, tokio runtime) |
| `platform/` | Platform-specific code (macOS Apple Events) |
| `single_instance.rs` | Lock file + TCP IPC so double-clicking files opens tabs in existing window |
| `fonts.rs` | Font loading, lazy CJK, complex script lazy loading (11 families), family selection |
| `update.rs` | Update checker (GitHub Releases API) |

## FerriteEditor

Custom high-performance editor at `src/editor/ferrite/`. Uses `ropey` rope for O(log n) text operations.

**Key files:** `editor.rs` (main widget), `buffer.rs` (rope), `view.rs` (viewport), `history.rs` (undo/redo), `line_cache.rs` (galley LRU cache)

**Capabilities:** Virtual scrolling (renders only visible lines), multi-cursor (Ctrl+Click), code folding, bracket matching, IME/CJK input, syntax highlighting, find/replace.

**Memory:** ~1x file size in RAM (rope-based vs ~6x with egui TextEdit).

**Integration:** `EditorWidget` in `widget.rs` creates/retrieves `FerriteEditor` from egui memory, syncs with `Tab.content`.

**Deep docs:** `docs/technical/editor/architecture.md`

## Critical Patterns

```rust
// Always use saturating math for line indices
let idx = line_number.saturating_sub(1);

// Never unwrap in library code
if let Some(tab) = self.tabs.get_mut(self.active_tab) { ... }

// Prefer borrowing over clone
fn process(text: &str) -> Vec<&str> { text.lines().collect() }
```

## Common Gotchas

| Issue | Wrong | Right |
|-------|-------|-------|
| Byte vs char index | `text[start..end]` with char pos | Use `text.char_indices()` or byte offsets |
| Line indexing | Mixing 0/1-indexed | Explicit: `line.saturating_sub(1)` |
| CPU spin | Always `request_repaint()` | Use `request_repaint_after()` when idle |

## Conventions

- **Logging:** `log::info!`, `log::error!` (not println!)
- **i18n:** `t!("key.path")`, keys in `locales/en.yaml`
- **State:** `Tab` for per-tab, `AppState` for global
- **Errors:** User-facing via `show_toast()`, technical via `log::error!`
- **Large files (>1MB):** Hash-based `is_modified()`, reduced undo stack (10 vs 100), no `original_bytes`

## Where Things Live

| Want to... | Look in... |
|------------|------------|
| Add keyboard shortcut | `app/keyboard.rs` → `handle_keyboard_shortcuts()` |
| Add a file operation | `app/file_ops.rs` |
| Add text formatting command | `app/formatting.rs` |
| Add line operation (duplicate, move) | `app/line_ops.rs` |
| Add navigation feature | `app/navigation.rs` |
| Modify the title bar | `app/title_bar.rs` |
| Modify the status bar | `app/status_bar.rs` |
| Modify the central editor panel | `app/central_panel.rs` |
| Add a special tab (settings-like panel) | `state.rs` → `SpecialTabKind`, `app/central_panel.rs` → `render_special_tab_content()` |
| Add a setting | `config/settings.rs` → `Settings` struct |
| Add a translation string | `locales/en.yaml` + use `t!("key")` |
| Modify markdown rendering | `markdown/editor.rs` or `markdown/widgets.rs` |
| Modify markdown parsing | `markdown/parser.rs` |
| Add mermaid diagram type | `markdown/mermaid/` → new module |
| Modify flowchart layout | `markdown/mermaid/flowchart/layout/` |
| Modify flowchart rendering | `markdown/mermaid/flowchart/render/` |
| Add flowchart node shape | `flowchart/types.rs` (NodeShape) + `flowchart/render/nodes.rs` |
| Modify editor core behavior | `editor/ferrite/editor.rs` |
| Modify editor text buffer | `editor/ferrite/buffer.rs` |
| Change undo/redo behavior | `editor/ferrite/history.rs` |
| Modify code folding | `editor/folding.rs` |
| Modify minimap | `editor/minimap.rs` |
| Add/modify a UI panel | `ui/` → create or edit panel module |
| Modify terminal features | `terminal/` (pty, screen, widget, layout) |
| Modify terminal panel UI | `ui/terminal_panel.rs` |
| Modify productivity hub | `ui/productivity_panel.rs` |
| Change themes | `theme/light.rs` or `theme/dark.rs` |
| Add export format | `export/` → new module |
| Modify Git integration | `vcs/git.rs` |
| Modify workspace features | `workspaces/` |
| Add global app state | `state.rs` → `AppState` struct |
| Add per-tab state | `state.rs` → `Tab` struct |
| Modify platform-specific code | `platform/` (currently macOS only) |

## Performance Rules (FerriteEditor)

| Tier | When Allowed | Examples |
|------|--------------|----------|
| O(1) | Always | `line_count()`, `is_dirty()` |
| O(log N) | Always | `get_line(idx)`, index conversions |
| O(visible) | Per-frame | Syntax highlighting visible lines |
| O(N) | User-initiated ONLY | Find All, Save, Export |

**Never** call `buffer.to_string()` in per-frame code.

## Build & Test

```bash
cargo build          # Build debug
cargo run            # Run app
cargo clippy         # Lint
cargo test           # Run tests
```

## Current Focus

- Finishing v0.2.7 release (performance, polish, new features)
- Key areas: wikilinks/backlinks, vim mode, callouts, single-instance, welcome page, Unicode font loading (Phase 1 done)
- PortableApps.com packaging complete — submit to Beta Testing forum after v0.2.7 release
- v0.2.8 planned: LSP integration, HarfRust text shaping for complex scripts (Arabic, Bengali, Devanagari)
- v0.3.0 planned: RTL/BiDi text support, mermaid crate extraction, math rendering

## Recently Changed

- **2026-03-04**: Task 33 — Complex Script Font Preferences. Added Settings UI (Appearance → Additional Scripts) for pre-selecting fonts per script (Arabic, Bengali, Devanagari, Thai, Hebrew, Tamil, Georgian, Armenian, Ethiopic, Other Indic, Southeast Asian). `complex_script_font_preferences: BTreeMap<String, String>` in Settings; preferences passed through `load_complex_script_fonts_for_text`, `load_cjk_for_text`, `reload_fonts`. Docs: `docs/technical/config/complex-script-font-preferences.md`.
- **2026-03-04**: Fixed Open Folder in Flatpak (Task 39). Added `resolve_initial_dir()` with $HOME fallback in `src/files/dialogs.rs` so xdg-desktop-portal always has a navigable starting directory. Added `is_flatpak()` and Flatpak-aware error handling in `handle_open_workspace()`. No manifest changes. Docs: `docs/technical/platform/flatpak-file-dialog-portal.md`.
- **2026-03-04**: Refactored frontmatter editor from standalone side panel into a tab ("FM") inside the outline panel, matching BacklinksPanel pattern. `FrontmatterPanel::show_content(ui, is_dark)` renders inside outline tab area. Removed `frontmatter_panel_enabled`/`frontmatter_panel_width` settings. `Ctrl+Shift+M` now opens outline panel + switches to FM tab. Added `OutlinePanelTab::Frontmatter`, `OutlinePanelOutput.frontmatter_new_content`.
- **2026-03-04**: Implemented visual frontmatter editor (Task 32). `src/ui/frontmatter_panel.rs` with form-based YAML editing, type-aware widgets (text, checkbox, tag chips), bidirectional sync with raw editor. 7 unit tests.
- **2026-03-02**: Fixed critical bug when opening binary files (images, etc.) as text documents. The app would crash with "byte index is not a char boundary" panic in `stats.rs`. Two fixes applied: (1) Added `is_binary_content()` function in `state.rs` to detect binary files using null byte detection and non-printable character ratio heuristics. Binary files are now rejected with a user-friendly error message during file open. (2) Fixed `count_links()` and `count_images()` in `editor/stats.rs` to use safe string slicing with `get()` instead of direct byte indexing, preventing panics on invalid UTF-8 boundaries. Added 7 tests for binary detection.
- **2026-02-26**: Merged Nix/NixOS flake support ([PR #92](https://github.com/OlaProeis/Ferrite/pull/92) by @liuxiaopai-ai). Added `flake.nix`, `flake.lock`, CI workflow `.github/workflows/nix.yml`, docs in README + `docs/building.md`. Supports `nix run`/`nix build`/`nix develop` for 4 platforms.
- **2026-02-26**: PortableApps.com Format packaging. Added `FERRITE_DATA_DIR` env var to `config/persistence.rs` (priority 1 before `portable/` folder). Created full PAF directory structure in `portable/FerriteMDPortable/` with `appinfo.ini`, launcher config, help.html, NSIS installer script. Automated in `.github/workflows/release.yml` — installs NSIS via choco, generates icons from source, updates version from git tag, compiles `.paf.exe`, sends through SignPath, attaches to GitHub Release.
- **2026-02-25**: Fixed preview list item wrapping (#82): (1) Bug 1 — empty list item (`- `) caused paragraph above to render as heading. Comrak treats single-dash + whitespace as setext heading underline. Added `fix_false_setext_headings()` post-processing in `parser.rs`. (2) Bug 2 — list items used `TextEdit::singleline` which cannot wrap. Changed all 4 list-item TextEdit locations to `multiline` with custom `LayoutJob` layouter, `desired_rows(1)`, and newline stripping.
- **2026-02-23**: Fixed table rendering in `widgets.rs`: (1) Background alignment — replaced pre-painted backgrounds with Shape::Noop placeholder technique, painting actual-rect backgrounds after row renders. (2) Cell layout — changed from inherited `left_to_right` to explicit `top_down` so vertical padding works. (3) Removed `set_max_size` height constraint. (4) Added column resizing — draggable separators with proportional width persistence in `TableEditState.custom_col_widths`, double-click reset, min-width enforcement.
- **2026-02-23**: Implemented Unicode Complex Script Font Loading (Phase 1). Extended `fonts.rs` with lazy loading for 11 script families (Arabic, Bengali, Devanagari, Thai, Hebrew, Tamil, Georgian, Armenian, Ethiopic, other Indic, Southeast Asian) — 22 Unicode ranges, per-script AtomicBool flags, platform-specific system font candidates, `ComplexScriptDetection`/`ComplexScriptLoadSpec` structs, 17 tests. Triggers in `app/mod.rs` (per-frame + deferred file open) and `app/central_panel.rs` (IME). Font rebuild automatically includes already-loaded complex script fonts via `from_loaded_flags()`.
