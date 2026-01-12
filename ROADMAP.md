# Ferrite Roadmap

## Known Issues 🐛

### Blocked by egui TextEdit
These issues cannot be fixed without replacing egui's built-in text editor:
- [ ] **Multi-cursor incomplete** - Basic cursor rendering works, but text operations not implemented
- [ ] **Code folding incomplete** - Detection works, but text hiding not possible
- [ ] **Scroll sync imperfect** - Limited access to egui's internal scroll state

---

## Planned Features 🚀

### v0.2.5 (Planned) - Mermaid Update, CSV Support & i18n

> **Status:** Planned

#### Mermaid Improvements
- [ ] **YAML frontmatter support** - Parse `---` metadata blocks with `title:`, `config:` etc. (MermaidJS v8.13+ syntax)
- [ ] **Parallel edge operator (`&`)** - Support `A --> B & C & D` syntax for multiple edges from one source
- [ ] **Rendering performance** - Optimize mermaid.rs for complex diagrams with caching
- [ ] **Code cleanup** - Address unused code warnings, improve modularity
- [ ] **Diagram insertion toolbar** ([#4](https://github.com/OlaProeis/Ferrite/issues/4)) - Toolbar button to insert mermaid code blocks
- [ ] **Syntax hints in Help** ([#4](https://github.com/OlaProeis/Ferrite/issues/4)) - Documentation of supported diagram types and syntax examples

#### CSV Support ([#19](https://github.com/OlaProeis/Ferrite/issues/19))
Native CSV file support with specialized viewing and editing capabilities.

- [ ] **CSV Tree Viewer** - Table view with fixed-width column alignment (like EmEditor)
- [ ] **Rainbow column coloring** - Alternating column colors for readability (like RainbowCSV)
- [ ] **Delimiter detection** - Auto-detect comma, tab, semicolon, pipe separators
- [ ] **Header row detection** - Highlight first row as column headers
- [ ] **Large file performance** - Virtual scrolling for CSVs with thousands of rows

#### Internationalization ([#18](https://github.com/OlaProeis/Ferrite/issues/18))
Multi-language UI support with community-driven translations.

- [ ] **i18n infrastructure** - Add `rust-i18n` crate with YAML translation files
- [ ] **String extraction** - Move all UI strings (~300-400) to translation keys
- [ ] **Language selector** - Settings option to choose UI language
- [ ] **Locale detection** - Auto-detect system language on first launch
- [ ] **Weblate integration** - Set up hosted.weblate.org for community translations
- [ ] **Simplified Chinese** - First community translation (thanks @sr79368142!)

#### Bug Fixes & Polish
- [ ] **Session restore reliability** - Investigate workspace folder not being remembered on restart; audit all persistence logic to prevent unnecessary "recover work" dialogs
- [ ] **Recent files persistence** - Audit when/how recent files list is saved and loaded; ensure it survives build-to-build testing
- [ ] **Zen mode rendered centering** - Center content in rendered/split view when Zen mode is active (currently only centers raw text)
- [ ] **Git status auto-refresh** - Refresh git indicators on file save and periodically (every ~10 seconds) instead of only on folder open
- [ ] **Quick switcher mouse support** - Fix mouse hover/click not working (item flickers but doesn't select); arrow keys + Enter work fine
- [ ] **Table editing cursor loss** - Fix cursor losing focus after each keystroke when editing tables in rendered mode (related to previous cursor issues)

#### New Features
- [ ] **Recent folders** - Extend the recent files menu (bottom-left status bar) into a split view with two columns: recent files and recent workspace folders for quick project switching
- [ ] **Keyboard shortcut customization** - Let users rebind shortcuts via settings panel; store in config.json
- [ ] **Drag & drop images** - Drop images into editor → auto-save to `./assets/` folder → insert markdown image link
- [ ] **Table of Contents generation** - Insert/update `<!-- TOC -->` block with auto-generated heading links; keep in sync on save
- [ ] **Document statistics panel** - Tabbed info panel for .md files: Outline tab + Statistics tab (heading count, link count, code block count, image count, word count, reading time, average sentence length)
- [ ] **Snippets/abbreviations** - User-defined text expansions (`;date` → current date, `;sig` → signature block); JSON config in `~/.config/ferrite/snippets.json`

#### Semantic Minimap
Enhanced minimap designed specifically for Markdown documents - show structure, not just pixels.

- [ ] **Header labels** - Display actual H1/H2/H3 text in minimap instead of unreadable scaled pixels
- [ ] **Content type indicators** - Visual markers for code blocks (```), mermaid diagrams, tables, images, blockquotes
- [ ] **Density visualization** - Show text density as subtle horizontal bars between headers
- [ ] **Sleek design** - Minimal, elegant styling that complements the editor aesthetic
- [ ] **Mode toggle** - Settings option to choose "Visual" (current pixel-based) or "Semantic" (new structured) mode

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

#### 3. Custom Editor Widget (Stretch Goal)
Replace egui's `TextEdit` with a custom `FerriteEditor` widget to unblock advanced editing features.

- [ ] **FerriteEditor widget** - Custom text editor using egui drawing primitives
- [ ] **Rope-based buffer** - Efficient text storage via `ropey` crate
- [ ] **Full multi-cursor editing** - Text operations at all cursor positions
- [ ] **Code folding with text hiding** - Actually collapse regions visually

#### 4. Markdown Enhancements
- [ ] **Wikilinks support** ([#1](https://github.com/OlaProeis/Ferrite/issues/1)) - `[[wikilinks]]` syntax with auto-completion
- [ ] **Backlinks panel** ([#1](https://github.com/OlaProeis/Ferrite/issues/1)) - Show documents linking to current file

#### 5. Platform & Distribution
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

### v0.2.3 (Current Release) - Polish & Editor Productivity

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
