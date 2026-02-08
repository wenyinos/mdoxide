# PRD: Fix Rendered Mode List Editing Index Bug

## Version Target: 0.2.0

## Priority: CRITICAL

## Problem Statement

When editing list items in **rendered (WYSIWYG) mode**, the editor selects and modifies the wrong content element. This is a critical usability bug that makes rendered mode editing unreliable.

### Observed Behavior

1. **First list item bug**: When clicking to edit the first item in a list, the editor instead selects the header/element ABOVE the list.

2. **Other list items bug**: When clicking other list items (2nd, 3rd, etc.), the visual selection appears correct, BUT:
   - The actual editing happens on a DIFFERENT item (wrong index)
   - Typed content appears in the previous item or another list item
   - There's an off-by-one or indexing mismatch issue

### Expected Behavior

- Clicking a list item should select THAT specific item
- Editing should modify ONLY the clicked item
- Changes should appear exactly where the user clicked

### Reproduction Steps

1. Open a markdown file with headings and lists in Ferrite
2. Switch to **Rendered** view mode
3. Click on the FIRST item in a list → Observe it selects the header above
4. Click on ANY OTHER list item → Observe the edit appears in a different item

### Test File

Use the test file pattern with structure:

```markdown
## Header Above List

- First list item
- Second list item  
- Third list item

## Another Header

- Item A
- Item B
```

## Technical Context

### Previous Related Fix (December 2025)

A "click-to-edit" fix was implemented in `src/markdown/editor.rs` for formatted content:
- Added `FormattedItemEditState` struct
- Functions: `render_list_item()`, `render_list_item_with_structural_keys()`
- The editing mechanism WORKS, but the INDEX/SELECTION is wrong

### Root Cause Hypothesis

The bug is likely in one of these areas:
1. **Index calculation** when mapping click position to markdown AST node
2. **State tracking** - the edit state HashMap keys don't match actual node indices
3. **AST traversal** - off-by-one error when iterating nodes
4. **ID generation** - structural keys/IDs don't correctly identify nodes

### Key Files to Investigate

| File | Relevance |
|------|-----------|
| `src/markdown/editor.rs` | Main editor - contains list rendering & edit state |
| `src/markdown/parser.rs` | AST parsing - node indexing |
| `src/markdown/widgets.rs` | Widget rendering helpers |
| `src/state.rs` | Tab state - edit state storage |

### Key Data Structures

```rust
// Edit state tracking (suspected issue area)
struct FormattedItemEditState {
    editing: bool,
    edit_text: String,
    needs_focus: bool,
}

// HashMap<String, FormattedItemEditState> - key generation may be wrong
```

## Solution Requirements

### Must Have

1. **Correct index mapping**: Clicked list item index must match the edited item
2. **First item fix**: First list item must NOT select header above
3. **Consistent behavior**: All list items (1st through Nth) must behave identically
4. **No regressions**: Existing formatting edit functionality must continue working

### Testing Criteria

- [ ] Click first list item → Edits first item (not header)
- [ ] Click second list item → Edits second item exactly
- [ ] Click last list item → Edits last item exactly
- [ ] Nested lists work correctly
- [ ] Mixed formatted content (bold, italic, code) still editable
- [ ] Raw mode editing unaffected
- [ ] Undo/redo works correctly with fixed editing

## Investigation Approach

1. **Add debug logging** to trace:
   - Click position → Calculated index
   - Node ID/key being generated
   - Which item the edit state maps to

2. **Review index generation** in:
   - `render_list_item()` and variants
   - Any `enumerate()` or manual indexing
   - HashMap key generation for edit state

3. **Compare with raw mode**: Raw mode works correctly - compare how it handles indexing

4. **Test with minimal file**: Create simplest reproduction case (heading + 3-item list)

## Out of Scope (v0.2.0)

- True WYSIWYG editing (keeping click-to-edit approach)
- Performance optimizations
- New features
- Other bugs from roadmap (will be separate tasks)

## Success Metric

User can click ANY list item in rendered mode and edit it in-place with changes appearing exactly where clicked, with 100% accuracy.
