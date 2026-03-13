# Ferrite Roadmap

## Next Up (Immediate Focus)

### v0.2.7 (Released March 2026) - Performance, Features & Polish
**Focus:** Features moved from v0.2.6 to allow focus on the text editor, plus checking for updates.

#### Bug Fixes & UX
- [x] **CJK fonts load on language switch** - Switching to Chinese or Japanese in the Welcome panel now lazily loads the required CJK font so translated UI labels render correctly instead of showing squares. Uses `Language::required_cjk_font()` to map language → `CjkFontPreference` and `preload_explicit_cjk_font()` to load only the single needed font (~15-20MB).
- [x] **Latin-only names in Language and CJK preference selectors** - Language selector (Settings → Appearance) and CJK Regional Preference (Settings → Editor) used native script (e.g. 简体中文, 日本語, 한국어), which rendered as squares when CJK fonts were not yet loaded (lazy loading). Both dropdowns now use Latin-only display names (e.g. "Chinese (Simplified)", "Japanese", "Korean (Hangul)") so they are always legible without preloading CJK fonts.
- [x] **View mode bar on unsupported file types** - When default view is Split and user opens a file that doesn't support split (e.g. .rs), the view mode bar is hidden so mode can only be changed via hotkeys. Always show the view mode bar: for non–split-supported files use the two-mode segment (Raw | Rendered) so users can switch without hotkeys.
- [x] **Open file in current window as tab** - Single-instance protocol: double-clicking files (file tree or OS/Explorer) opens them as tabs in the existing Ferrite window instead of spawning a new process. Lock file + TCP IPC with background accept thread for instant response (<50ms end-to-end). Early instance check in main() before config/icon loading; `ctx.request_repaint()` wakeup bypasses idle intervals.
- [x] **Images not displaying in rendered mode** - Markdown image syntax `![](path)` now shows images in rendered/split view; path resolution (relative to document dir + workspace root), image loading/caching via `image` crate (PNG, JPEG, GIF, WebP), scaled rendering with aspect ratio, graceful placeholders for missing/unsupported files.
- [x] **CJK rendering after restart with explicit preference** ([#76](https://github.com/OlaProeis/Ferrite/issues/76)) - When "Which CJK font to prioritize" is set to a non-Auto value and the app restarts, Chinese can render as tofu in restored tabs because we only lazy-load CJK for the active tab and don't preload the user's preferred font at startup. Fix: preload the single preferred CJK font at startup when preference is explicit (same approach as Auto + system locale), so restored documents render correctly regardless of which tab is active.
- [x] **Syntax highlighting per-frame re-parsing** - `highlight_line()` was called on every frame for every visible line *before* checking the galley cache, causing severe lag on files with long lines (e.g. dense markdown with inline formatting). Fixed by checking the cache first and only running syntect regex parsing on cache misses.
- [x] **Scrollbar position incorrect with word wrap** - Scrollbar thumb position was calculated using uniform `first_visible_line * line_height`, ignoring actual wrapped line heights. Fixed to use cumulative y-offsets from the height cache, so the scrollbar accurately reflects scroll position and reaches the bottom.
- [x] **Scrollbar drag used wrong reverse mapping** - Dragging the scrollbar converted pixel position to a line number using uniform division, causing inaccurate jumps with word wrap. Fixed to use `y_offset_to_line()` binary search with sub-line precision.
- [x] **Scrollbar jumping as new wrap info discovered** - `rebuild_height_cache` ran every frame (O(N)) and `total_content_height` changed abruptly as previously-unseen wrapped lines were measured during scrolling. Fixed with a dirty flag (only rebuild when wrap info changes) and smoothed scrollbar height that lerps toward the actual value.
- [x] **Crash on large selection delete with word wrap** - Selecting a large block of text top-down and pressing Backspace caused an instant crash (`capacity overflow` panic). Root cause: after deletion, `first_visible_line` remained past the now-shorter document due to stale `wrap_info`/`cumulative_heights`, causing `get_visible_line_range()` to return `start > end` and `Vec::with_capacity(end - start)` to underflow. Fixed with 4 layers: (1) `saturating_sub` on the Vec allocation, (2) hard-clamp `first_visible_line` to `total_lines-1` in `clamp_scroll_position`, (3) clamp `cursor_to_char_pos` result to `buffer.len()`, (4) new `truncate_wrap_info()` to trim stale entries instead of full-clearing (avoids flickering).
- [ ] **Wrapped line scroll stuttering** - Scrolling through documents with many word-wrapped lines still shows micro-stuttering. Likely related to per-line galley layout cost or height cache granularity. Needs further investigation.
- [x] **Light mode text invisible** - All `RichText::strong()` section headers in Settings, Terminal, and other panels were invisible in light mode. Root cause: egui's `strong_text_color()` returns `widgets.active.fg_stroke.color` (set to WHITE for pressed buttons), bypassing `override_text_color`. Fixed by using primary text color for `active.fg_stroke` in light theme.
- [x] **List item text not wrapping in preview** ([#82](https://github.com/OlaProeis/Ferrite/issues/82)) - Long list item text extended beyond the pane edge instead of wrapping in rendered/split view. All 4 list-item `TextEdit` widgets used `singleline` (cannot wrap). Changed to `multiline` with custom `LayoutJob` layouter, `desired_rows(1)`, and newline stripping so Enter doesn't insert literal newlines in list items.
- [x] **Empty list item causes heading mis-render** ([#82](https://github.com/OlaProeis/Ferrite/issues/82)) - Typing `- ` after a paragraph caused the paragraph to render as a heading. Comrak interprets a single `-` + whitespace as a setext heading underline. Added `fix_false_setext_headings()` post-processing in `parser.rs` to convert these back to Paragraph + List(Item).
- [x] **Windows IME backspace deleting editor text** ([#91](https://github.com/OlaProeis/Ferrite/issues/91)) - Pressing Backspace during Chinese/Japanese/Korean IME composition deleted already-committed characters. Raw `Key::Backspace` events are now suppressed while IME composition is active.
- [x] **Crash when opening binary files as text** - Opening image files (PNG, JPEG, etc.) or other binary data as text documents caused a panic: "byte index is not a char boundary". Fixed with (1) binary file detection using null byte and non-printable character heuristics in `state.rs`, (2) safe string slicing in `editor/stats.rs` using `get()` instead of direct byte indexing. Binary files now show a user-friendly error instead of crashing.
- [x] **macOS Release .app Bundle** ([#93](https://github.com/OlaProeis/Ferrite/issues/93)) - Release workflow ships raw binary instead of .app bundle, causing Gatekeeper to block launch. Fix CI to package `target/release/bundle/osx/Ferrite.app` instead of raw binary.
- [x] **Task List Checkbox Rendering** ([#95](https://github.com/OlaProeis/Ferrite/issues/95)) - Fix task list checkboxes showing as ASCII text (`[ ]` / `[x]`) instead of proper checkbox UI elements in rendered view. Also suppress bullet point markers for task list items.
- [x] **Open Folder in Flatpak** ([#96](https://github.com/OlaProeis/Ferrite/issues/96)) - Fix "Open Folder" not working unless a file from that folder is already open. All file dialogs now fall back to `$HOME` when no recent directory is available; added Flatpak environment detection and user-friendly error messages.
- [x] **General Bug Fixes** - Addressed additional issues reported post-v0.2.6.1 release (binary file crash, header spacing, i18n translation loading, complex script font preferences).
- [x] **Ctrl+Mouse Wheel zoom** ([#85](https://github.com/OlaProeis/Ferrite/issues/85)) - Zoom in/out with Ctrl+Scroll using existing zoom logic from Ctrl++/Ctrl+-. ZoomIn/ZoomOut/ResetZoom shortcut commands.
- [x] **Keep text selected after formatting** ([#72](https://github.com/OlaProeis/Ferrite/issues/72)) - After applying Bold/Italic/etc. to selected text, selection is preserved and focus restored so users can chain formatting operations.
- [x] **Linux file dialog error handling** ([#97](https://github.com/OlaProeis/Ferrite/issues/97)) - Detect xdg-desktop-portal failures (e.g. Hyprland without GTK portal) and show helpful error message with distro-specific installation instructions.
- [x] **CJK explicit preference preloading** ([#76](https://github.com/OlaProeis/Ferrite/issues/76)) - Verified non-Auto CJK font preference is preloaded at startup so restored tabs render correctly. `preload_explicit_cjk_font()` handles all explicit preferences, `bump_font_generation()` increments counter, editor invalidates stale galley cache. No code changes required (Task 45)
- [x] **Extra spaces when copying from rendered mode** - Syntax-highlighted code blocks and inline-formatted text inserted phantom spaces at every token boundary when copied (e.g. `json. loads( f. read())`). Fixed by zeroing `item_spacing.x` in code block and inline content layouts.
- [x] **Session recovery dialog after "Don't Save"** - Closing the app with unsaved changes and choosing "Don't Save" caused the recovery dialog to appear on next launch (also closing the workspace). Root cause: `create_lock_file()` called before `load_session_state()` at startup, making `is_crash` always true. Fixed by reordering so the previous run's lock file is checked before the new one is created.

#### Markdown Linking
- [x] **Wikilinks support** ([#1](https://github.com/OlaProeis/Ferrite/issues/1)) - `[[target]]` and `[[target|display]]` syntax with relative path resolution, spaces in filenames, click-to-navigate, ambiguity handling.
- [x] **Backlinks panel** - Panel showing files linking to current file; graph-based indexing for large workspaces; click-to-navigate.

#### Editing Modes
- [x] **Vim Mode** - Optional Vim-style modal editing (Normal / Insert / Visual modes). Essential commands: hjkl, dd, yy, p, /search, v/V selection. Mode indicator in status bar. Toggle in Settings → Editor.

#### Check for Updates
- [x] **Check for Updates button** - Settings → About section with button that checks GitHub Releases API and shows result inline (up-to-date, update available with link, or error). Uses `ureq` (lightweight blocking HTTP) on a background thread with `mpsc` channel for non-blocking UI.
- [x] **Manual Trigger Only** - No automatic or background checking. Strictly user-initiated (offline-first philosophy).
- [x] **Security hardening** - Response URL validated against `https://github.com/OlaProeis/Ferrite/releases/` prefix; malformed URLs are replaced with a constructed safe URL. TLS via `rustls` (pure Rust, no OpenSSL).

#### Large File Performance
- [x] **Large file detection** - Auto-detect files > 10MB on open, show warning toast.
- [x] **Lazy CSV row parsing** - Byte-offset row index (`Vec<u64>`) built on CSV load; only visible rows parsed on demand with viewport caching. Reduces additional memory from ~100-200MB to ~8MB for 1M-row files. Small files (<1MB) now cached instead of re-parsed every frame. Note: files >50MB still bottleneck on initial file I/O and string allocation — memory-mapped I/O is planned for v0.4+.

#### Welcome View
- [x] **Welcome view on first run** - Welcome tab on first launch with configuration for theme, language, editor settings (word wrap, line numbers, minimap, bracket matching, syntax highlighting), max line width, CJK font preference, and auto-save. Only shown when no CLI paths and no session-restored tabs. Contributed by [@blizzard007dev](https://github.com/blizzard007dev) ([PR #80](https://github.com/OlaProeis/Ferrite/pull/80)).

#### Installer, Distribution & Localization
- [x] **Windows MSI installer overhaul** - Complete installer rewrite with WixUI_FeatureTree: optional file associations (.md, .txt, .json, .yaml, .toml, .csv) via OpenWithProgids with per-extension toggles, Explorer context menu ("Open with Ferrite" on files and folders), optional add-to-PATH, desktop shortcut, Windows Default Apps registration (ApplicationCapabilities), and launch-after-install checkbox. All features user-selectable; no forced associations.
- [x] **PortableApps.com Format packaging** - Full PAF-compliant portable distribution. PAF launcher sets `FERRITE_DATA_DIR` env var to redirect all config to `Data\settings\`. No registry, no AppData writes. NSIS-compiled `.paf.exe` installer with LZMA compression. Automated in CI — version, icons, and metadata derived from git tag. Ready for PortableApps.com Beta Testing forum submission.
- [x] **Nix/NixOS flake support** - Official `flake.nix` for reproducible builds and development shells. Supports `nix run`, `nix build`, `nix develop` for x86_64-linux, aarch64-linux, x86_64-darwin, aarch64-darwin. CI workflow for flake evaluation. Declarative NixOS/Home Manager usage. Contributed by [@liuxiaopai-ai](https://github.com/liuxiaopai-ai) ([PR #92](https://github.com/OlaProeis/Ferrite/pull/92)).
- [x] **German and Japanese in Settings** - German (Deutsch) and Japanese (日本語) now available in Settings → Appearance → Language.

#### Table Editing
- [x] **Table background & layout fix** - Replaced pre-painted backgrounds with Shape::Noop placeholder technique. Backgrounds use actual rendered row dimensions, fixing gaps/overlap/text-outside-bounds. Cell layout direction corrected from inherited `left_to_right` to `top_down` for proper vertical padding.
- [x] **Column resizing** - Draggable column separators with resize cursor, visual guide line, min-width enforcement (40px), proportional scaling on window resize, double-click to reset to auto widths. Custom widths stored in egui memory per table via `TableEditState.custom_col_widths`.

#### Frontmatter Editor (Basic) ([#94](https://github.com/OlaProeis/Ferrite/issues/94))
- [x] **Visual frontmatter panel** - FM tab in outline panel for editing YAML frontmatter as key-value pairs with form-based interface, bidirectional sync with source.
- [x] **Basic field type support** - String, date, and list (tags with chip UI) field types with appropriate input widgets.
- [ ] **New file templates** - Optional frontmatter templates when creating new markdown files.

*Note: This is initial/basic frontmatter support. Advanced features (project-wide tag autocomplete, SSG-specific field types like slug/permalink, template system) planned for future releases.*

#### UI Declutter & Edge Toggles
- [x] **Move format toolbar to editor bottom** - Markdown formatting buttons (bold, italic, code, headings, lists, etc.) moved from the ribbon to a collapsible toolbar at the bottom of the raw editor area. Visible in Raw and Split modes for markdown files. Collapse/expand via chevron toggle. Reduces ribbon clutter significantly.
- [x] **Side panel toggle strip** - Replaced separate Outline and Productivity Hub ribbon buttons with a thin toggle strip on the right edge of the editor. Click to open/close the side panel (which contains Outline, Statistics, Backlinks, and Productivity Hub tabs). Consistent UX pattern with the bottom format toolbar.
- [x] **Keyboard shortcuts preserved** - All existing keyboard shortcuts for formatting, outline toggle, and productivity hub continue to work.

#### Refactoring & Quality
- [x] **Flowchart Refactoring** - Modularized 3600-line `flowchart.rs` into 12 focused modules: `flowchart/types.rs`, `parser.rs`, `layout/` (config, graph, subgraph, sugiyama), `render/` (colors, nodes, edges, subgraphs), `utils.rs`.
- [x] **Window Controls** - Redesigned Close, Minimize, Maximize/Restore, and Fullscreen buttons: crisp manually-painted icons (line segments), rounded hover backgrounds (4 px radius), compact size (36 × 22 px). Fixed fullscreen icon (was rendering as ×, now uses proper corner-bracket expand/compress symbols). Re-enabled NE corner resize — 12 px right margin keeps the corner grab zone button-free. `TITLE_BAR_BUTTON_RIGHT_MARGIN` constant documents the sizing invariant in `window.rs`.

#### Unicode & Complex Script Support (Phase 1: Font Loading)
- [x] **Lazy font loading for complex scripts** - Extended the CJK lazy-loading system to cover 22 Unicode ranges across 11 script families: Arabic (5 sub-ranges), Bengali, Devanagari, Thai, Hebrew, Tamil, Georgian, Armenian, Ethiopic, other Indic (Gujarati, Gurmukhi, Kannada, Malayalam, Telugu), and Southeast Asian (Myanmar, Khmer, Sinhala). Per-script atomic load flags, platform-specific system font candidates, and font family fallback chain. ~500 lines in `fonts.rs`, triggers in `app/mod.rs` and `app/central_panel.rs`.
- [x] **Script detection utility** - `detect_complex_scripts()` and `needs_complex_script_fonts()` functions with `ComplexScriptDetection` struct, 17 unit tests covering all script families, boundary checks, and negative cases (ASCII, CJK).
- [x] **Settings UI for script preferences** - Settings → Appearance → Additional Scripts section for pre-selecting fonts per script (Arabic, Bengali, Devanagari, Thai, Hebrew, Tamil, Georgian, Armenian, Ethiopic, Other Indic, Southeast Asian). User preferences tried first before platform defaults; persisted in config.

*Note: Phase 1 provides correct glyph display for scripts that don't require complex shaping (Hebrew, Thai, Cyrillic extended) and partial display for scripts that do (Arabic, Bengali show individual glyphs without ligature/contextual shaping). Full shaping requires Phase 2 (v0.2.8). See [research notes](docs/technical/editor/unicode-complex-scripts.md).*

#### Executable Code Blocks *(deferred to v0.2.8+)*
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
- [x] **IME backspace deleting text** ([#91](https://github.com/OlaProeis/Ferrite/issues/91)) - Fixed in v0.2.7. Backspace during IME composition no longer deletes editor text.

### Deferred (from v0.2.7)
- [ ] **Bidirectional scroll sync** - Editor-Preview scroll synchronization in Split view. Requires deeper investigation into viewport-based line tracking.

### Platform & Distribution
- [x] **macOS Gatekeeper blocking** ([#93](https://github.com/OlaProeis/Ferrite/issues/93)) - Fixed: CI now packages proper `.app` bundle via `cargo-bundle`. Workaround no longer needed.

### Rendered View Limitations
- [ ] **Click-to-edit cursor drift on mixed-format lines** - When clicking formatted text in rendered/split view, cursor may land 1-5 characters off on long lines with mixed formatting.

---

## Planned Features 

### v0.2.8 - UI, Accessibility, Text Shaping & Bug Fixes

#### Bug Fixes
- [ ] **macOS .md file association** ([#102](https://github.com/OlaProeis/Ferrite/issues/102)) - Add `UTImportedTypeDeclarations` to `info_plist_ext.xml` to properly declare the `net.daringfireball.markdown` UTI, enabling Finder "Open With" and double-click file opening

#### Executable Code Blocks
- [ ] **Run button on code blocks** - Add `▶ Run` button to fenced code blocks.
- [ ] **Shell / Bash execution** - Execute shell snippets via `std::process::Command`.
- [ ] **Python support** - Detect `python` / `python3` and run with system interpreter.
- [ ] **Timeout handling** - Kill long-running scripts after configurable timeout (default: 30s).
- [ ] **Security warning** - First-run dialog explaining execution risks.
  *Security note: Code execution is opt-in and disabled by default.*

#### Unicode & Complex Script Support (Phase 2: Text Shaping Engine)
*Depends on: Phase 1 font loading from v0.2.7*

- [ ] **HarfRust integration for FerriteEditor** - Integrate [HarfRust](https://github.com/harfbuzz/harfrust) (pure-Rust HarfBuzz port, v0.5.0+) into the FerriteEditor rendering pipeline for production-quality text shaping. Converts Unicode codepoint sequences into correctly positioned, contextually-formed glyphs for all scripts including Arabic (contextual forms: initial/medial/final/isolated), Bengali (conjunct consonants, vowel reordering), Devanagari, Tamil, and other Indic scripts.
- [ ] **Shaped galley cache** - Extend `LineCache` to store shaped text runs (glyph IDs + positions) instead of raw character galleys. Invalidation on content change, font change, or viewport resize.
- [ ] **Grapheme-cluster-aware cursor** - Replace character-based cursor movement with grapheme-cluster-aware navigation using `unicode-segmentation`. A single visual "character" in Bengali or Arabic may span multiple Unicode codepoints — cursor must step over the entire cluster.
- [ ] **Shaped text measurement** - Update word wrap, line width calculation, and scroll offset computation to use shaped advance widths instead of per-character metrics.

*Note: Phase 2 provides correct rendering of all complex scripts in the Raw editor (FerriteEditor). Text direction remains LTR — Arabic/Hebrew text will be shaped correctly (ligatures, contextual forms) but displayed left-to-right. Full RTL layout requires Phase 3 (v0.3.0). WYSIWYG/rendered view inherits egui's text pipeline and will not benefit until egui itself adds shaping support or Phase 4.*

*Background: egui (as of 0.33) uses `ab_glyph` which does glyph-by-glyph rendering with no shaping. PR [#5784](https://github.com/emilk/egui/pull/5784) to integrate Parley is stalled (Nov 2025). We cannot wait for upstream — HarfRust integration in our custom editor widget is the pragmatic path.*

#### LSP Integration (Language Server Protocol)
*Plan: [docs/lsp-integration-plan.md](docs/lsp-integration-plan.md)*

- [ ] **Phase 1: Infrastructure** — Auto-detect language server by file extension; spawn server as child process via stdio on workspace open; graceful fallback with dismissable notification if not installed; server lifecycle (start, restart on crash, shutdown on close); LSP toggle per workspace (opt-in, off by default); status bar indicator (ready / indexing / not found).
- [ ] **Phase 2: Inline diagnostics** — Error/warning squiggles under text with hover tooltip; incremental document sync (changed ranges only); diagnostic count in status bar (e.g. 2 errors, 1 warning).
- [ ] **Phase 3: Hover & Go to Definition** — Hover documentation with configurable delay; Go to Definition (F12 or Ctrl+Click).
- [ ] **Phase 4: Autocomplete** — Completion popup on typing or Ctrl+Space, debounced (e.g. 150ms), navigable with arrow keys; request cancellation for stale completions.
- [ ] **Settings** — Per-language server path override; all processing local (no network calls).

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

### v0.3.0 - Mermaid Crate, Markdown Enhancements & Full RTL/BiDi
**Focus:** Extracting the Mermaid renderer as a standalone crate, improving markdown rendering, and completing right-to-left and bidirectional text support.

#### 0. Unicode & Complex Script Support (Phase 3 & 4: RTL, BiDi, WYSIWYG)
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
