# Ferrite Roadmap

## Next Up (Immediate Focus)

### v0.2.7 (Planned) - Performance, Features & Polish
**Focus:** Features moved from v0.2.6 to allow focus on the text editor, plus checking for updates.

#### Bug Fixes & UX
- [x] **Latin-only names in Language and CJK preference selectors** - Language selector (Settings → Appearance) and CJK Regional Preference (Settings → Editor) used native script (e.g. 简体中文, 日本語, 한국어), which rendered as squares when CJK fonts were not yet loaded (lazy loading). Both dropdowns now use Latin-only display names (e.g. "Chinese (Simplified)", "Japanese", "Korean (Hangul)") so they are always legible without preloading CJK fonts.
- [ ] **View mode bar on unsupported file types** - When default view is Split and user opens a file that doesn't support split (e.g. .rs), the view mode bar is hidden so mode can only be changed via hotkeys. Always show the view mode bar: for non–split-supported files use the two-mode segment (Raw | Rendered) so users can switch without hotkeys.
- [x] **Open file in current window as tab** - Single-instance protocol: double-clicking files (file tree or OS/Explorer) opens them as tabs in the existing Ferrite window instead of spawning a new process. Lock file + TCP IPC for cross-platform support.
- [x] **Images not displaying in rendered mode** - Markdown image syntax `![](path)` now shows images in rendered/split view; path resolution (relative to document dir + workspace root), image loading/caching via `image` crate (PNG, JPEG, GIF, WebP), scaled rendering with aspect ratio, graceful placeholders for missing/unsupported files.
- [x] **CJK rendering after restart with explicit preference** ([#76](https://github.com/OlaProeis/Ferrite/issues/76)) - When "Which CJK font to prioritize" is set to a non-Auto value and the app restarts, Chinese can render as tofu in restored tabs because we only lazy-load CJK for the active tab and don't preload the user's preferred font at startup. Fix: preload the single preferred CJK font at startup when preference is explicit (same approach as Auto + system locale), so restored documents render correctly regardless of which tab is active.
- [x] **Syntax highlighting per-frame re-parsing** - `highlight_line()` was called on every frame for every visible line *before* checking the galley cache, causing severe lag on files with long lines (e.g. dense markdown with inline formatting). Fixed by checking the cache first and only running syntect regex parsing on cache misses.
- [x] **Scrollbar position incorrect with word wrap** - Scrollbar thumb position was calculated using uniform `first_visible_line * line_height`, ignoring actual wrapped line heights. Fixed to use cumulative y-offsets from the height cache, so the scrollbar accurately reflects scroll position and reaches the bottom.
- [x] **Scrollbar drag used wrong reverse mapping** - Dragging the scrollbar converted pixel position to a line number using uniform division, causing inaccurate jumps with word wrap. Fixed to use `y_offset_to_line()` binary search with sub-line precision.
- [x] **Scrollbar jumping as new wrap info discovered** - `rebuild_height_cache` ran every frame (O(N)) and `total_content_height` changed abruptly as previously-unseen wrapped lines were measured during scrolling. Fixed with a dirty flag (only rebuild when wrap info changes) and smoothed scrollbar height that lerps toward the actual value.
- [x] **Crash on large selection delete with word wrap** - Selecting a large block of text top-down and pressing Backspace caused an instant crash (`capacity overflow` panic). Root cause: after deletion, `first_visible_line` remained past the now-shorter document due to stale `wrap_info`/`cumulative_heights`, causing `get_visible_line_range()` to return `start > end` and `Vec::with_capacity(end - start)` to underflow. Fixed with 4 layers: (1) `saturating_sub` on the Vec allocation, (2) hard-clamp `first_visible_line` to `total_lines-1` in `clamp_scroll_position`, (3) clamp `cursor_to_char_pos` result to `buffer.len()`, (4) new `truncate_wrap_info()` to trim stale entries instead of full-clearing (avoids flickering).
- [ ] **Wrapped line scroll stuttering** - Scrolling through documents with many word-wrapped lines still shows micro-stuttering. Likely related to per-line galley layout cost or height cache granularity. Needs further investigation.
- [ ] **General Bug Fixes** - Addressing additional issues reported post-v0.2.6.1 release.

#### Markdown Linking
- [x] **Wikilinks support** ([#1](https://github.com/OlaProeis/Ferrite/issues/1)) - `[[target]]` and `[[target|display]]` syntax with relative path resolution, spaces in filenames, click-to-navigate, ambiguity handling.
- [x] **Backlinks panel** - Panel showing files linking to current file; graph-based indexing for large workspaces; click-to-navigate.

#### Editing Modes
- [ ] **Vim Mode** - Optional Vim-style modal editing (Normal / Insert / Visual modes).

#### Check for Updates
- [x] **Check for Updates button** - Settings → About section with button that checks GitHub Releases API and shows result inline (up-to-date, update available with link, or error). Uses `ureq` (lightweight blocking HTTP) on a background thread with `mpsc` channel for non-blocking UI.
- [x] **Manual Trigger Only** - No automatic or background checking. Strictly user-initiated (offline-first philosophy).
- [x] **Security hardening** - Response URL validated against `https://github.com/OlaProeis/Ferrite/releases/` prefix; malformed URLs are replaced with a constructed safe URL. TLS via `rustls` (pure Rust, no OpenSSL).

#### Large File Performance
- [x] **Large file detection** - Auto-detect files > 10MB on open, show warning toast.
- [ ] **Lazy CSV row parsing** - Parse rows on-demand using byte offset index for massive CSVs.

#### Welcome View
- [x] **Welcome view on first run** - Welcome tab on first launch with configuration for theme, language, editor settings (word wrap, line numbers, minimap, bracket matching, syntax highlighting), max line width, CJK font preference, and auto-save. Only shown when no CLI paths and no session-restored tabs. Contributed by [@blizzard007dev](https://github.com/blizzard007dev) ([PR #80](https://github.com/OlaProeis/Ferrite/pull/80)).

#### Installer & Localization
- [ ] **Windows MSI: optional file associations** - During install, ask user whether to set Ferrite as default for .md, .txt, .json, .yaml, .toml (e.g. "Set as default for all" or per-extension); do not force associations without consent. WiX: `wix/main.wxs`.
- [x] **German and Japanese in Settings** - German (Deutsch) and Japanese (日本語) now available in Settings → Appearance → Language.

#### Refactoring & Quality
- [x] **Flowchart Refactoring** - Modularized 3600-line `flowchart.rs` into 12 focused modules: `flowchart/types.rs`, `parser.rs`, `layout/` (config, graph, subgraph, sugiyama), `render/` (colors, nodes, edges, subgraphs), `utils.rs`.
- [ ] **Window Controls** - Native-feel window controls for macOS; further icon polish.

#### Executable Code Blocks
- [ ] **Run button on code blocks** - Add `▶ Run` button to fenced code blocks.
- [ ] **Shell / Bash execution** - Execute shell snippets via `std::process::Command`.
- [ ] **Python support** - Detect `python` / `python3` and run with system interpreter.
- [ ] **Timeout handling** - Kill long-running scripts after configurable timeout (default: 30s).
- [ ] **Security warning** - First-run dialog explaining execution risks.  
  *Security note: Code execution is opt‑in and disabled by default.*

#### Content Blocks / Callouts
- [x] **GitHub-style callouts** - Support `> [!NOTE]`, `> [!TIP]`, `> [!WARNING]`, `> [!CAUTION]`, `> [!IMPORTANT]`.
- [x] **Custom titles** - `> [!NOTE] Custom Title`.
- [x] **Styled rendering** - Color-coded blocks with icons in rendered view.
- [x] **Collapsible callouts** - `> [!NOTE]-` syntax for collapsed-by-default blocks.

---

## Known Issues 

### FerriteEditor Limitations
With the v0.2.6 custom editor, most previous egui TextEdit limitations are resolved. Remaining issues:

- [ ] **IME candidate box positioning** ([#15](https://github.com/OlaProeis/Ferrite/issues/15)) - Chinese/Japanese IME candidate window may appear offset from cursor position.

### Deferred to v0.2.7
- [ ] **Bidirectional scroll sync** - Editor-Preview scroll synchronization in Split view. Requires deeper investigation into viewport-based line tracking.

### Rendered View Limitations
- [ ] **Click-to-edit cursor drift on mixed-format lines** - When clicking formatted text in rendered/split view, cursor may land 1-5 characters off on long lines with mixed formatting.

---

## Planned Features 

### v0.2.8 - UI & Accessibility

#### Traditional Menu Bar ([#59](https://github.com/OlaProeis/Ferrite/issues/59))
- [ ] **Alt-key menu access** - Traditional File/Edit/View menus toggled via Alt key (VS Code style).
- [ ] **Accessibility** - Full keyboard navigation for all menu items.

#### Additional Format Support

##### XML Tree Viewer
- [ ] **XML file support** - Open `.xml` files with syntax highlighting.
- [ ] **Tree view** - Reuse JSON/YAML tree viewer for hierarchical XML display.
- [ ] **Attribute display** - Show element attributes in tree nodes.

##### Configuration Files
- [ ] **INI / CONF / CFG support** - Parse and display `.ini`, `.conf`, `.cfg` files.
- [ ] **Java properties files** - Support for `.properties` files.
- [ ] **ENV files** - `.env` file support with optional secret masking.

##### Log File Viewing
- [ ] **Log file detection** - Recognize `.log` files and common log formats.
- [ ] **Level highlighting** - Color-code `ERROR`, `WARN`, `INFO`, `DEBUG`.
- [ ] **Timestamp recognition** - Highlight ISO timestamps and common date formats.

---

### v0.3.0 - Mermaid Crate + Markdown Enhancements
**Focus:** Extracting the Mermaid renderer as a standalone crate and improving markdown rendering.

#### 1. Mermaid Crate Extraction
- [ ] **Standalone crate** - Backend-agnostic architecture with SVG, PNG, and egui outputs.
- [ ] **Public API** - `parse()`, `layout()`, `render()` pipeline.
- [ ] **SVG export** - Generate valid SVG files from diagrams.
- [ ] **PNG export** - Rasterize via `resvg`.
- [ ] **WASM compatibility** - SVG backend usable in browsers.

#### 2. Mermaid Diagram Improvements
- [ ] **Diagram insertion toolbar** ([#4](https://github.com/OlaProeis/Ferrite/issues/4)) - Toolbar button to insert Mermaid code blocks.
- [ ] **Syntax hints in Help** ([#4](https://github.com/OlaProeis/Ferrite/issues/4)) - Help panel with diagram syntax examples.
- [ ] **Git Graph rewrite** - Horizontal timeline, branch lanes, and merge visualization.
- [ ] **Flowchart enhancements** - More node shapes; `style` directive for per-node styling.
- [ ] **State diagram enhancements** - Fork/join pseudostates; shallow/deep history states.
- [ ] **Manual layout support**
  - Comment-based position hints: `%% @pos <node_id> <x> <y>`
  - Drag-to-reposition in rendered view with source auto-update
  - Export option to strip layout hints (“Export clean”)

#### 3. Markdown Enhancements
- [ ] **Wikilinks support** ([#1](https://github.com/OlaProeis/Ferrite/issues/1)) - `[[wikilinks]]` syntax with auto-completion.
- [ ] **Backlinks panel** ([#1](https://github.com/OlaProeis/Ferrite/issues/1)) - Show documents linking to current file.

#### 4. HTML Rendering (GitHub Parity)
**Phase 1 – Block Elements**
- [ ] `<div align="...">`, `<details><summary>`, `<br>`
**Phase 2 – Inline Elements**
- [ ] `<kbd>`, `<sup>`, `<sub>`, `<img width/height>`
**Phase 3 – Advanced**
- [ ] Nested HTML, HTML tables
*Note: Safe subset only (no scripts, styles, iframes).*

#### 5. Platform & Distribution
**Windows**
- [ ] Inno Setup installer
- [ ] File associations (`.md`, `.json`, `.yaml`, `.toml`)
- [ ] Context menu integration
- [ ] Optional add-to-PATH
**macOS**
- [ ] App signing & notarization

#### 6. Mermaid Authoring Improvements
- [ ] **Mermaid authoring hints** ([#4](https://github.com/OlaProeis/Ferrite/issues/4))  
  Inline hints and validation feedback when editing Mermaid diagrams to catch syntax errors and common mistakes early.

---

### v0.4.0 - Math Support & Document Formats
**Focus:** Native LaTeX math rendering and "page-less" Office document viewing.

#### Math Rendering Engine
- [ ] **LaTeX parser** - `$...$` inline and `$$...$$` display math.
- [ ] **Layout engine** - TeX-style box model (fractions, radicals, scripts).
- [ ] **Math fonts** - Embedded glyph subset for consistent rendering.
- [ ] **egui integration** - Render in preview and split views.

**Supported LaTeX (Target)**
- [ ] Fractions, subscripts/superscripts, Greek letters
- [ ] Operators (`\sum`, `\int`, `\prod`, `\lim`)
- [ ] Roots, delimiters, matrices
- [ ] Font styles (`\mathbf`, `\mathit`, `\mathrm`)

**WYSIWYG Features**
- [ ] Inline math preview while typing
- [ ] Click-to-edit rendered math
- [ ] Symbol palette

#### Office Document Support (Read‑Only)
**DOCX**
- [ ] Page-less rendering, text & tables, images
- [ ] Export DOCX → Markdown (lossy, with warnings)
**XLSX**
- [ ] Sheet selector, table rendering
- [ ] Basic number/date formatting
- [ ] Lazy loading for large sheets
**OpenDocument**
- [ ] ODT / ODS viewing with shared renderers

#### FerriteEditor Crate Extraction
- [ ] Standalone `ferrite-editor` crate (egui-first)
- [ ] Abstract providers (fonts, highlighting, folding)
- [ ] Delimiter matcher included
- [ ] Documentation and examples

---

## Future & Long-Term Vision 

### Core Improvements
- [ ] **Persistent undo history** - Disk-backed, diff-based history.
- [ ] **Memory-mapped I/O** ([#19](https://github.com/OlaProeis/Ferrite/issues/19)) - GB-scale files.
- [ ] **TODO list UX** - Smarter cursor behavior in task lists.
- [ ] **Spell checking** - Custom dictionaries.
- [ ] **Custom themes** - Import/export.
- [ ] **Virtual/ghost text** - AI suggestions.
- [ ] **Column/box selection** - Rectangular selection.

### Additional Document Formats (Candidates)
- [ ] **Jupyter Notebooks (.ipynb)** - Read-only viewing of cells and outputs.
- [ ] **EPUB** - Page-less e-book reading with TOC and position memory.
- [ ] **LaTeX source (.tex)** - Syntax highlighting, math preview, outline.
- [ ] **Alternative Markup Languages** ([#21](https://github.com/OlaProeis/Ferrite/issues/21))
  - reStructuredText, Org-mode, AsciiDoc, Zim-Wiki
  - Auto-detection by extension/content

### Plugin System
- [ ] Plugin API & extension points
- [ ] Scripting (Lua / WASM / Rhai)
- [ ] Community plugin distribution

### Headless Editor Library
- [ ] Framework-agnostic core extraction
- [ ] Abstract rendering backends (egui, wgpu, SVG)
- [ ] Advanced text layout integration (e.g., Parley)

**Note:** These are ideas under consideration.

---

## Recently Completed ✅

### v0.2.6.1 (Released Feb 2026) - Terminal, Productivity Hub & Refactoring
**First code-signed release.** Integrated Terminal Workspace and Productivity Hub contributed by [@wolverin0](https://github.com/wolverin0) ([PR #74](https://github.com/OlaProeis/Ferrite/pull/74)) — the first major community contribution. Major app.rs refactoring into ~15 modules. 8+ bug fixes.

### v0.2.6 (Released Jan 2026) - Custom Text Editor
**The critical rewrite.** Replaced the default egui editor with a custom-built virtual scrolling editor engine.

* **Memory Fixed:** 
* **Virtual Scrolling:** Only renders visible lines; massive performance boost.
* **Code Folding:** Visual collapse for code regions.
* **Editor Polish:** Word wrap, bracket matching, undo/redo, search highlights.

### Prior Releases
* **v0.2.5.x:** Syntax themes, Code signing prep, Multi-encoding support, Memory optimizations.
* **v0.2.5:** Mermaid modular refactor, CSV viewer, Semantic minimap.
* **v0.2.0:** Split view, Native Mermaid rendering.

> For detailed logs of all previous versions, see [CHANGELOG.md](CHANGELOG.md).
