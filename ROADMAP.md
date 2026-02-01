# Ferrite Roadmap

## Next Up (Immediate Focus) 

### v0.2.6.1 (Planned) - Patch & Stability
**Focus:** Critical bug fixes, code signing, and stability improvements following the major editor rewrite.

- [ ] **Code Signing** - Windows artifacts (exe, MSI) will be code signed via SignPath.io (pending approval).
- [ ] **Japanese Crash Fix** - Fix critical crash affecting Japanese users.
- [ ] **General Bug Fixes** - Addressing issues reported post-v0.2.6 release.

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

---

### v0.2.8 - UI & Accessibility

#### Traditional Menu Bar ([#59](https://github.com/OlaProeis/Ferrite/issues/59))
- [ ] **Alt-key menu access** - Traditional File/Edit/View menus toggled via Alt key (VS Code style).
- [ ] **Accessibility** - Full keyboard navigation for all menu items.

---

### v0.3.0 - Mermaid Crate + Markdown Enhancements
**Focus:** Extracting the Mermaid renderer as a standalone crate and improving markdown rendering.

#### 1. Mermaid Crate Extraction
- [ ] **Standalone crate** - Backend-agnostic architecture with SVG, PNG, and egui outputs.
- [ ] **Public API** - `parse()`, `layout()`, `render()` pipeline.

#### 2. Markdown Enhancements
- [ ] **Wikilinks support** ([#1](https://github.com/OlaProeis/Ferrite/issues/1)) - `[[wikilinks]]` syntax.
- [ ] **Backlinks panel** - Show documents linking to current file.
- [ ] **GitHub-style HTML** - Render `<div align>`, `<details>`, `<kbd>`, etc.

---

### v0.4.0 - Math Support & Document Formats
**Focus:** Native LaTeX math rendering and "page-less" Office document viewing.

- [ ] **Math Rendering Engine** - Parse and render `$inline$` and `$$display$$` LaTeX math.
- [ ] **Office Document Support** - Read-only view for DOCX and XLSX files (rendered as flowing text/tables, not paginated).

---

## Future & Long-Term Vision 

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