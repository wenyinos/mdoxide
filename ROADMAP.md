# Ferrite Roadmap

## Known Issues 🐛

### Blocked by egui TextEdit
These issues cannot be fixed without replacing egui's built-in text editor:
- [ ] **Multi-cursor incomplete** - Basic cursor rendering works, but text operations not implemented
- [ ] **Code folding incomplete** - Detection works, but text hiding not possible
- [ ] **Scroll sync imperfect** - Limited access to egui's internal scroll state
- [ ] **IME candidate box positioning** ([#15](https://github.com/OlaProeis/Ferrite/issues/15)) - Chinese/Japanese IME candidate window appears offset from cursor position; egui's IME support is limited
- [ ] **IME undo behavior** ([#15](https://github.com/OlaProeis/Ferrite/issues/15)) - Undoing during IME composition may delete an extra character; related to egui's text input handling

---

## Planned Features 🚀

### v0.2.5 (Released) - Mermaid Update & Editor Polish

> **Status:** Released (2026-01-16)

#### Mermaid Improvements
- [x] **Modular refactor** - Split 7000+ line `mermaid.rs` into `src/markdown/mermaid/` directory with separate files per diagram type
- [x] **Edge parsing fixes** - Fix chained edge parsing (`A --> B --> C`), arrow pattern matching, label extraction
- [x] **Flowchart direction fix** - Respect LR/TB/RL/BT direction keywords in layout algorithm
- [x] **Node detection fixes** - Fix missing nodes and improve branching layout in complex flowcharts
- [x] **YAML frontmatter support** - Parse `---` metadata blocks with `title:`, `config:` etc. (MermaidJS v8.13+ syntax)
- [x] **Parallel edge operator (`&`)** - Support `A --> B & C & D` syntax for multiple edges from one source
- [x] **Rendering performance** - AST and layout caching with blake3 hashing for complex diagrams
- [x] **Semicolon & ampersand syntax** - Support Mermaid semicolon line terminators and `&` parallel edge syntax
- [x] **classDef/class styling** - Node styling via `classDef` and `class` directives
- [x] **linkStyle edge styling** - Edge customization via `linkStyle` directive
- [x] **Subgraph improvements** - Layer clustering, internal layout, edge routing, title expansion, nested margins
- [x] **Asymmetric shape rendering** - Flag/asymmetric node shape with proper text centering
- [x] **Viewport clipping fix** - Prevent diagram clipping with negative coordinate shifting
- [x] **Crash prevention** - Infinite loop safety, panic handling for malformed input

#### CSV Support ([#19](https://github.com/OlaProeis/Ferrite/issues/19))
- [x] **CSV/TSV viewer** - Native table view for CSV and TSV files with fixed-width column alignment
- [x] **Rainbow column coloring** - Alternating column colors for improved readability
- [x] **Delimiter detection** - Auto-detect comma, tab, semicolon, pipe separators
- [x] **Header row detection** - Intelligent detection and highlighting of header rows

#### Internationalization ([#18](https://github.com/OlaProeis/Ferrite/issues/18))
- [x] **i18n infrastructure** - YAML translation files in `locales/` directory
- [x] **String extraction** - UI strings moved to translation keys
- [x] **Weblate integration** - Community translations via [hosted.weblate.org/projects/ferrite](https://hosted.weblate.org/projects/ferrite/)

#### CJK Writing Conventions ([#20](https://github.com/OlaProeis/Ferrite/issues/20))
- [x] **Paragraph indentation setting** - New option in Settings: Off / Chinese (2 chars) / Japanese (1 char) / Custom
- [x] **Rendered view support** - Apply `text-indent` styling to paragraphs in preview mode

#### Split View Enhancements
- [x] **Dual editable panes** - Split view rendered pane is now fully editable, matching full Rendered mode behavior with undo/redo support

#### Bug Fixes & Polish
- [x] **Search highlight drift** - Fixed find/search highlight boxes drifting progressively from matched text (byte vs character position mismatch)
- [x] **Config.json persistence** ([#15](https://github.com/OlaProeis/Ferrite/issues/15)) - Fixed window state dirty flag, settings now persist correctly across restarts
- [x] **Zen mode rendered centering** - Center content in rendered/split view when Zen mode (F11) is active
- [x] **Git status auto-refresh** - Refresh git indicators on file save, window focus, periodically (every ~10 seconds), and on file system events
- [x] **Quick switcher mouse support** - Fixed mouse hover/click not working in quick switcher
- [x] **Table editing cursor loss** - Fix cursor losing focus after each keystroke when editing tables in rendered mode (deferred update model)
- [x] **Line width in rendered/split view** ([#15](https://github.com/OlaProeis/Ferrite/issues/15)) - Fixed line width setting respecting pane boundaries with proper centering behavior
- [x] **Windows top edge resize** ([#15](https://github.com/OlaProeis/Ferrite/issues/15)) - Window can now be resized from all edges including top
- [x] **macOS Intel CPU optimization** ([#24](https://github.com/OlaProeis/Ferrite/issues/24)) - Idle repaint scheduling to reduce CPU usage on Intel Macs

#### New Features
- [x] **Keyboard shortcut customization** - Users can rebind shortcuts via settings panel; stored in config.json
- [x] **Custom font selection** ([#15](https://github.com/OlaProeis/Ferrite/issues/15)) - Select preferred font for editor and UI; important for CJK regional glyph preferences
- [x] **Main menu UI redesign** - Modernized main menu with improved layout and visual design
- [x] **Windows fullscreen toggle** ([#15](https://github.com/OlaProeis/Ferrite/issues/15)) - Dedicated fullscreen button (F10) separate from Zen mode (F11)
- [x] **Session restore reliability** - Workspace folders and recent files now persist correctly with atomic file writes
- [x] **Recent files persistence** - Recent files list saves immediately on file open, pruning stale paths
- [x] **Recent folders** - Recent files menu now includes workspace folders
- [x] **Drag & drop images** - Drop images into editor → auto-save to `./assets/` → insert markdown link
- [x] **Table of Contents generation** - Insert/update `<!-- TOC -->` block with auto-generated heading links (Ctrl+Shift+U)
- [x] **Document statistics panel** - Tabbed info panel: Outline + Statistics (word count, reading time, heading/link/image counts)
- [x] **Snippets/abbreviations** - User-defined text expansions (`;date` → current date, `;time` → current time)

#### Semantic Minimap
- [x] **Header labels** - Display actual H1/H2/H3 text in minimap instead of unreadable scaled pixels
- [x] **Content type indicators** - Visual markers for code blocks, mermaid diagrams, tables, images
- [x] **Density visualization** - Show text density as subtle horizontal bars between headers
- [x] **Mode toggle** - Settings option to choose "Visual" or "Semantic" mode

#### Branding
New Ferrite logo and icon set.

- [x] **New logo design** - Ferrite crystal icon (orange geometric crystal shape)
- [x] **Windows icon** - Multi-size `.ico` file (16, 32, 48, 256px) embedded in executable
- [x] **macOS iconset** - `.iconset` folder for CI-generated `.icns`
- [x] **Linux icons** - PNG icons for `.deb` package (16-512px)
- [x] **Window icon** - Embedded 256px icon replaces default eframe "E" logo
- [x] **Icon generation script** - `assets/icons/generate_all_icons.py` for regenerating all sizes

---

### v0.2.6 (Planned) - Large File Performance & Polish

> **Status:** Planned

v0.2.6 focuses on **large file performance** (handling 80MB+ CSV files), code cleanup, and polish. egui's TextEdit cannot handle massive text buffers efficiently, so large files require a specialized read-only viewing mode.

#### Large File Performance ([#19](https://github.com/OlaProeis/Ferrite/issues/19) partial)
Critical performance improvements for handling large CSV/data files:

- [ ] **Large file detection** - Auto-detect files > 10MB on open, show warning toast
- [ ] **View-only mode for large files** - Disable Raw view editing for files > threshold (egui TextEdit can't handle 80MB)
- [ ] **Lazy CSV row parsing** - Parse rows on-demand using byte offset index instead of loading all rows into `Vec<Vec<String>>`
- [ ] **Row offset indexing** - First pass scans file to record byte offsets of each row start; parse only visible rows + buffer
- [ ] **LRU row cache** - Cache recently parsed rows (max ~10K rows) for smooth scrolling
- [ ] **Background CSV scanning** - Scan file in background thread with progress indicator; show first 1000 rows immediately
- [ ] **Disable expensive features** - Skip minimap, syntax highlighting, and undo history for large files
- [ ] **Memory optimization** - Don't store both raw content and parsed rows; use slices into original string where possible

#### Flowchart Refactoring
- [ ] **Modular refactor** - Split the 3500+ line `flowchart.rs` into smaller, maintainable modules (parser, layout, renderer, shapes, edges)
- [ ] **Code cleanup** - Improve code organization, reduce duplication, add documentation

#### Mermaid Improvements
Additional mermaid fixes and enhancements:

- [ ] **Testing & validation** - Comprehensive testing of all diagram types with edge cases
- [ ] **Bug fixes** - Address rendering issues discovered during v0.2.5 testing

#### Bug Fixes & Polish
- [ ] **macOS Intel sync scrolling** ([#24](https://github.com/OlaProeis/Ferrite/issues/24)) - Bidirectional scroll sync between Raw/Rendered views on Intel Macs
- [ ] **macOS window controls** ([#24](https://github.com/OlaProeis/Ferrite/issues/24)) - Native traffic light style instead of Windows-style icons

#### Internationalization Polish
- [ ] **Language selector** - Settings option to choose UI language
- [ ] **Locale detection** - Auto-detect system language on first launch
- [ ] **Simplified Chinese** - First community translation (thanks @sr79368142!)
- [ ] **HTML export i18n** - Include CJK paragraph indentation in exported HTML

#### Executable Code Blocks
Run code snippets directly in the rendered preview — inspired by Jupyter notebooks and Marco.

- [ ] **Run button on code blocks** - Add "▶ Run" button to fenced code blocks in rendered/split view
- [ ] **Shell/Bash execution** - Execute shell scripts via `std::process::Command`; display stdout/stderr below block
- [ ] **Python support** - Detect `python` or `python3` and run with system interpreter
- [ ] **Output persistence** - Option to keep output visible or clear on re-run
- [ ] **Timeout handling** - Kill long-running scripts after configurable timeout (default 30s)
- [ ] **Security warning** - First-run dialog warning about code execution risks; require opt-in in settings

> **Security Note:** Code execution is inherently risky. This feature will be opt-in and disabled by default. Users must explicitly enable it in settings.

#### Content Blocks / Callouts
Styled callout blocks for notes, warnings, tips — common in technical documentation (Obsidian, Notion, GitHub).

- [ ] **GitHub-style syntax** - Support `> [!NOTE]`, `> [!TIP]`, `> [!WARNING]`, `> [!CAUTION]`, `> [!IMPORTANT]`
- [ ] **Custom titles** - Support `> [!NOTE] Custom Title` syntax
- [ ] **Styled rendering** - Color-coded blocks with icons (ℹ️ 💡 ⚠️ 🔴 ❗) in rendered view
- [ ] **Collapsible variant** - `> [!NOTE]- Collapsed by default` syntax for expandable sections
- [ ] **Nesting support** - Allow content blocks inside other blocks and lists

---

### v0.3.0 (Planned) - Mermaid Crate + Editor Improvements

> **Status:** Planning  
> **Docs:** [Mermaid Crate Plan](docs/mermaid-crate-plan.md) | [Custom Editor Plan](docs/technical/custom-editor-widget-plan.md) | [Modular Refactor Plan](docs/refactor.md)

v0.3.0 focuses on extracting the Mermaid renderer as a standalone crate and continuing diagram improvements.

#### 1. Mermaid Crate Extraction
Extract Ferrite's native Mermaid renderer (~6000 lines) into a standalone pure-Rust crate.

- [ ] **Standalone crate** - Backend-agnostic architecture with SVG, PNG, and egui outputs
- [ ] **Public API** - `parse()`, `layout()`, `render()` pipeline
- [ ] **SVG export** - Generate valid SVG files from diagrams
- [ ] **PNG export** - Rasterize via resvg
- [ ] **WASM compatible** - SVG backend works in browsers

#### 2. Mermaid Diagram Improvements
Continue improving diagram rendering quality:

##### Deferred from v0.2.5
- [ ] **Diagram insertion toolbar** ([#4](https://github.com/OlaProeis/Ferrite/issues/4)) - Toolbar button to insert mermaid code blocks with template syntax
- [ ] **Syntax hints in Help** ([#4](https://github.com/OlaProeis/Ferrite/issues/4)) - Help panel documenting supported diagram types with syntax examples

##### Git Graph (Major Rewrite)
- [ ] **Horizontal timeline layout** - Left-to-right commit flow like Mermaid
- [ ] **Branch lanes** - Distinct horizontal lanes per branch with colored labels
- [ ] **Merge visualization** - Curved paths connecting branches
- [ ] **Tags and highlights** - Visual markers on commits

##### Flowchart
- [ ] **More node shapes** - Parallelogram, trapezoid, double-circle, etc.
- [ ] **Styling syntax** - `style` and `classDef` directives

##### State Diagram
- [ ] **Fork/join pseudostates** - Parallel regions
- [ ] **History states** - Shallow (H) and deep (H*) history

##### Manual Layout Support
Enable manual node positioning while maintaining mermaid.js compatibility — a key differentiator for mermaid-rs.

- [ ] **Comment-based position hints** - Parse `%% @pos <node_id> <x> <y>` directives (ignored by mermaid.js, respected by Ferrite)
- [ ] **Layout mode toggle** - Support `%% @ferrite-layout: manual` to enable manual positioning
- [ ] **Drag-to-reposition** - Drag nodes in rendered view → auto-update source with position comments
- [ ] **Export options** - "Export clean" strips layout hints for sharing pure Mermaid syntax
- [ ] **Fallback behavior** - Diagrams without position hints use auto-layout (Sugiyama, etc.)

> **Why this matters:** Mermaid is declarative — layout is computed, not specified. This prevents "visual thinking" workflows where users want to arrange diagrams as thought tools. By using `%%` comments (which mermaid.js ignores), we add manual positioning without breaking compatibility. Diagrams remain valid Mermaid syntax and render everywhere — just with different layouts.

#### 3. Custom Editor Widget (Stretch Goal)
Replace egui's `TextEdit` with a custom `FerriteEditor` widget to unblock advanced editing features.

- [ ] **FerriteEditor widget** - Custom text editor using egui drawing primitives
- [ ] **Rope-based buffer** - Efficient text storage via `ropey` crate
- [ ] **Full multi-cursor editing** - Text operations at all cursor positions
- [ ] **Code folding with text hiding** - Actually collapse regions visually

#### 4. Semantic Minimap Polish
- [ ] **Scroll position accuracy** - Fix navigation centering for variable line heights, word wrap, and editor padding (deferred from v0.2.5)

#### 5. Markdown Enhancements
- [ ] **Wikilinks support** ([#1](https://github.com/OlaProeis/Ferrite/issues/1)) - `[[wikilinks]]` syntax with auto-completion
- [ ] **Backlinks panel** ([#1](https://github.com/OlaProeis/Ferrite/issues/1)) - Show documents linking to current file

#### 6. HTML Rendering (GitHub Parity)
Render embedded HTML in markdown preview, matching GitHub's supported subset.

##### Phase 1: Block Elements
- [ ] **`<div align="...">`** - Center/left/right alignment for content blocks
- [ ] **`<details><summary>`** - Collapsible sections using egui's CollapsingHeader
- [ ] **`<br>`** - Explicit line breaks

##### Phase 2: Inline Elements
- [ ] **`<kbd>`** - Keyboard key styling (monospace with border)
- [ ] **`<sup>` / `<sub>`** - Superscript and subscript text
- [ ] **`<img>` attributes** - Respect width/height on image tags

##### Phase 3: Advanced
- [ ] **Nested HTML** - HTML containing markdown containing HTML
- [ ] **`<table>` (HTML tables)** - Render HTML table syntax (separate from GFM pipe tables)

> **Note:** Only safe HTML elements supported — no `<script>`, `<style>`, `<iframe>`, or event handlers. This matches GitHub's security model.

#### 7. Platform & Distribution
Improve installation experience across all platforms.

##### Windows Installer
- [ ] **Inno Setup installer** - Professional `.exe` installer (like the Linux `.deb`)
- [ ] **File associations** - Register as handler for `.md`, `.json`, `.yaml`, `.toml` files
- [ ] **Context menu integration** - "Open with Ferrite" for files and folders (workspace mode)
- [ ] **Add to PATH option** - Run `ferrite` from any terminal
- [ ] **Start Menu & Desktop shortcuts** - Standard Windows integration
- [ ] **Clean uninstaller** - Remove all registry entries on uninstall
- [ ] **CI automation** - Build installer automatically in GitHub Actions release workflow

##### macOS
- [ ] **App signing & notarization** - Create proper `.app` bundle, sign with Developer ID, notarize with Apple

### v0.4.0 (Planned) - TeX Math Support

> **Status:** Planning  
> **Docs:** [Math Support Plan](docs/math-support-plan.md)

Native LaTeX/TeX math rendering - the most requested feature for academic and technical writing. Pure Rust implementation, no JavaScript dependencies.

#### Math Rendering Engine
- [ ] **LaTeX parser** - Parse `$...$` (inline) and `$$...$$` (display) syntax
- [ ] **Layout engine** - TeX-style box model for fractions, subscripts, radicals
- [ ] **Math fonts** - Embedded glyph subset for consistent cross-platform rendering
- [ ] **egui integration** - Render math in preview and split views

#### Supported LaTeX (Target)
- [ ] **Fractions** - `\frac{a}{b}` with proper stacking
- [ ] **Subscripts/superscripts** - `x^2`, `x_i`, `x_i^2`
- [ ] **Greek letters** - `\alpha`, `\beta`, `\pi`, etc.
- [ ] **Operators** - `\sum`, `\int`, `\prod`, `\lim`
- [ ] **Roots** - `\sqrt{x}`, `\sqrt[n]{x}`
- [ ] **Delimiters** - Auto-scaling `\left( \right)`
- [ ] **Matrices** - `\begin{matrix}...\end{matrix}`
- [ ] **Font styles** - `\mathbf`, `\mathit`, `\mathrm`

#### WYSIWYG Features (Requires FerriteEditor from v0.3.0)
- [ ] **Inline math preview** - See rendered math while typing (Typora-style)
- [ ] **Click-to-edit** - Click rendered math to edit source
- [ ] **Symbol palette** - Quick access to common symbols

---

### Future (v0.5.0+)
- [ ] **Memory-mapped file I/O** ([#19](https://github.com/OlaProeis/Ferrite/issues/19)) - Handle GB-scale CSV/JSON files efficiently without loading into RAM
- [ ] **TODO list editing UX** - Smart cursor behavior in task lists (respect line start position, don't jump past `- [ ]` syntax)
- [ ] Spell checking
- [ ] Custom themes (import/export)
- [ ] Virtual/ghost text (AI completions, etc.)
- [ ] Column/box selection

#### Additional Markup Formats ([#21](https://github.com/OlaProeis/Ferrite/issues/21))
Support for markup languages beyond Markdown, enabled by the plugin system.

- [ ] **AsciiDoc support** - Parser and renderer for AsciiDoc syntax (requires plugin system or native Rust parser)
- [ ] **Zim-Wiki support** - Parser and renderer for Zim Desktop Wiki syntax
- [ ] **Format auto-detection** - Detect markup format from file extension or content

### Long-Term Vision

#### Plugin System
Extensibility architecture for custom functionality, inspired by Obsidian's plugin ecosystem.

- [ ] **Plugin API design** - Define extension points (commands, views, file handlers)
- [ ] **Scripting support** - Lua, WASM, or Rhai-based plugins
- [ ] **Community plugins** - Distribution and discovery mechanism

#### Headless Editor Library
Extract `FerriteEditor` as a standalone, framework-agnostic text editing library for the Rust ecosystem.

> **Context:** There's currently no general-purpose "headless" code editor library in Rust. Existing implementations (egui's TextEdit, Lapce/Floem, Zed/gpui) are tightly coupled to their UI frameworks. The v0.3.0 custom editor and modular architecture lay the groundwork for potential extraction.

**Prerequisites (from v0.3.0):**
- Custom `FerriteEditor` widget with rope-based buffer
- Modular architecture with clean separation of concerns
- Framework-agnostic core logic

**Extraction would involve:**
- [ ] Abstract rendering backend (trait-based: egui, wgpu, vello, SVG, etc.)
- [ ] Framework-agnostic input handling
- [ ] Standalone crate with minimal dependencies
- [ ] Integration with [Parley](https://github.com/linebender/parley) for advanced text layout/shaping (optional)

---

## Completed ✅

### v0.2.5 (Current Release) - Mermaid Update & Editor Polish

See [CHANGELOG.md](CHANGELOG.md) for full release notes. Key highlights:
- **Mermaid modular refactor** - Split 7000+ line file into maintainable modules
- **Mermaid improvements** - YAML frontmatter, parallel edges, classDef/linkStyle, subgraph improvements
- **CSV/TSV viewer** - Native table view with rainbow columns, delimiter detection, header detection
- **Semantic minimap** - Header labels, content type indicators, density visualization, mode toggle
- **i18n infrastructure** - String extraction, YAML translation files, Weblate integration
- **CJK paragraph indentation** - First-line indentation for Chinese/Japanese text
- **Custom font selection** - Select preferred fonts for editor and UI
- **Main menu UI redesign** - Modernized layout and visual design
- **Split view dual editing** - Both panes now fully editable with undo/redo support
- **Keyboard shortcut customization** - Rebind shortcuts via settings panel
- **Git status auto-refresh** - Automatic refresh on save, focus, timer, and file events
- **Drag & drop images** - Drop images to auto-save to ./assets/ and insert markdown link
- **Table of Contents generation** - Generate/update TOC with Ctrl+Shift+U
- **Document statistics** - Tabbed panel with word count, reading time, heading/link/image counts
- **Snippets** - Text expansions (`;date`, `;time`) with custom snippet support
- **Recent folders** - Recent files menu now includes workspace folders
- **Windows fullscreen toggle** - F10 for fullscreen (separate from F11 Zen mode)
- **Bug fixes** - Session persistence, table editing, quick switcher, config persistence, line width, window resize

### v0.2.3 - Polish & Editor Productivity

A focused release adding editor productivity features and platform improvements.

#### Editor Productivity
- [x] **Go to Line (Ctrl+G)** - Quick navigation to specific line number with modal dialog
- [x] **Duplicate Line (Ctrl+Shift+D)** - Duplicate current line or selection
- [x] **Move Line Up/Down (Alt+↑/↓)** - Rearrange lines without cut/paste
- [x] **Auto-close Brackets & Quotes** - Type `(` to get `()` with cursor in middle
- [x] **Smart Paste for Links** - Select text, paste URL → creates `[text](url)` markdown link

#### UX Improvements
- [x] **Configurable line width** ([#15](https://github.com/OlaProeis/Ferrite/issues/15)) - Option to limit text width for improved readability (Off/80/100/120/Custom)

#### Platform & Distribution
- [x] **Linux musl build** - Statically-linked musl binary for maximum Linux compatibility (no glibc dependency)

#### Bug Fixes
- [x] **Linux close button cursor flicker** - Fixed cursor rapidly switching between pointer/move/resize near window close button (title bar exclusion zone)

### v0.2.2 - Performance & Stability

A focused release addressing bugs reported after v0.2.1 launch, improving CLI usability, and adding quality-of-life features.

#### Bug Fixes
- [x] **UTF-8 crash in tree viewer** - Fix string slicing panic when displaying JSON/YAML strings containing multi-byte characters (Norwegian øæå, Chinese, emoji, etc.)
- [x] **Ubuntu 22.04 .deb compatibility** ([#6](https://github.com/OlaProeis/Ferrite/issues/6)) - Build on Ubuntu 22.04 for glibc 2.35 compatibility
- [x] **Undo/redo behavior** ([#5](https://github.com/OlaProeis/Ferrite/issues/5)) - Fixed scroll position reset, focus loss, double-press requirement, and cursor restoration on Ctrl+Z
- [x] **Misleading code folding UI** ([#12](https://github.com/OlaProeis/Ferrite/issues/12)) - Hide non-functional fold indicators by default; remove confusing "Raw View" button from Rendered JSON view
- [x] **CJK character rendering** ([#7](https://github.com/OlaProeis/Ferrite/issues/7)) - ✅ Multi-region CJK support (Korean, Chinese, Japanese) via system font fallback using `font-kit` (PR [#8](https://github.com/OlaProeis/Ferrite/pull/8) by [@SteelCrab](https://github.com/SteelCrab) 🙏)
- [x] **macOS Intel support** ([#16](https://github.com/OlaProeis/Ferrite/issues/16)) - Separate x86_64 build for Intel Macs via `macos-13` runner (PR [#2](https://github.com/OlaProeis/Ferrite/pull/2) fixed naming, Intel job added)

#### Performance Optimizations
- [x] **Large file performance** - Deferred syntax highlighting keeps typing responsive in 5000+ line files
- [x] **Syntax highlighting optimization** - Galley caching for instant scrolling, deferred re-highlighting while typing
- [x] **Scroll performance** - Instant syntax colors when scrolling/jumping via minimap

#### UX Improvements
- [x] **Default view mode setting** ([#3](https://github.com/OlaProeis/Ferrite/issues/3)) - Option to set default view mode (Raw/Rendered/Split) for new tabs

#### CLI Improvements
- [x] **Command-line file opening** ([#9](https://github.com/OlaProeis/Ferrite/issues/9)) - `ferrite file.md` opens file directly in editor
- [x] **Version/help flags** ([#10](https://github.com/OlaProeis/Ferrite/issues/10)) - `-V/--version` and `-h/--help` CLI support
- [x] **Configurable log level** ([#11](https://github.com/OlaProeis/Ferrite/issues/11)) - `log_level` setting in config.json with CLI override (`--log-level`)

### v0.2.1

#### Mermaid Diagram Enhancements
- [x] **Accurate text measurement** - Replace character-count estimation with egui font metrics
- [x] **Dynamic node sizing** - Nodes resize to fit their labels without clipping
- [x] **Text overflow handling** - Edge labels truncate with ellipsis when too long
- [x] **User Journey icons** - Fixed unsupported emoji rendering with text fallbacks
- [x] **Sequence control-flow blocks** - Support for `loop`, `alt`, `opt`, `par`, `critical`, `break` blocks with nesting
- [x] **Sequence activation boxes** - `activate`/`deactivate` markers and `+`/`-` shorthand on lifelines
- [x] **Sequence notes** - `Note left/right/over` syntax support with dog-ear rendering
- [x] **Flowchart branching layout** - Sugiyama-style layered graph with side-by-side branches
- [x] **Flowchart subgraphs** - Nested `subgraph`/`end` blocks with direction overrides
- [x] **Back-edge routing** - Cycle edges rendered with smooth bezier curves
- [x] **Smart edge exit points** - Decision node edges exit from different points to prevent crossing
- [x] **Composite/nested states** - `state Parent { ... }` syntax with recursive nesting
- [x] **Advanced state transitions** - Color-coded transitions and smart anchor points

### v0.2.0

#### Major Features
- [x] **Side-by-side split view** - Raw editor on left, rendered preview on right with resizable divider
- [x] **MermaidJS native rendering** - 11 diagram types rendered natively in Rust/egui (flowchart, sequence, pie, state, mindmap, class, ER, git graph, gantt, timeline, user journey)
- [x] **Editor minimap** - VS Code-style scaled preview with click-to-navigate and viewport indicator
- [x] **Code folding indicators** - Fold detection for headings, code blocks, lists; gutter indicators (▶/▼)
- [x] **Live Pipeline panel** - Pipe JSON/YAML content through shell commands with real-time output
- [x] **Zen Mode** - Distraction-free writing with centered text column
- [x] **Git integration** - Visual status indicators in file tree (modified, added, untracked, ignored)
- [x] **Auto-save** - Configurable delay, per-tab toggle, temp-file based safety
- [x] **Session persistence** - Restore open tabs, cursor position, scroll offset, view mode on restart
- [x] **Bracket matching** - Highlight matching brackets `()[]{}<>` and markdown emphasis `**` `__`
- [x] **Syntax highlighting** - Full-file syntax highlighting for source code files (40+ languages including Rust, Python, JavaScript, Go, C/C++, etc.)

#### Bug Fixes
- [x] **Rendered mode list editing** - Fixed item index mapping, structural key hashing, edit state consistency
- [x] **Light mode contrast** - Improved text/border visibility, WCAG AA compliant, added tab/editor separator
- [x] **Scroll synchronization** - Bidirectional sync between Raw/Rendered, mode switch preservation
- [x] **Search-in-Files navigation** - Click result scrolls to match with transient highlight
- [x] **Search panel viewport** - Fixed top/bottom clipping issues

#### UX Improvements
- [x] **Tab context menu** - Reorganized icons with logical grouping

### v0.1.0

#### Core Features
- [x] WYSIWYG Markdown editing
- [x] Multi-format support (Markdown, JSON, YAML, TOML)
- [x] Tree viewer for structured data
- [x] Workspace mode with file tree
- [x] Quick switcher (Ctrl+P)
- [x] Search in files (Ctrl+Shift+F)
- [x] Light and dark themes
- [x] Document outline panel
- [x] HTML export
- [x] Formatting toolbar
- [x] Custom borderless window
- [x] Multi-tab editing
- [x] Find and replace
- [x] Undo/redo per tab

---

## Contributing

Found a bug or have a feature request? Please [open an issue](https://github.com/OlaProeis/Ferrite/issues/new/choose) on GitHub!
