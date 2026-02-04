# Ferrite Roadmap

## Next Up (Immediate Focus)

### v0.2.6.1 (Planned) - Patch & Stability
**Focus:** Critical bug fixes, code signing, and stability improvements following the major editor rewrite.

- [ ] **Code Signing** - Windows artifacts (exe, MSI) will be code signed via SignPath.io (pending approval).
- [ ] **CJK Font Crash on Startup** ([#63](https://github.com/OlaProeis/Ferrite/issues/63))  
  Fix crash caused by invalid persisted CJK font configuration when a non-Auto CJK preference is selected but the corresponding system font cannot be loaded. Startup will defensively validate persisted font/CJK settings and fall back to *Auto* instead of crashing. Also resolves missing glyphs (tofu □) in settings UI.
- [ ] **Portable Windows Startup Crash** ([#57](https://github.com/OlaProeis/Ferrite/issues/57))  
  Validate persisted window position values on load. Corrupted values (NaN, infinity, or out-of-bounds) are reset so the OS selects a safe default. Portable ZIP now always includes the `portable/` folder with a placeholder file.
- [ ] **Duplicate Keyboard Shortcut (Ctrl+B)** ([#46](https://github.com/OlaProeis/Ferrite/issues/46))  
  Remove duplicate keybinding assignment so Ctrl+B is mapped to a single, consistent action.
- [ ] **Chinese Paragraph Indentation (Rendered & Editor Views)**  
  Improve paragraph indentation handling for Chinese text in both editor and rendered modes. This pulls forward fixes originally planned for v0.2.7:
  - [#26](https://github.com/OlaProeis/Ferrite/issues/26) – Feedback on paragraph indentation, rendered mode, and multi-tab behavior  
  - [#20](https://github.com/OlaProeis/Ferrite/issues/20) – Adding paragraph indentation in Chinese editing renderings
- [ ] **Raw Mode Viewport / Text Jitter Fix**  
  Fixed periodic visual glitches during typing in Raw mode caused by spurious editor recreation and aggressive scroll clamping. Added viewport restoration, scroll clamp tolerance, and spurious sync detection.
- [ ] **General Bug Fixes** - Addressing additional issues reported post-v0.2.6 release.

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

### v0.2.7 - Performance, Features & Polish
**Focus:** Features moved from v0.2.6 to allow focus on the text editor, plus checking for updates.

#### Markdown Linking
- [ ] **Wikilinks support** ([#1](https://github.com/OlaProeis/Ferrite/issues/1)) - `[[wikilinks]]` syntax.
- [ ] **Backlinks panel** - Show documents linking to current file.

#### Editing Modes
- [ ] **Vim Mode** - Optional Vim-style modal editing (Normal / Insert / Visual modes).

#### Check for Updates
- [ ] **Check for Updates button** - Settings panel button that checks GitHub and prompts to install if update found.
- [ ] **Manual Trigger Only** - No automatic background checking (offline-first philosophy).

#### Large File Performance
- [ ] **Large file detection** - Auto-detect files > 10MB on open, show warning toast.
- [ ] **Lazy CSV row parsing** - Parse rows on-demand using byte offset index for massive CSVs.

#### Refactoring & Quality
- [ ] **Flowchart Refactoring** - Modularize the 3500+ line `flowchart.rs`.
- [ ] **App.rs Refactoring** - Split the 8000+ line `app.rs` into focused modules.
- [ ] **Window Controls** - Redesign minimize/maximize/close icons; native feel for macOS.

#### Executable Code Blocks
- [ ] **Run button on code blocks** - Add `▶ Run` button to fenced code blocks.
- [ ] **Shell / Bash execution** - Execute shell snippets via `std::process::Command`.
- [ ] **Python support** - Detect `python` / `python3` and run with system interpreter.
- [ ] **Timeout handling** - Kill long-running scripts after configurable timeout (default: 30s).
- [ ] **Security warning** - First-run dialog explaining execution risks.  
  *Security note: Code execution is opt‑in and disabled by default.*

#### Content Blocks / Callouts
- [ ] **GitHub-style callouts** - Support `> [!NOTE]`, `> [!TIP]`, `> [!WARNING]`, `> [!CAUTION]`, `> [!IMPORTANT]`.
- [ ] **Custom titles** - `> [!NOTE] Custom Title`.
- [ ] **Styled rendering** - Color-coded blocks with icons in rendered view.
- [ ] **Collapsible callouts** - `> [!NOTE]-` syntax for collapsed-by-default blocks.

---

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

#### Mermaid Authoring Improvements
- [ ] **Mermaid authoring hints** ([#4](https://github.com/OlaProeis/Ferrite/issues/4))  
  Inline hints and validation feedback when editing Mermaid diagrams to catch syntax errors and common mistakes early.

#### 1. Mermaid Crate Extraction
- [ ] **Standalone crate** - Backend-agnostic architecture with SVG, PNG, and egui outputs.
- [ ] **Public API** - `parse()`, `layout()`, `render()` pipeline.

#### 2. Markdown Enhancements
- [ ] **GitHub-style HTML** - Render `<div align>`, `<details>`, `<kbd>`, etc.

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

- [ ] **Math Rendering Engine** - Parse and render `$inline$` and `$$display$$` LaTeX math.
- [ ] **Office Document Support** - Read-only view for DOCX and XLSX files (rendered as flowing text/tables, not paginated).

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

- **Persistent Undo History:** Save undo stack to disk.
- **Plugin System:** Lua or WASM based extensions.
- **Headless Editor Library:** Extract `FerriteEditor` as a standalone Rust crate.
- **Additional Formats:** Jupyter Notebooks (`.ipynb`), EPUB, LaTeX source (`.tex`).

---

## Recently Completed ✅

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
