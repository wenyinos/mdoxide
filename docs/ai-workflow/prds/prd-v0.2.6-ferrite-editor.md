# PRD: Ferrite v0.2.6 - Custom Editor & Code Signing

## Overview

v0.2.6 addresses two critical issues:

1. **Memory Crisis**: egui's TextEdit creates ~500MB-1GB galleys for 4MB files, making large files unusable
2. **Trust Crisis**: Windows Defender false positives blocking users from running Ferrite

**Goal**: Fully replace TextEdit with a custom virtual-scrolling editor (FerriteEditor), and sign releases to eliminate false positives.

### Release Requirements

**All three phases are required for v0.2.6 release:**
- Phase 1: Foundation (basic editor, horizontal scroll)
- Phase 2: Feature parity (word wrap, selection, syntax)
- Phase 3: CJK/IME support - **RELEASE BLOCKER** (Chinese users)

There is **no fallback** to TextEdit in the shipped release. The feature flag is for development only and will be removed after Phase 3 validation.

---

## P0 - Critical: Custom Text Editor Widget

### Problem Statement

egui's `TextEdit` widget stores layout information (Galley) for **every character** in the document. A 4MB file creates 500MB-1GB of memory usage. This is unfixable without replacing the underlying widget.

**Current**: 4MB file → ~500MB-1GB RAM  
**Target**: 4MB file → ~10-20MB RAM (50x improvement)

### Scope Clarification

**What's being replaced:**
- **Raw view** - The plain text editor (`EditorWidget` → `TextEdit::multiline`)
- **Split view (left pane)** - Same `EditorWidget`

**NOT being replaced:**
- **Rendered view** - Uses `MarkdownEditor` which renders parsed AST as widgets, not TextEdit
- **Split view (right pane)** - Same `MarkdownEditor`

The Rendered/WYSIWYG view has a completely different architecture and doesn't suffer from this memory problem.

### Technical Approach

Replace `TextEdit` in `src/editor/widget.rs` with a custom `FerriteEditor` widget that:

1. Uses **ropey::Rope** for O(log n) text operations
2. Implements **virtual scrolling** - only renders visible lines + small buffer
3. Caches **per-line galleys** instead of full-document galley
4. Maintains **feature flag** for fallback to TextEdit during development

### Architecture

```
FerriteEditor
├── TextBuffer (ropey::Rope)      # Efficient text storage
├── ViewState                      # Scroll position, visible line range
├── LineCache                      # Cached galleys for visible lines only
└── InputHandler                   # Keyboard/mouse/IME events
```

### Data Model & Synchronization

**Primary storage**: `ropey::Rope` inside `TextBuffer`

**Sync strategy**: 
- Rope is the source of truth during editing
- `Tab.content: String` synced via **debounced sync** (~500ms delay)
- Sync triggers: typing pause, save, tab switch, undo checkpoint
- Avoids per-keystroke string allocation for large files

**Live Preview (Split View)**:
- Debounced sync feeds the Markdown renderer (~500ms latency acceptable)
- For files >5MB: Live preview **disabled** (too expensive to sync frequently)
- User sees "Large file - preview disabled" message

**Undo/Redo**: Rope-native operation-based undo (not full snapshots)
- Store `EditOperation` enum: `Insert { pos, text }`, `Delete { pos, text }`
- Group operations by typing sessions (debounce ~500ms)
- Undo replays inverse operations
- More memory efficient than storing full content snapshots

### Requirements

#### Phase 1: Foundation (No Wrap, Horizontal Scroll)

**Key simplification**: Phase 1 uses **horizontal scrolling** (no word wrap). This makes virtual scrolling math trivial: `total_height = line_count × fixed_line_height`. Word wrap is added in Phase 2.

1. **TextBuffer Module** (`src/editor/buffer.rs`)
   - Wrap ropey::Rope with editing operations
   - Methods: `insert()`, `remove()`, `line()`, `line_count()`, `line_to_char()`
   - `sync_to_string()` for saving and compatibility
   - `from_string()` for loading
   - Debounced sync timer integration

2. **EditHistory Module** (`src/editor/history.rs`)
   - `EditOperation` enum for insert/delete operations
   - `EditHistory` struct with undo/redo stacks
   - Operation grouping with 500ms debounce
   - `undo()` / `redo()` apply inverse operations to rope

3. **ViewState Module** (`src/editor/view.rs`)
   - Track first visible line, visible line count
   - Calculate render range with overscan (visible + 5 lines above/below)
   - Scroll-to-line conversion for navigation
   - **No wrap**: `visible_lines = viewport_height / line_height`

4. **LineCache Module** (`src/editor/line_cache.rs`)
   - Cache per-line galleys with content hash keys
   - LRU eviction, max ~200 entries
   - Invalidate on content change
   - Single-line galleys (no wrap)

5. **Basic Rendering**
   - Render only visible lines using egui::Painter
   - **Horizontal scroll** for long lines (no wrap)
   - Line numbers gutter (reuse existing line_numbers.rs)
   - Single cursor display and movement
   - Mouse click for cursor placement
   - Vertical scroll via mouse wheel

6. **Basic Input**
   - Character insertion (typing)
   - Backspace/Delete
   - Arrow key navigation
   - Home/End (line), Ctrl+Home/End (document)
   - Page Up/Down
   - Enter for newlines
   - Tab insertion

7. **Integration**
   - Feature flag in EditorWidget (dev-only, not shipped)
   - Preserve Tab state compatibility (cursor position, scroll offset)
   - Debounced sync to Tab.content for preview/save

**Success Criteria (Phase 1)**:
- Opens 4MB file with <50MB memory
- Basic typing, cursor movement, scrolling works
- Horizontal scroll for long lines (wrap OFF)
- Undo/redo works with rope-native operations
- Live preview works via debounced sync (disabled for >5MB files)

#### Phase 2: Word Wrap & Feature Parity

**Key addition**: Word wrap support. This is complex because we need to track visual rows vs logical lines.

1. **Word Wrap Support**
   - Calculate wrapped line heights per logical line
   - ViewState tracks visual rows, not just logical lines
   - Line cache stores wrapped galleys
   - Cursor navigation respects wrapped lines (visual up/down)
   - Scrollbar height calculation with wrapped content
   - Max line width setting integration
   - Zen mode centering support

2. **Selection & Clipboard**
   - Shift+Arrow selection (wrap-aware)
   - Ctrl+A select all
   - Ctrl+C/X/V clipboard operations
   - Click-drag selection
   - Double-click word select, triple-click line select

3. **Syntax Highlighting**
   - Port existing layouter logic to per-line highlighting
   - Only highlight visible lines
   - Cache highlighted galleys
   - Theme integration

4. **Search Highlights**
   - Port search match highlighting
   - Current match distinct color
   - Scroll-to-match positioning
   - Transient highlight for search-in-files navigation

5. **Bracket Matching**
   - Port existing bracket matching logic
   - Highlight matching pairs

6. **Find & Replace Integration**
   - Selection-based find
   - Replace at cursor
   - Replace all

**Success Criteria (Phase 2)**:
- Word wrap works correctly with max line width / Zen mode
- All existing EditorWidget features work
- Selection works correctly on wrapped lines
- No user-visible regression vs TextEdit
- Memory target maintained

#### Phase 3: IME/CJK Support & Advanced Features (RELEASE BLOCKER)

**CRITICAL**: IME support is required for Chinese users. v0.2.6 will NOT release until this phase is complete.

1. **IME (Input Method Editor) Support** - **RELEASE BLOCKER**
   - Handle `Event::Ime` events from egui
   - Composition window display at cursor position
   - Preedit text rendering (underlined composition)
   - Commit handling for finalized text
   - Test with Chinese (Pinyin), Japanese (Romaji), Korean (Hangul) input
   - CJK font rendering verification

2. **Multi-cursor Text Operations**
   - Apply edits at all cursor positions
   - Offset adjustment after edits
   - Visual cursor rendering for secondary cursors

3. **Code Folding with Text Hiding**
   - Fold state integration with line rendering
   - Skip rendering folded regions
   - Cursor navigation respects folds

4. **Scroll Sync Improvements**
   - Expose precise line-to-pixel mapping
   - Bidirectional sync with rendered view

5. **TextEdit Fallback Removal**
   - Remove feature flag from codebase
   - Clean up all TextEdit-related code paths
   - FerriteEditor becomes the only editor

**Success Criteria (Phase 3)**:
- **IME works for Chinese, Japanese, Korean input**
- Multi-cursor actually works for typing
- Folded regions hidden (not just indicators)
- Scroll sync is pixel-perfect
- No TextEdit code remains in shipped release

### Files to Create/Modify

**New files:**
- `src/editor/ferrite_editor.rs` - Main custom editor widget
- `src/editor/buffer.rs` - TextBuffer (ropey wrapper)
- `src/editor/history.rs` - EditHistory (rope-native undo/redo)
- `src/editor/view.rs` - ViewState for virtual scrolling
- `src/editor/line_cache.rs` - Galley caching
- `src/editor/input.rs` - Input handling

**Modified files:**
- `src/editor/mod.rs` - Export new modules
- `src/editor/widget.rs` - Add feature flag for FerriteEditor
- `Cargo.toml` - Add ropey dependency

### Dependencies

```toml
ropey = "1.6"
```

---

## P0 - Critical: Code Signing

### Problem Statement

Windows Defender's ML-based detection flagged Ferrite as `Trojan:Win32/Bearfoos.B!ml` (false positive). This blocks users from running the application and damages trust.

### Requirements

1. **SignPath Setup**
   - Create SignPath.io account (free tier for open source)
   - Configure signing certificate
   - Document setup for future maintainers

2. **CI/CD Integration**
   - Integrate SignPath into `.github/workflows/release.yml`
   - Sign Windows .exe during release builds
   - Verify signatures post-signing

3. **EV Certificate Research**
   - Document Extended Validation certificate requirements
   - Cost/benefit analysis for SmartScreen reputation
   - Recommendation for future (may defer actual EV cert)

**Success Criteria**:
- Windows releases are signed
- No more Defender false positives
- SmartScreen shows verified or no warning

### Files to Modify

- `.github/workflows/release.yml` - Add signing step
- Create `docs/technical/platform/code-signing.md` - Setup documentation

---

## Out of Scope for v0.2.6

Deferred to future releases:
- Large CSV lazy loading (v0.2.7)
- Vim mode (v0.3.0+)
- Executable code blocks (v0.3.0+)
- Content blocks/callouts (v0.3.0+)
- Additional format support (v0.3.0+)
- Memory-mapped file I/O (v0.3.0+)

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Word wrap complexity | Deferred to Phase 2; Phase 1 uses horizontal scroll for simple math |
| IME failures | Test early, egui has Event::Ime support; this is a **release blocker** |
| Feature parity gaps | Feature flag during dev; removed after Phase 3 validation |
| Rope-native undo complexity | Start simple (insert/delete ops), add grouping incrementally |
| SignPath delays | Can release unsigned while waiting for approval |
| Very long lines (>10K chars) | Horizontal scroll in Phase 1; wrap handles in Phase 2 |
| Live preview latency | Debounced sync (~500ms); disabled for >5MB files |
| CJK font rendering | Already works in current Ferrite; verify with new editor |

---

## Testing Strategy

### Memory Testing
- Benchmark: 1KB, 100KB, 1MB, 4MB, 10MB files
- Target: <2x file size in memory
- Verify debounced sync doesn't cause memory spikes

### Feature Testing
- All existing keyboard shortcuts work
- Selection, clipboard, search highlights
- Line numbers, syntax highlighting
- Word wrap with max line width / Zen mode

### IME Testing (RELEASE BLOCKER)
- Chinese input (Pinyin on Windows/macOS)
- Japanese input (Romaji → Hiragana → Kanji)
- Korean input (Hangul)
- Composition window positioning
- Mixed CJK and Latin text

### Cross-Platform
- Windows, macOS, Linux builds
- Code signing verified on Windows
- IME tested on all platforms

---

## Timeline Estimate

With AI-assisted development (all phases required for release):

- **Phase 1**: 4-6 focused sessions (~1 week)
  - Buffer + History: 2 sessions
  - View + LineCache: 1 session
  - Rendering (no wrap): 1-2 sessions
  - Input + Integration: 1 session
  
- **Phase 2**: 4-6 focused sessions (~1 week)
  - Word wrap implementation: 2-3 sessions
  - Selection (wrap-aware): 1-2 sessions
  - Syntax highlighting + search: 1-2 sessions
  
- **Phase 3**: 3-5 focused sessions (~1 week) - **REQUIRED**
  - IME/CJK support: 2-3 sessions - **RELEASE BLOCKER**
  - Multi-cursor, folding: 1-2 sessions
  - Cleanup & TextEdit removal: 1 session

- **Code Signing**: 1-2 sessions (parallel with Phase 1)
- **Testing & Polish**: 2-3 sessions

**Total**: ~3-4 weeks of focused work

**Release gate**: v0.2.6 ships only after Phase 3 IME validation passes

---

## References

- [egui Painter docs](https://docs.rs/egui/latest/egui/struct.Painter.html)
- [egui LayoutJob docs](https://docs.rs/egui/latest/egui/text/struct.LayoutJob.html)
- [ropey docs](https://docs.rs/ropey/latest/ropey/)
- [Xi-editor rope science](https://xi-editor.io/docs/rope_science.html) - CRDT-inspired undo model
- [Helix editor](https://github.com/helix-editor/helix) - Reference Rust editor using ropey
- [SignPath.io](https://signpath.io/)
- [Existing plan](docs/technical/planning/custom-editor-widget-plan.md) - Original v0.3.0 plan (now accelerated to v0.2.6)
