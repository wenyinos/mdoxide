# Ferrite Roadmap

## Next Up (Immediate Focus)

### v0.2.8 (April 2026) - Performance, Text Shaping, LSP Integration & Viewers
**Focus:** Major performance overhaul (rendered view, editor, viewers), HarfRust text shaping for complex scripts, LSP integration (Phases 1 & 2), image/PDF viewer tabs, strict line breaks, background file loading, and 10+ bug fixes.

#### Rendered View Performance ([#105](https://github.com/OlaProeis/Ferrite/issues/105))
*Addresses slow/unusable rendered view on large documents (6k+ lines, 50k+ words). Two users reported this as a blocker in v0.2.7.*

- [x] **Markdown AST caching** - Cache parsed `MarkdownDocument` using blake3 content hash (same pattern as Mermaid diagram caching). Only call `parse_markdown()` when content actually changes instead of re-parsing the full document 60 times per second.
- [x] **Rendered view viewport culling** - Switch `ScrollArea::vertical().show()` to `.show_viewport()` and skip `render_node()` for blocks outside the visible area + 500px overscan buffer. Use `ui.allocate_space()` for off-screen content. Reduces per-frame widget construction from O(N blocks) to O(visible blocks).
- [x] **Block-level height cache** - LRU cache of measured heights per rendered block keyed on content hash. Avoids re-measuring off-screen blocks and enables accurate scrollbar positioning with viewport culling.
- [x] **Lazy block height estimation** - Heuristic heights for unmeasured blocks on first-frame render, render budget cap (max 20 blocks/frame), progressive refinement as user scrolls. Reduces initial lag to under 100ms for 10K+ block files.

*Note: This is not a full virtual document architecture (planned v0.4.0). It brings rendered view performance closer to parity with Obsidian for large files by eliminating the most wasteful per-frame work. The raw editor (FerriteEditor) already handles large files well via virtual scrolling.*

#### Editor Performance
- [x] **Uniform height mode for large files** - Auto-enable for 100K+ line files: O(1) line positioning via `line * line_height`, no O(N) `cumulative_heights` vector, force-disabled word wrap above threshold.
- [x] **Smarter LineCache invalidation and scaling** - Targeted `invalidate_range(start, end)` evicts only affected lines instead of full cache clear. Dynamic `max_entries(visible_lines)` scales cache size with viewport. 80%+ hit rate for unchanged regions after edits.
- [x] **Per-frame O(N) elimination** - 7 per-frame O(N) operations on `tab.content` cached via `content_version` counter: `TextStats::from_text()`, `tab.title()/is_modified()` (blake3 hash), save button `is_modified()`, `needs_cjk()/needs_complex_script_fonts()` char scans, `should_auto_save()` tab loop, frontmatter panel content clone, MarkdownEditor content clone. Target: <16ms/frame for 50MB files.
- [x] **Background thread file loading** - Move synchronous `std::fs::read` to background thread for files ≥5MB. Progress bar with spinner, MB loaded/total, cancellation on tab close. UI remains responsive at 60fps during load. `TabContent::Loading`/`Ready`/`Error` state tracking via `FileLoadMsg` channel.

#### Viewer Performance
- [x] **CSV raw view per-frame allocation fix** - `show_raw_view()` called `self.content.to_string()` every frame. Fixed with blake3 hash-guarded `raw_view_text` cache, rebuilt only on content change.
- [x] **TreeViewer parse and raw view caching** - Two blake3-guarded caches: raw view text cache (skip per-frame `to_string()`), parsed tree cache (skip per-frame `parse_structured_content()`). Supports JSON/YAML/TOML.
- [x] **Central panel undo content clone elimination** - Removed per-frame `tab.content.clone()` for undo recording across Raw, Rendered, Split, and TreeViewer modes. Raw mode leverages FerriteEditor's native EditHistory; other modes use blake3 hash-based change detection with conditional recording.

#### Bug Fixes
- [x] **macOS .md file association** ([#102](https://github.com/OlaProeis/Ferrite/issues/102)) - `UTImportedTypeDeclarations` in `info_plist_ext.xml` declares `net.daringfireball.markdown` (conforms to `public.plain-text`); `Cargo.toml` `osx_info_plist_exts` merges it into bundled `Info.plist`.
- [x] **Windows IME candidate box positioning** ([#103](https://github.com/OlaProeis/Ferrite/issues/103), [#15](https://github.com/OlaProeis/Ferrite/issues/15)) - Applied `layer_transform_to_global()` to `IMEOutput` coordinates so the OS receives correct screen coordinates for the candidate popup.
- [x] **Double-dash (`--`) setext headings and line collapsing** - Extended `fix_false_setext_headings()` to handle `--` (not just `-`). Preprocessing for `\n-- ` line breaks while preserving `---` horizontal rules and YAML frontmatter.
- [x] **No space between paragraphs in rendered view** ([#109](https://github.com/OlaProeis/Ferrite/issues/109)) - Added proper paragraph spacing after paragraph blocks. Cumulative heights updated for accurate scrollbar.
- [x] **Trailing spaces on plain text paragraphs** - Plain paragraph WYSIWYG editing dropped trailing spaces due to per-frame re-initialization from AST. Fixed with persistent egui edit buffer (keyed by `node.start_line`) using `extract_paragraph_content()` pattern.
- [x] **Search highlight positioning in markdown tables + z-order** - Fixed find/render coordinate mapping for table cells. Resolved z-order stacking where Jump to menu rendered above Find and Search panels.
- [x] **Search highlight misalignment after document edits** - Recompute match positions on buffer mutations. Version/hash-based auto-refresh for highlight state. Targeted cache invalidation for shaped galleys and heights.
- [x] **Terminal CJK double-width character rendering** ([#110](https://github.com/OlaProeis/Ferrite/issues/110)) - Added `unicode-width` crate. `put_char()` advances cursor by 2 for wide chars with continuation markers. Renderer draws wide chars spanning 2 cell widths. Selection snaps to wide char boundaries.
- [x] **Windows 11 borderless window UI offset** ([#112](https://github.com/OlaProeis/Ferrite/issues/112)) - Added `.with_transparent(true)` to `ViewportBuilder` as DWM compositing workaround for Intel HD 4600 GPU rendering offset in borderless mode. Full fix expected in v0.2.9 eframe upgrade.
- [x] **Scrollbar not resetting when switching documents** ([#113](https://github.com/OlaProeis/Ferrite/issues/113)) - Scoped central-panel editor/preview widget IDs with `tab.id` to prevent egui `ScrollArea` state leaking across tab switches.
- [ ] **macOS Sonoma keyboard input** ([#111](https://github.com/OlaProeis/Ferrite/issues/111)) - Same root cause as Wayland #106 (winit 0.29 input pipeline). **Target fix: v0.2.9** eframe/egui 0.31+ upgrade (Task 38).

#### Markdown Rendering Settings
- [x] **Strict line breaks setting** - "Strict line breaks" toggle in Settings → Editor (default: off). When enabled, single newlines render as hard `<br>` breaks. Wires up Comrak's `render.hardbreaks` option. Follows the Obsidian model.
- [x] **Strict line breaks in Settings UI** - Toggle in Settings → Editor section with live update to rendered view.
- [x] **Strict line breaks on Welcome page** - Toggle on Welcome page editor settings section (alongside word wrap, line numbers, minimap, etc.).

#### Unicode & Complex Script Support (Phase 2: Text Shaping Engine)
*Depends on: Phase 1 font loading from v0.2.7*

- [x] **HarfRust integration for FerriteEditor** - Integrated [HarfRust](https://github.com/harfbuzz/harfrust) (pure-Rust HarfBuzz port, v0.5.2+) into the FerriteEditor rendering pipeline. Converts Unicode codepoint sequences into correctly positioned, contextually-formed glyphs for Arabic (contextual forms), Bengali (conjuncts), Devanagari, Tamil, and other Indic scripts.
- [x] **Shaped galley cache** - Extended `LineCache` to store shaped text runs (glyph IDs + positions) keyed on content+font+width. LRU eviction. Invalidation on content change, font change, or viewport resize.
- [x] **Grapheme-cluster-aware cursor** - Replaced character-based cursor movement with grapheme-cluster-aware navigation using `unicode-segmentation`. Cursor steps over entire clusters for Bengali conjuncts, Korean jamo, emoji ZWJ sequences.
- [x] **Shaped text measurement** - Word wrap, line width, scroll offset, cursor/mouse positioning, and selection rendering all use shaped advance widths for complex-script lines. Latin text unchanged (egui path).

*Note: Phase 2 provides correct rendering of all complex scripts in the Raw editor (FerriteEditor). Text direction remains LTR — Arabic/Hebrew text is shaped correctly (ligatures, contextual forms) but displayed left-to-right. Full RTL layout requires Phase 3 (v0.3.0).*

#### LSP Integration (Language Server Protocol)
*Plan: [docs/lsp-integration-plan.md](docs/lsp-integration-plan.md)*

- [x] **Phase 1: Infrastructure** — Module structure (`src/lsp/`), LspManager with background thread, stdio JSON-RPC transport, extension-to-server detection mapping.
- [x] **Phase 1: Server lifecycle** — Auto-detect language server by file extension; spawn as child process via stdio; crash restart with exponential backoff (1s→30s, reset after 60s uptime); graceful shutdown on workspace close; dismissible notification if binary not found.
- [x] **Phase 1: Status bar & settings** — Status bar indicator (ready / indexing / not found / disabled); LSP toggle in Settings → Editor (opt-in, off by default); per-language server path overrides (`lsp_server_overrides`).
- [x] **Phase 1: On-demand server startup** — Lazy server spawn on tab activation (not eager workspace scan). Idle server shutdown after 30s when last tab for a language closes. Tab-aware reference counting.
- [x] **Phase 1: Windows CMD window suppression** — `CREATE_NO_WINDOW` flag on LSP server process spawn to prevent visible cmd.exe windows.
- [x] **Phase 2: Inline diagnostics** — Error/warning squiggles under text with hover tooltips; incremental document sync (`textDocument/didOpen`, `textDocument/didChange`); diagnostic count in status bar.
- [ ] **Phase 3: Hover & Go to Definition** — Hover documentation with configurable delay; Go to Definition (F12 or Ctrl+Click). *Planned for future release.*
- [ ] **Phase 4: Autocomplete** — Completion popup on typing or Ctrl+Space, debounced, navigable with arrow keys; request cancellation for stale completions. *Planned for future release.*

#### Image & PDF Viewer Tabs ([#108](https://github.com/OlaProeis/Ferrite/issues/108))
- [x] **Image viewer tabs** - Open PNG, JPEG, GIF, WebP, BMP files in a dedicated viewer tab with zoom (Ctrl+scroll), fit-to-window, metadata display (dimensions, format, file size). Reuses existing `image` crate and texture infrastructure.
- [x] **PDF viewer tab** - Open PDF files using [hayro](https://github.com/LaurenzV/hayro) (pure Rust PDF renderer, MIT/Apache-2.0). Page navigation, zoom, texture caching per page. No native C/C++ dependencies.

#### Command Palette ([#59](https://github.com/OlaProeis/Ferrite/issues/59))
- [x] **Alt+Space command launcher** - Searchable command palette with fuzzy search, recent commands, category grouping, shortcut hints. Configurable keybinding (Alt+Space default, Ctrl+Shift+P alternative). All ribbon and keyboard actions accessible.
- [x] **Windows system menu suppression** - `WH_KEYBOARD` thread hook blocks Alt+Space at OS level. Deferred dispatch prevents mid-render crashes.
- [x] **Open/Close Workspace commands** - Added as palette-only commands for folder workspace management.

#### Executable Code Blocks *(deferred to v0.2.9+)*
- [ ] **Run button on code blocks** - Add `▶ Run` button to fenced code blocks.
- [ ] **Shell / Bash execution** - Execute shell snippets via `std::process::Command`.
- [ ] **Python support** - Detect `python` / `python3` and run with system interpreter.
- [ ] **Timeout handling** - Kill long-running scripts after configurable timeout (default: 30s).
- [ ] **Security warning** - First-run dialog explaining execution risks.
  *Security note: Code execution is opt-in and disabled by default.*
  *Deferred from v0.2.7 and v0.2.8 to focus on performance and LSP.*

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

### v0.2.9 - Platform Stack Upgrade, Export & Code Execution
**Primary focus:** **eframe / egui 0.31+** (Task 38) — large dependency migration with full cross-platform QA. Intended to address **Wayland keyboard input** ([#106](https://github.com/OlaProeis/Ferrite/issues/106)), **macOS Sonoma keyboard** ([#111](https://github.com/OlaProeis/Ferrite/issues/111)), and **Windows 11 borderless / DPI** ([#112](https://github.com/OlaProeis/Ferrite/issues/112)) where fixes depend on newer winit/egui. Workarounds (e.g. `WAYLAND_DISPLAY=` on Ubuntu Wayland) remain documented until this ships.

**Secondary focus:** First-class export from markdown to shareable files (PDF, self-contained HTML). Complements **PDF viewer tabs** (v0.2.8)—**writing → publish**, not only viewing external PDFs.

**Tertiary focus:** Executable code blocks (deferred from v0.2.8) and LSP Phases 3 & 4.

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

#### LSP Integration (Phases 3 & 4)
- [ ] **Phase 3: Hover & Go to Definition** — Hover documentation with configurable delay; Go to Definition (F12 or Ctrl+Click).
- [ ] **Phase 4: Autocomplete** — Completion popup on typing or Ctrl+Space, debounced (e.g. 150ms), navigable with arrow keys; request cancellation for stale completions.
- [ ] **Settings** — Per-language server path override; all processing local (no network calls).

*Note: **LaTeX math** rendering in preview and export is planned under **v0.4.0**; PDF/HTML export will pick up formulas once the math engine exists.*

---

### v0.3.0 - Mermaid Crate, Markdown Enhancements, Alt Menus & Full RTL/BiDi
**Focus:** Extracting the Mermaid renderer as a standalone crate, improving markdown rendering, traditional Alt-key menus, and completing right-to-left and bidirectional text support.

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
