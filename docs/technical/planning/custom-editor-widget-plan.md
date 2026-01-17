# Custom Editor Widget Plan (v0.3.0)

> **Status:** Planning phase - collecting v0.2.0 feedback before implementation  
> **Target:** v0.3.0  
> **Created:** 2025-01-10

## Executive Summary

Replace egui's `TextEdit` widget with a custom `FerriteEditor` widget built on egui's drawing primitives. This unblocks multiple v0.3.0 features that are currently impossible due to egui's single-cursor, opaque text handling design.

---

## Problem Statement

### Current Limitations

egui's `TextEdit` is fundamentally designed for simple, single-cursor text input. It:

- Handles all keyboard input internally with no hooks for interception
- Provides no API for text hiding (code folding)
- Has opaque scroll handling that prevents precise sync
- Offers no virtual/ghost text insertion capability
- Groups all edits internally, preventing fine-grained undo

### Features Blocked by egui TextEdit

| Feature | Why Blocked |
|---------|-------------|
| **Full multi-cursor editing** | TextEdit handles input internally; can't replicate edits to multiple positions |
| **Code folding (text hiding)** | No way to show subset of text while maintaining cursor/position mapping |
| **Scroll sync perfection** | No access to internal scroll state or line-to-pixel mapping |
| **Split view scroll sync** | Same scroll opacity issues |
| **Virtual text (ghost text, inline widgets)** | No API for display-only text insertion |
| **Operation-based undo** | No access to individual edit operations |

### Current Workarounds in `widget.rs`

We're already working around egui extensively:
- Custom overlay rendering for additional cursors (lines 813-855)
- Manual scroll management via `pending_scroll_offset` 
- Custom syntax highlighting via `layouter` closure
- Bracket matching as post-render overlays
- Transient search highlights as overlays

The workarounds are becoming unmaintainable and can't achieve the required functionality.

---

## Solution: Custom FerriteEditor Widget

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    FerriteEditor Widget                      │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │   Buffer    │  │   Cursors   │  │     Rendering       │  │
│  │  (ropey)    │  │ (MultiCur)  │  │  (egui Painter)     │  │
│  │             │  │             │  │                     │  │
│  │ • Rope stor │  │ • Positions │  │ • LayoutJob         │  │
│  │ • Line idx  │  │ • Selections│  │ • Syntax colors     │  │
│  │ • Edit ops  │  │ • Primary   │  │ • Cursor rendering  │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
│                                                              │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │   Folding   │  │   History   │  │   Input Handler     │  │
│  │             │  │             │  │                     │  │
│  │ • Regions   │  │ • Edit ops  │  │ • Keyboard events   │  │
│  │ • Collapsed │  │ • Grouping  │  │ • Mouse events      │  │
│  │ • Virtual   │  │ • Undo/redo │  │ • IME handling      │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### Core Components

#### 1. Text Buffer (`ropey`)

```rust
use ropey::Rope;

pub struct TextBuffer {
    rope: Rope,
    // Line metadata cache (fold state, syntax tokens, etc.)
    line_cache: Vec<LineMetadata>,
}
```

**Why ropey:**
- O(log n) insertions/deletions (critical for multi-cursor)
- Efficient line indexing
- Memory-efficient for large files
- Battle-tested in Helix, Lapce, and other Rust editors

#### 2. Multi-Cursor System (exists, needs enhancement)

```rust
pub struct MultiCursor {
    selections: Vec<Selection>,
    primary_index: usize,
}

impl MultiCursor {
    /// Apply an edit at all cursor positions, adjusting offsets as we go
    pub fn apply_edit(&mut self, buffer: &mut TextBuffer, edit: EditOperation) {
        // Sort cursors by position (descending) to avoid offset invalidation
        // Apply edit at each position
        // Adjust subsequent cursor positions based on text length change
    }
}
```

#### 3. Fold State

```rust
pub struct FoldState {
    /// Collapsed regions (start_line, end_line)
    collapsed: Vec<(usize, usize)>,
}

impl FoldState {
    /// Map display line to buffer line
    pub fn display_to_buffer_line(&self, display_line: usize) -> usize;
    
    /// Map buffer line to display line (None if folded)
    pub fn buffer_to_display_line(&self, buffer_line: usize) -> Option<usize>;
    
    /// Get total display line count
    pub fn display_line_count(&self, total_buffer_lines: usize) -> usize;
}
```

#### 4. Edit History (operation-based)

```rust
pub enum EditOperation {
    Insert { position: usize, text: String },
    Delete { position: usize, text: String },
    // Future: Replace, MultiInsert, etc.
}

pub struct EditHistory {
    undo_stack: Vec<Vec<EditOperation>>,  // Grouped operations
    redo_stack: Vec<Vec<EditOperation>>,
    current_group: Vec<EditOperation>,
}
```

#### 5. Input Handler

```rust
impl FerriteEditor {
    fn handle_input(&mut self, ui: &mut Ui) {
        // Keyboard events
        ui.input(|i| {
            for event in &i.events {
                match event {
                    Event::Text(text) => self.insert_text(text),
                    Event::Key { key, pressed, modifiers } => {
                        self.handle_key(*key, *pressed, modifiers);
                    }
                    // ...
                }
            }
        });
        
        // Mouse events for cursor placement, selection, etc.
    }
}
```

#### 6. Rendering

```rust
impl FerriteEditor {
    fn render(&self, ui: &mut Ui) {
        // 1. Build LayoutJob with syntax highlighting
        let job = self.build_layout_job();
        
        // 2. Layout text
        let galley = ui.fonts(|f| f.layout_job(job));
        
        // 3. Paint text
        let painter = ui.painter();
        painter.galley(pos, galley);
        
        // 4. Paint cursors (all of them!)
        for cursor in &self.cursors {
            self.paint_cursor(painter, cursor);
        }
        
        // 5. Paint selections
        for selection in &self.selections {
            self.paint_selection(painter, selection);
        }
        
        // 6. Paint fold indicators, brackets, etc.
    }
}
```

---

## Implementation Phases

### Phase 1: Foundation (2-3 weeks)

**Goal:** Basic editable text widget with single cursor

- [ ] Create `FerriteEditor` struct with `ropey::Rope` buffer
- [ ] Implement text rendering via egui `LayoutJob`
- [ ] Handle basic keyboard input (characters, backspace, delete, enter)
- [ ] Implement single cursor movement (arrows, home/end, page up/down)
- [ ] Basic mouse click for cursor placement
- [ ] Click-drag selection
- [ ] Clipboard operations (Ctrl+C/X/V)
- [ ] Basic scrolling

**Deliverable:** Editor that can open, edit, and save a file

### Phase 2: Multi-Cursor (2-3 weeks)

**Goal:** Full multi-cursor text operations

- [ ] Port existing `MultiCursor` struct
- [ ] Implement `apply_edit` with offset adjustment
- [ ] Ctrl+D: Select next occurrence (already works, integrate)
- [ ] Ctrl+Click: Add cursor
- [ ] Alt+Click+Drag: Column/box selection
- [ ] Multi-cursor typing
- [ ] Multi-cursor deletion
- [ ] Escape: Collapse to single cursor

**Deliverable:** Full multi-cursor editing like VS Code

### Phase 3: Code Folding (2 weeks)

**Goal:** Collapsible regions with text hiding

- [ ] Implement `FoldState` with display/buffer line mapping
- [ ] Integrate fold detection from existing `folding.rs`
- [ ] Render fold indicators in gutter (already done, port)
- [ ] Click to toggle fold
- [ ] Hide folded text in rendering
- [ ] Cursor navigation respects folds (skip over collapsed regions)
- [ ] Ctrl+Shift+[ / ] for fold/unfold all

**Deliverable:** Working code folding with text hiding

### Phase 4: Scroll Sync & Polish (3-4 weeks)

**Goal:** Perfect scroll sync, edge cases, production quality

- [ ] Expose precise line-to-pixel mapping for scroll sync
- [ ] Implement bidirectional scroll sync with Rendered view
- [ ] Split view scroll synchronization
- [ ] IME support (composition window, candidate window)
- [ ] Word wrap handling
- [ ] Long line performance optimization
- [ ] Undo/redo grouping (e.g., typing session = one undo)
- [ ] Selection highlight colors (theme integration)
- [ ] Bracket matching integration
- [ ] Search highlight integration
- [ ] Minimap integration

**Deliverable:** Production-ready editor widget

### Phase 5: Integration & Migration (1-2 weeks)

**Goal:** Replace TextEdit throughout the app

- [ ] Replace in Raw view
- [ ] Replace in Split view raw pane
- [ ] Update WYSIWYG editor to use new cursor system
- [ ] Update find/replace integration
- [ ] Update all keyboard shortcuts
- [ ] Performance testing with large files
- [ ] Memory usage testing

**Deliverable:** v0.3.0 release candidate

---

## Technical Considerations

### Performance

- **Large files:** ropey handles 100MB+ files efficiently
- **Syntax highlighting:** Keep existing cache strategy, re-highlight only visible lines
- **Rendering:** Only layout visible portion of document
- **Multi-cursor:** Sort cursors descending to avoid offset recalculation cascade

### IME Support

IME (Input Method Editor) for CJK languages is complex:
- Need to handle `Event::Ime` events in egui
- Show composition text at cursor position
- Handle commit vs. composition states

Consider deferring full IME support to Phase 5 or post-v0.3.0.

### Compatibility

- Maintain same keyboard shortcuts as current editor
- Preserve undo history format if possible (for session restore)
- Keep theme color system integration

### Testing Strategy

- Unit tests for `FoldState` line mapping
- Unit tests for `MultiCursor` offset adjustment
- Integration tests for edit operations
- Manual testing matrix for keyboard shortcuts
- Performance benchmarks (file sizes: 1KB, 100KB, 1MB, 10MB)

---

## Dependencies

### Required Crates

```toml
# Already in Cargo.toml
# egui = "0.28" - for rendering primitives

# New dependency
ropey = "1.6"  # Rope-based text buffer
```

### Optional Future Additions

```toml
# If needed for advanced features
unicode-segmentation = "1.10"  # Grapheme cluster handling
unicode-width = "0.1"          # Display width calculation
```

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| IME complexity | High | Defer to Phase 5, basic Latin first |
| Performance regression | Medium | Benchmark early, optimize visible-only rendering |
| Edge cases in text editing | Medium | Port existing test cases, expand coverage |
| Word wrap complexity | Medium | Start with no-wrap, add wrap in Phase 4 |
| Scope creep | High | Strict phase boundaries, defer nice-to-haves |

---

## Success Criteria

### v0.3.0 Release Requirements

- [ ] Multi-cursor editing works for all text operations
- [ ] Code folding hides collapsed regions
- [ ] Scroll sync is noticeably improved
- [ ] No regression in basic editing performance
- [ ] All existing keyboard shortcuts still work

### Nice-to-Have (can defer to v0.4.0)

- [ ] Full IME support
- [ ] Column selection
- [ ] Virtual text / ghost text
- [ ] Split view preview editing

---

## References

- [Ropey documentation](https://docs.rs/ropey)
- [egui custom widgets guide](https://docs.rs/egui/latest/egui/struct.Ui.html)
- [Helix editor architecture](https://github.com/helix-editor/helix)
- [Xi-editor rope design](https://xi-editor.io/docs/rope_science.html)
- [VS Code multi-cursor implementation](https://github.com/microsoft/vscode)

---

## Changelog

| Date | Change |
|------|--------|
| 2025-01-10 | Initial planning document created |
