# Changelog

All notable changes to Ferrite will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.3] - 2025-01-12

### Added

#### Editor Productivity
- **Go to Line (Ctrl+G)** - Quick navigation to specific line number with modal dialog and viewport centering
- **Duplicate Line (Ctrl+Shift+D)** - Duplicate current line or selection with proper char-to-byte index handling
- **Move Line Up/Down (Alt+↑/↓)** - Rearrange lines without cut/paste, cursor follows moved line
- **Auto-close Brackets & Quotes** - Type `(`, `[`, `{`, `"`, or `'` to get matching pair with cursor in middle; selection wrapping and skip-over behavior
- **Smart Paste for Links** - Select text then paste URL to create `[text](url)` markdown link; image URLs create `![](url)` syntax

#### UX Improvements
- **Configurable line width** ([#15](https://github.com/OlaProeis/Ferrite/issues/15)) - Limit text width for improved readability with presets (Off/80/100/120) or custom value; text centered in viewport

#### Platform & Distribution
- **macOS Intel cross-compilation** - CI now cross-compiles for Intel Macs from ARM64 runner

### Fixed

#### Bug Fixes
- **Task list rendering** - Task list items with inline formatting now render correctly; fixed checkbox alignment and replaced interactive checkboxes with non-interactive ASCII-style `[ ]`/`[x]` markers (interactive editing planned for v0.3.0)
- **macOS Intel support** ([#16](https://github.com/OlaProeis/Ferrite/issues/16)) - Fixed artifact naming for Intel Mac builds; separate x86_64 build via `macos-13` runner
- **Linux close button cursor flicker** - Fixed cursor rapidly switching between pointer/resize near window close button by adding title bar exclusion zone (35px) for north-edge resize detection and cursor caching

### Technical
- Added 7 new technical documentation files in `docs/technical/`
- Extended keyboard shortcut system with pre-render key consumption for move line operations

## [0.2.2] - 2025-01-11

### Added

#### CLI Features
- **Command-line file opening** ([#9](https://github.com/OlaProeis/Ferrite/issues/9)) - Open files directly: `ferrite file.md`, `ferrite file1.md file2.md`, or `ferrite ./folder/`
- **Version and help flags** ([#10](https://github.com/OlaProeis/Ferrite/issues/10)) - Support for `-V/--version` and `-h/--help` CLI arguments
- **Configurable log level** ([#11](https://github.com/OlaProeis/Ferrite/issues/11)) - New `log_level` setting in config.json with CLI override (`--log-level debug|info|warn|error|off`)

#### UX Improvements
- **Default view mode setting** ([#3](https://github.com/OlaProeis/Ferrite/issues/3)) - Choose default view mode (Raw/Rendered/Split) for new tabs in Settings > Appearance

### Fixed

#### Bug Fixes
- **CJK character rendering** ([#7](https://github.com/OlaProeis/Ferrite/issues/7)) - Multi-region CJK support (Korean, Chinese, Japanese) via system font fallback (PR [#8](https://github.com/OlaProeis/Ferrite/pull/8) by [@SteelCrab](https://github.com/SteelCrab) 🙏)
- **Undo/redo behavior** ([#5](https://github.com/OlaProeis/Ferrite/issues/5)) - Fixed scroll position reset, focus loss, double-press requirement, and cursor restoration
- **UTF-8 tree viewer crash** - Fixed string slicing panic when displaying JSON/YAML with multi-byte characters (Norwegian øæå, Chinese, emoji)
- **Misleading code folding UI** ([#12](https://github.com/OlaProeis/Ferrite/issues/12)) - Fold indicators now hidden by default (setting available for power users); removed confusing "Raw View" button from tree viewer toolbar

#### Performance
- **Large file editing** - Deferred syntax highlighting keeps typing responsive in 5000+ line files
- **Scroll performance** - Galley caching for instant syntax colors when scrolling via minimap

### Changed
- **Ubuntu 22.04 compatibility** ([#6](https://github.com/OlaProeis/Ferrite/issues/6)) - Release builds now target Ubuntu 22.04 for glibc 2.35 compatibility

### Documentation
- Added CLI reference documentation (`docs/cli.md`)
- Added technical docs for log level config, default view mode, and code folding UI changes

## [0.2.1] - 2025-01-10

### Added

#### Mermaid Diagram Enhancements
- **Sequence Diagram Control Blocks** - Full support for `loop`, `alt`, `opt`, `par`, `critical`, `break` blocks with proper nesting and colored labels
- **Sequence Activation Boxes** - `activate`/`deactivate` commands and `+`/`-` shorthand on messages for lifeline activation tracking
- **Sequence Notes** - `Note left/right/over` syntax with dog-ear corner rendering
- **Flowchart Subgraphs** - Nested `subgraph`/`end` blocks with semi-transparent backgrounds and direction overrides
- **Composite/Nested States** - State diagrams now support `state Parent { ... }` syntax with recursive nesting
- **Advanced State Transitions** - Color-coded transitions, smart anchor points, and cross-nesting-level edge routing

#### Layout Improvements
- **Flowchart Branching** - Sugiyama-style layered graph layout with proper side-by-side branch placement
- **Cycle Detection** - Back-edges rendered with smooth bezier curves instead of crossing lines
- **Smart Edge Routing** - Decision node edges exit from different points to prevent crossing
- **Edge Declaration Order** - Branch ordering now matches Mermaid's convention (later-declared edges go left)

### Fixed
- **Text Measurement** - Replaced character-count estimation with egui font metrics for accurate node sizing
- **Node Overflow** - Nodes dynamically resize to fit their labels without clipping
- **Edge Labels** - Long labels truncate with ellipsis instead of overflowing
- **User Journey Icons** - Fixed unsupported emoji rendering with text fallbacks

### Technical
- Extended `mermaid.rs` from ~4000 to ~6000+ lines
- Added technical documentation for all new features in `docs/technical/`

## [0.2.0] - 2025-01-09

### Added

#### Major Features
- **Split View** - Side-by-side raw editor and rendered preview with resizable divider and per-tab split ratio persistence
- **MermaidJS Native Rendering** - 11 diagram types rendered natively in Rust/egui (flowchart, sequence, pie, state, mindmap, class, ER, git graph, gantt, timeline, user journey)
- **Editor Minimap** - VS Code-style scaled preview with click-to-navigate, viewport indicator, and search highlights visible in minimap
- **Code Folding** - Fold detection for headings, code blocks, and lists with gutter indicators (▶/▼) and indentation-based folding for JSON/YAML
- **Live Pipeline Panel** - Pipe JSON/YAML content through shell commands with real-time output preview and command history
- **Zen Mode** - Distraction-free writing with centered text column and configurable column width
- **Git Integration** - Visual status indicators in file tree showing modified, added, untracked, and ignored files (using git2 library)
- **Auto-Save** - Configurable delay (default 15s), per-tab toggle, temp-file based for safety
- **Session Persistence** - Restore open tabs on restart with cursor position, scroll offset, view mode, and per-tab split ratio
- **Bracket Matching** - Highlight matching brackets `()[]{}<>` and markdown emphasis pairs `**` and `__` with theme-aware colors

### Fixed
- **Rendered Mode List Editing** - Fixed item index mapping issues, proper structural key hashing, and edit state consistency (Tasks 64-69)
- **Light Mode Contrast** - Improved text and border visibility with WCAG AA compliant contrast ratios, added separator between tabs and editor
- **Scroll Synchronization** - Bidirectional sync between Raw and Rendered modes with hybrid line-based/percentage approach and mode switch scroll preservation
- **Search-in-Files Navigation** - Click result now scrolls to match with transient highlight that auto-clears on scroll or edit
- **Search Panel Viewport** - Fixed top and bottom clipping issues with proper bounds calculation

### Changed
- **Tab Context Menu** - Reorganized icons with logical grouping for better visual clarity

### Technical
- Added ~4000 lines of Mermaid rendering code in `src/markdown/mermaid.rs`
- New modules: `src/vcs/` for git integration, `src/editor/minimap.rs`, `src/editor/folding.rs`, `src/editor/matching.rs`, `src/ui/pipeline.rs`, `src/config/session.rs`
- Comprehensive technical documentation for all major features in `docs/technical/`

### Deferred
- **Multi-cursor editing** (Task 72) - Deferred to v0.3.0, requires custom text editor implementation

## [0.1.0] - 2025-01-XX

### Added

#### Core Editor
- Multi-tab file editing with unsaved changes tracking
- Three view modes: Raw, Rendered, and Split (Both)
- Full undo/redo support per tab (Ctrl+Z, Ctrl+Y)
- Line numbers with scroll synchronization
- Text statistics (words, characters, lines) in status bar

#### Markdown Support
- WYSIWYG markdown editing with live preview
- Click-to-edit formatting for lists, headings, and paragraphs
- Formatting toolbar (bold, italic, headings, lists, links, code)
- Sync scrolling between raw and rendered views
- Syntax highlighting for code blocks (syntect)
- GFM (GitHub Flavored Markdown) support via comrak

#### Multi-Format Support
- JSON file editing with tree viewer
- YAML file editing with tree viewer
- TOML file editing with tree viewer
- Tree viewer features: expand/collapse, inline editing, path copying
- File-type aware adaptive toolbar

#### Workspace Features
- Open folders as workspaces
- File tree sidebar with expand/collapse
- Quick file switcher (Ctrl+P) with fuzzy matching
- Search in files (Ctrl+Shift+F) with results panel
- File system watching for external changes
- Workspace settings persistence (.ferrite/ folder)

#### User Interface
- Modern ribbon-style toolbar
- Custom borderless window with title bar
- Custom resize handles for all edges and corners
- Light and dark themes with runtime switching
- Document outline panel for navigation
- Settings panel with appearance, editor, and file options
- About dialog with version info
- Help panel with keyboard shortcuts reference
- Native file dialogs (open, save, save as)
- Recent files menu in status bar
- Toast notifications for user feedback

#### Export Features
- Export document to HTML file with themed CSS
- Copy as HTML to clipboard

#### Platform Support
- Windows executable with embedded icon
- Linux .desktop file for application integration
- macOS support (untested)

#### Developer Experience
- Comprehensive technical documentation
- Optimized release profile (LTO, symbol stripping)
- Makefile for common build tasks
- Clean codebase with zero clippy warnings

### Technical Details
- Built with Rust 1.70+ and egui 0.28
- Immediate mode GUI architecture
- Per-tab state management
- Platform-specific configuration storage
- Graceful error handling with fallbacks

---

## Version History

- **0.2.3** - Editor productivity release (Go to Line, Duplicate Line, Move Line, Auto-close, Smart Paste, Line Width)
- **0.2.2** - Stability & CLI release (CJK fonts, undo/redo fixes, CLI arguments, default view mode)
- **0.2.1** - Mermaid diagram improvements (control blocks, subgraphs, nested states, improved layout)
- **0.2.0** - Major feature release (Split View, Mermaid, Minimap, Git integration, and more)
- **0.1.0** - Initial public release

[Unreleased]: https://github.com/OlaProeis/Ferrite/compare/v0.2.3...HEAD
[0.2.3]: https://github.com/OlaProeis/Ferrite/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/OlaProeis/Ferrite/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/OlaProeis/Ferrite/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/OlaProeis/Ferrite/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/OlaProeis/Ferrite/releases/tag/v0.1.0
