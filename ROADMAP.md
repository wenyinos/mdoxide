# Ferrite Roadmap

## Next Up (Immediate Focus)

### v0.3.0 - Platform Stack Upgrade, Export, Code Execution, Media Embeds, Mermaid Crate & RTL/BiDi
**Primary focus:** **eframe / egui 0.31+** (Task 38) — large dependency migration with full cross-platform QA. Fixes Wayland keyboard input ([#106](https://github.com/OlaProeis/Ferrite/issues/106)), macOS Sonoma keyboard ([#111](https://github.com/OlaProeis/Ferrite/issues/111)), and Windows 11 borderless/DPI ([#112](https://github.com/OlaProeis/Ferrite/issues/112)). Also: PDF/HTML export, executable code blocks (deferred from v0.2.8), LSP integration (all phases, deferred from v0.2.8), embedded YouTube/video playback ([#119](https://github.com/OlaProeis/Ferrite/issues/119)), Mermaid crate extraction, and full RTL/BiDi (Phases 3–4). See [detailed plan](#v030---platform-stack-upgrade-export-code-execution-media-embeds-mermaid-crate--rtlbidi-1) below.

> **v0.2.9 (Apr 2026)** was a hotfix release for four critical v0.2.8 regressions — see [Recently Completed](#recently-completed-). The original v0.2.9 roadmap (platform upgrade, export, code execution, media embeds) has been rolled into v0.3.0, which was already planned as a large release. Split points for v0.3.1 will be decided once the v0.3.0 scope settles.

---

## Known Issues

### FerriteEditor Limitations
With the v0.2.6 custom editor, most previous egui TextEdit limitations are resolved. Remaining issues:

- [x] **IME candidate box positioning** ([#15](https://github.com/OlaProeis/Ferrite/issues/15), [#103](https://github.com/OlaProeis/Ferrite/issues/103)) - Fixed in v0.2.8. Applied `layer_transform_to_global()` to IME coordinates.
- [x] **IME backspace deleting text** ([#91](https://github.com/OlaProeis/Ferrite/issues/91)) - Fixed in v0.2.7. Backspace during IME composition no longer deletes editor text.
- [ ] **Wrapped line scroll stuttering** - Scrolling through documents with many word-wrapped lines still shows micro-stuttering. Likely related to per-line galley layout cost or height cache granularity. Needs further investigation.

### Deferred
- [ ] **Bidirectional scroll sync** - Editor-Preview scroll synchronization in Split view. Requires deeper investigation into viewport-based line tracking.
- [ ] **New file templates** - Optional frontmatter templates when creating new markdown files. Deferred from v0.2.7.

### Platform & Distribution
- [x] **macOS Gatekeeper blocking** ([#93](https://github.com/OlaProeis/Ferrite/issues/93)) - Fixed: CI now packages proper `.app` bundle via `cargo-bundle`.
- [ ] **Wayland keyboard input on Ubuntu 24.04** ([#106](https://github.com/OlaProeis/Ferrite/issues/106)) - No keyboard input on GNOME/Mutter Wayland. Root cause: winit 0.29.15 (via eframe 0.28) Wayland backend bug. Workaround: `WAYLAND_DISPLAY= ferrite`. **Fix scheduled: v0.2.9** — upgrade to eframe/egui 0.31+ (winit 0.31+ rewritten Wayland backend). Task 38.
- [ ] **macOS Sonoma keyboard input** ([#111](https://github.com/OlaProeis/Ferrite/issues/111)) - No keyboard input on macOS Sonoma 14.2. Likely same class of issue as #106 (winit 0.29 input pipeline). **Fix scheduled: v0.2.9** — eframe/egui 0.31+ upgrade (Task 38).
- [x] **Windows 11 borderless window offset** ([#112](https://github.com/OlaProeis/Ferrite/issues/112)) - Fixed in v0.2.8 with `.with_transparent(true)` DWM workaround. Full fix via eframe/egui 0.31+ expected in v0.2.9 (Tasks 38 & 46).

### Terminal
- [x] **CJK double-width character overlap in terminal** ([#110](https://github.com/OlaProeis/Ferrite/issues/110)) - Fixed in v0.2.8. Added `unicode-width` crate, 2-column cursor advancement, wide char rendering spanning 2 cells.

### Rendered View Limitations
- [x] **Slow rendering on large documents** ([#105](https://github.com/OlaProeis/Ferrite/issues/105)) - Fixed in v0.2.8. AST caching, viewport culling, block height cache, and lazy estimation bring large-file rendered view to usable performance.
- [ ] **Click-to-edit cursor drift on mixed-format lines** - When clicking formatted text in rendered/split view, cursor may land 1-5 characters off on long lines with mixed formatting.

---

## Planned Features

### v0.3.0 - Platform Stack Upgrade, Export, Code Execution, Media Embeds, Mermaid Crate & RTL/BiDi
**Primary focus:** **eframe / egui 0.31+** (Task 38) — large dependency migration with full cross-platform QA. Intended to address **Wayland keyboard input** ([#106](https://github.com/OlaProeis/Ferrite/issues/106)), **macOS Sonoma keyboard** ([#111](https://github.com/OlaProeis/Ferrite/issues/111)), and **Windows 11 borderless / DPI** ([#112](https://github.com/OlaProeis/Ferrite/issues/112)) where fixes depend on newer winit/egui. Workarounds (e.g. `WAYLAND_DISPLAY=` on Ubuntu Wayland) remain documented until this ships.

**Secondary focus:** First-class export from markdown to shareable files (PDF, self-contained HTML). Complements **PDF viewer tabs** (v0.2.8) — **writing → publish**, not only viewing external PDFs.

**Tertiary focus:** Executable code blocks (deferred from v0.2.8), full LSP integration (Phases 1–4, deferred from v0.2.8 due to memory/usability issues), embedded YouTube/video playback via native web views ([#119](https://github.com/OlaProeis/Ferrite/issues/119)), Mermaid crate extraction, markdown rendering improvements, Alt-key menu rework, and full RTL + BiDi script support (Phases 3–4 of the Unicode shaping roadmap).

*Scope note:* This is a large release. Items listed below may be split into a follow-up v0.3.1 if scope creep threatens ship-quality. Split points will be decided once the egui 0.31+ migration lands and stabilizes.

#### Platform & Dependency Upgrade (Task 38)
- [ ] **Bump eframe / egui** to 0.31+ (confirm compatible versions); `cargo update`; fix breaking API changes across `main.rs`, editor input, themes, terminal, markdown UI, etc.
- [ ] **Regression pass** — Windows, macOS, Linux X11, **Ubuntu 24.04 Wayland** (native Wayland, no XWayland override); IME, LSP, terminal input, HarfRust/shaped text.
- [ ] **Close or update** GitHub issues #106, #111, #112 once verified on the new stack.

#### PDF & Print
- [ ] **PDF export** - Export markdown to PDF with sensible defaults (margins, page breaks, code blocks, tables). Implementation TBD: e.g. HTML intermediate + system/headless print pipeline, or a Rust PDF stack for simpler paths; evaluate quality vs. bundle size.
- [ ] **Print preview** *(optional)* - Quick preview of paginated output before saving PDF where the pipeline supports it.

#### HTML Export
- [ ] **Themed / self-contained HTML** - Stronger parity between exported HTML and in-app appearance: embedded or linked CSS reflecting light/dark (or user-chosen) theme, improved Mermaid/code/table styling for static pages.
- [ ] **Export options** - User-tunable toggles where useful (e.g. include outline, strip comments, base path for assets).

#### Executable Code Blocks
- [ ] **Run button on code blocks** - Add `▶ Run` button to fenced code blocks.
- [ ] **Shell / Bash execution** - Execute shell snippets via `std::process::Command`.
- [ ] **Python support** - Detect `python` / `python3` and run with system interpreter.
- [ ] **Timeout handling** - Kill long-running scripts after configurable timeout (default: 30s).
- [ ] **Security warning** - First-run dialog explaining execution risks.
  *Security note: Code execution is opt-in and disabled by default.*

#### Embedded Media — YouTube / Video Embeds ([#119](https://github.com/OlaProeis/Ferrite/issues/119))
- [ ] **Custom syntax detection** — Detect YouTube/video URLs in markdown (e.g. `{{video URL}}` or bare YouTube URLs in their own paragraph) in `markdown/parser.rs`.
- [ ] **Embedded web view via `wry`** — Use Tauri's [`wry`](https://lib.rs/crates/wry) crate to spawn a platform-native WebView (WebView2 on Windows, WebKitGTK on Linux, WebKit on macOS) as a child window positioned over the egui rendered view.
- [ ] **Viewport tracking** — Sync the child WebView position/size with the egui rect each frame; hide when scrolled off-screen or tab is inactive.
- [ ] **Fallback: thumbnail + open-in-browser** — For platforms where `wry` child windows aren't viable, fetch YouTube thumbnail (`img.youtube.com`) and render as clickable image with play overlay; click opens system browser.
- [ ] **Extensible embed system** — Design the embed trait/interface to support future providers (Vimeo, etc.).

*Note: This is an exploratory feature. The `wry` child-window-over-egui approach has known challenges (z-ordering, scroll sync, platform quirks). The thumbnail fallback ensures the feature ships something usable regardless. Depends on egui 0.31+ upgrade (Task 38) for stable `RawWindowHandle` access.*

#### LSP Integration (Phases 1–4)
*Deferred from v0.2.8: Phase 1–2 implementation had high memory usage (rust-analyzer ~3.8 GB) and no diagnostics panel to surface warnings. Code remains in-tree behind the `lsp` feature flag; needs fixes before shipping.*

- [ ] **Phase 1 fixes: Infrastructure & lifecycle** — Fix unbounded channels (add backpressure), clear diagnostics on workspace switch, cap transport frame size, properly join reader threads on shutdown.
- [ ] **Phase 1 fix: Incremental document sync** — Switch from full-document `didChange` to `TextDocumentSyncKind::Incremental` to reduce memory churn.
- [ ] **Phase 2 fix: Diagnostics panel** — Dedicated problems panel with click-to-navigate (bare minimum for LSP to be useful). Fix UTF-16→char column conversion for squiggle accuracy.
- [ ] **Phase 2 fix: Memory** — Stop per-frame diagnostic cloning (`Arc<Vec<DiagnosticEntry>>`), bounded event channels, `DiagnosticMap` cleanup on workspace switch.
- [ ] **Phase 3: Hover & Go to Definition** — Hover documentation with configurable delay; Go to Definition (F12 or Ctrl+Click).
- [ ] **Phase 4: Autocomplete** — Completion popup on typing or Ctrl+Space, debounced (e.g. 150ms), navigable with arrow keys; request cancellation for stale completions.
- [ ] **Settings** — Per-language server path override; all processing local (no network calls).

*Note: **LaTeX math** rendering in preview and export is planned under **v0.4.0**; PDF/HTML export will pick up formulas once the math engine exists.*

#### Command Discoverability ([#59](https://github.com/OlaProeis/Ferrite/issues/59))
*Addressed in v0.2.8 with Command Palette.*

- [x] **Command Palette (Alt+Space)** - Searchable command launcher with fuzzy search across all actions. Recent commands, category grouping, shortcut hints. Configurable keybinding. Replaces the need for traditional text menus.
- [x] **Windows Alt+Space suppression** - Thread-level keyboard hook prevents OS system menu conflict.
- [ ] **Accessibility** - Full keyboard navigation for all menu items. *(Ongoing)*

#### Unicode & Complex Script Support (Phase 3 & 4: RTL, BiDi, WYSIWYG)
*Depends on: Phase 2 text shaping from v0.2.8*

**Phase 3: Right-to-Left Layout & Bidirectional Text**
- [ ] **RTL text layout in FerriteEditor** - Render Arabic, Hebrew, and other RTL scripts right-to-left within lines. Shaped glyph runs are placed from the right edge; line alignment respects detected paragraph direction.
- [ ] **Unicode BiDi algorithm** - Implement the Unicode Bidirectional Algorithm (UAX #9) via the `unicode-bidi` crate for mixed-direction text (e.g., English embedded in Arabic). Resolves embedding levels, reorders glyph runs per line, and handles directional isolates/overrides.
- [ ] **RTL cursor navigation** - Arrow keys move in visual order (left arrow moves left visually, regardless of text direction). Home/End respect paragraph direction. Selection handles disjoint byte ranges in BiDi text.
- [ ] **RTL selection rendering** - Selection highlighting for BiDi text may produce multiple visual rectangles per logical selection range. Click-to-position respects visual glyph boundaries.
- [ ] **RTL line wrapping** - Word wrap respects script direction. Break opportunities follow UAX #14 (Unicode Line Breaking Algorithm) for correct behavior with Arabic, Hebrew, Thai, and other scripts.

**Phase 4: WYSIWYG & UI Chrome**
- [ ] **Shaped text in WYSIWYG editor** - Integrate text shaping into the rendered markdown view (`markdown/editor.rs`). RichText labels use shaped runs for correct Arabic/Bengali rendering in headings, paragraphs, lists, and tables.
- [ ] **Shaped text in Mermaid diagrams** - Update `TextMeasurer` to use shaped advance widths so diagram node labels render complex scripts correctly.
- [ ] **UI label shaping** - If egui has native shaping by this point (via Parley or direct HarfRust integration), adopt it. Otherwise, provide a shaping wrapper for critical UI surfaces (file tree, outline panel, status bar) where non-Latin file/heading names appear.

*Note: Full RTL+BiDi is one of the hardest problems in text editing. This phase has high risk in cursor positioning, selection handling, and find/replace with mixed-direction text. Thorough testing with real Arabic, Hebrew, and Bengali content is essential.*

#### 1. Mermaid Crate Extraction
- [ ] **Standalone crate** - Backend-agnostic architecture with SVG, PNG, and egui outputs.
- [ ] **Public API** - `parse()`, `layout()`, `render()` pipeline.
- [ ] **SVG export** - Generate valid SVG files from diagrams.
- [ ] **PNG export** - Rasterize via `resvg`.
- [ ] **WASM compatibility** - SVG backend usable in browsers.

#### 2. Mermaid Diagram Improvements
- [ ] **Evaluate `mermaid-rs-renderer` (mmdr) parser integration** - The [mmdr crate](https://github.com/1jehuang/mermaid-rs-renderer) (first released Jan 2026, after our renderer shipped) supports 23 diagram types with comprehensive Mermaid syntax coverage in pure Rust. Evaluate borrowing or depending on mmdr's parser for broader syntax support while keeping our native egui rendering layer. mmdr outputs SVG (not egui primitives), so a full replacement is not viable — but the parser could fill gaps in our syntax coverage for diagram types we haven't implemented yet (Sankey, Kanban, Quadrant, XY Chart, C4, Block, Architecture, Requirement, ZenUML, Packet, Radar, Treemap). Assess: parser API stability, dependency weight (`default-features = false` drops CLI+PNG deps), AST compatibility with our layout/render pipeline.
- [ ] **Diagram insertion toolbar** ([#4](https://github.com/OlaProeis/Ferrite/issues/4)) - Toolbar button to insert Mermaid code blocks.
- [ ] **Syntax hints in Help** ([#4](https://github.com/OlaProeis/Ferrite/issues/4)) - Help panel with diagram syntax examples.
- [ ] **Git Graph rewrite** - Horizontal timeline, branch lanes, and merge visualization.
- [ ] **Flowchart enhancements** - More node shapes; `style` directive for per-node styling.
- [ ] **State diagram enhancements** - Fork/join pseudostates; shallow/deep history states.
- [ ] **Manual layout support**
  - Comment-based position hints: `%% @pos <node_id> <x> <y>`
  - Drag-to-reposition in rendered view with source auto-update
  - Export option to strip layout hints ("Export clean")

#### 3. Markdown Enhancements
- [x] **Wikilinks support** ([#1](https://github.com/OlaProeis/Ferrite/issues/1)) - `[[wikilinks]]` syntax with click-to-navigate. *(Completed in v0.2.7)*
- [x] **Backlinks panel** ([#1](https://github.com/OlaProeis/Ferrite/issues/1)) - Show documents linking to current file with graph-based indexing. *(Completed in v0.2.7)*

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
- [x] File associations (`.md`, `.json`, `.yaml`, `.toml`) — done via MSI installer (v0.2.7)
- [x] Context menu integration — done via MSI installer (v0.2.7)
- [x] Optional add-to-PATH — done via MSI installer (v0.2.7)
- [x] PortableApps.com listing — PAF packaging and CI automation done (v0.2.7); forum submission pending
**macOS**
- [ ] App signing & notarization

#### 6. Mermaid Authoring Improvements
- [ ] **Mermaid authoring hints** ([#4](https://github.com/OlaProeis/Ferrite/issues/4))
  Inline hints and validation feedback when editing Mermaid diagrams to catch syntax errors and common mistakes early.

#### 7. Additional Format Support
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
- [ ] **PDF viewing (read-only)** - Page-by-page PDF rendering via native library bindings (PDFium or MuPDF). Requires shipping platform-specific native libraries (~20MB per platform). Complex cross-compilation. Low priority — OS viewers handle this well.
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
- [ ] Advanced text layout integration (HarfRust/skrifa, with Parley as future option)

**Note:** These are ideas under consideration.

---

## Recently Completed ✅

### v0.2.9 (Apr 2026) - Hotfix Release
Hotfix for four critical v0.2.8 regressions. No new features.
- **Crash in Split / Rendered view on empty documents** ([#127](https://github.com/OlaProeis/Ferrite/issues/127)) — viewport-culling bootstrap indexed `doc.root.children[0]` when `block_count == 0`. Fixed with a half-open render range.
- **No unsaved-changes indicator (`*`) and no save prompt on close, causing silent data loss** — raw-mode edits bypassed `content_version`, so `is_modified()` stayed cached at `false`. `content_version` bumps centralized in `record_edit_from_snapshot()` / `set_content()`.
- **Undo / redo reporting "Nothing to undo" after typing** — FerriteEditor's internal edits were never diffed into `tab.edit_history`, which is the stack Ctrl+Z / Ctrl+Y read. Fixed by snapshotting pre-edit content and recording ops per dirty frame.
- **Selection invisible in Light mode** ([#121](https://github.com/OlaProeis/Ferrite/issues/121)) — 40% alpha made the pale light-theme selection blend into the panel. Alpha reduction is now dark-mode-only.
- **Document side panel tab labels overlapping at default width** — raised default outline panel width from 200 → 300 px, minimum from 120 → 260 px; existing users auto-migrated by settings validator.

### v0.2.8 (Apr 2026) - Performance, Text Shaping, LSP Integration & Viewers
Command Palette (Alt+Space) with fuzzy search across all actions. LSP integration (Phases 1-2): inline diagnostics, server lifecycle, status bar, on-demand startup. HarfRust text shaping for Arabic, Bengali, Devanagari, and other complex scripts. Image viewer tabs (PNG/JPEG/GIF/WebP/BMP) and PDF viewer tabs (hayro, pure Rust). Major rendered view performance overhaul: AST caching, viewport culling, block height cache, lazy estimation. Per-frame O(N) elimination for large files. Background file loading for 5MB+ files. Strict line breaks (Obsidian model). Middle-click to close tabs. CSV/TreeViewer/central panel per-frame allocation fixes. Table cell rich text rendering with click-to-edit (bold, italic, strikethrough, code, nesting). 13 bug fixes including macOS .md file association (#102), Windows IME positioning (#103), custom font crash on Linux (#114), Linux Cinnamon dialog detection (#116), table inline formatting preservation and rendering (#117), terminal CJK rendering (#110), Windows 11 borderless offset (#112), and more.

### v0.2.7 (Mar 2026) - Performance, Features & Polish
Wikilinks & backlinks, Vim mode, welcome view, GitHub-style callouts, check for updates, Ctrl+Scroll Wheel zoom, keep text selected after formatting, lazy CSV parsing, large file detection, single-instance protocol, MSI installer overhaul with optional file associations, PortableApps.com Format packaging with automated CI build, Nix/NixOS flake support, German and Japanese localization, Unicode complex script font loading (Phase 1: 11 script families, 22 Unicode ranges), complex script font preferences UI (Settings → Additional Scripts), visual frontmatter editor, format toolbar moved to editor bottom, side panel toggle strip, Linux file dialog error handling with portal failure detection, flowchart modular refactoring, window control redesign, macOS .app bundle CI, task list checkbox rendering, word-wrap scroll correctness & performance fixes, preview list item wrapping fix, false setext heading fix, IME backspace fix (#91), binary file crash fix, rendered mode copy spacing fix, 20+ bug fixes including light mode visibility, scrollbar accuracy, and crash on large selection delete.

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
