# New File Save Prompt Logic

## Overview

Improved the save prompt logic to skip unnecessary prompts for unmodified untitled files. Empty new tabs can now be closed silently without a "Save changes?" dialog, while still protecting user content that has been typed.

## Key Files

- `src/state.rs` - Tab struct methods, AppState close/quit logic

## Implementation Details

### New Methods Added to Tab

```rust
/// Check if this is a new/untitled file (not yet saved to disk)
pub fn is_new_file(&self) -> bool {
    self.path.is_none()
}

/// Check if this is an unmodified empty untitled file
pub fn is_empty_untitled(&self) -> bool {
    self.is_new_file() && self.content.is_empty() && self.original_content.is_empty()
}

/// Determine if we should prompt to save before closing
pub fn should_prompt_to_save(&self) -> bool {
    // Don't prompt for unmodified files
    if !self.is_modified() {
        return false;
    }
    // Don't prompt for empty untitled files
    if self.is_new_file() && self.content.is_empty() {
        return false;
    }
    true
}
```

### Updated Methods

- `close_tab()` - Now uses `should_prompt_to_save()` instead of `is_modified()`
- `has_unsaved_changes()` - Now uses `should_prompt_to_save()` for consistency

## Behavior Matrix

| Scenario | Save Prompt? |
|----------|-------------|
| New file, unmodified (empty) | No |
| New file, with content | Yes |
| New file, typed then deleted | No |
| Existing file, unmodified | No |
| Existing file, modified | Yes |
| Quit with only empty untitled tabs | No |
| Quit with at least one modified tab | Yes |

## Tests Added

### Tab-Level Tests
- `test_tab_is_new_file` - Verifies new file detection
- `test_tab_is_empty_untitled` - Verifies empty untitled file detection
- `test_tab_should_prompt_to_save` - Comprehensive test of all scenarios

### AppState-Level Tests
- `test_appstate_close_new_unmodified_tab_no_prompt`
- `test_appstate_close_new_modified_tab_prompts`
- `test_appstate_close_empty_typed_deleted_tab_no_prompt`
- `test_appstate_quit_with_mixed_tabs`
- `test_appstate_quit_with_only_empty_untitled_tabs`

## Usage

No changes to user workflow required. The behavior is now:

1. **Create new tab** → empty tab appears
2. **Close immediately** → closes silently (no prompt)
3. **Type content, then close** → "Save changes?" prompt appears
4. **Type content, delete all, then close** → closes silently (back to empty state)

## Edge Cases Handled

- **Content typed then deleted**: Returns to empty state, no prompt needed
- **Multiple empty tabs on quit**: All close silently
- **Mix of empty and modified tabs**: Only modified tabs trigger prompts
- **Existing empty file from disk**: Treated as saved file, not as "new"
