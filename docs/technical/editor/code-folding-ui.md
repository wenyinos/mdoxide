# Code Folding UI Changes

**Issue:** [#12](https://github.com/OlaProeis/Ferrite/issues/12) - Misleading code folding UI  
**Version:** v0.2.2  
**Status:** Implemented

## Problem

The code folding feature in Ferrite displays fold indicators (▶/▼) in the editor gutter for JSON/YAML files and Markdown documents. However, clicking these indicators does **not** collapse the content - the collapse functionality is not yet implemented. This caused user confusion as the visual indicators suggested interactive functionality that didn't exist.

Additionally, the Rendered view for JSON/YAML files had a "Raw View" button that showed non-editable raw text, which was confusing since users should use the view mode selector to switch to Raw mode for editing.

## Solution

### 1. Hide Fold Indicators by Default

Changed the default value of `folding_show_indicators` from `true` to `false`:

```rust
// src/config/settings.rs
folding_show_indicators: false,  // Hide fold indicators by default (they don't collapse yet)
```

This prevents new users from encountering the misleading UI. Power users who want to see the fold regions (even without collapse functionality) can enable it in Settings.

### 2. Updated Settings Tooltip

The tooltip for the "Show Fold Indicators" checkbox now clearly states that the feature is visual-only:

```
"Display fold indicators in the gutter (visual only - collapse not yet implemented)"
```

### 3. Removed "Raw View" Button from Tree Viewer

The "Raw View" / "Tree View" toggle button was removed from the Rendered view toolbar for JSON/YAML/TOML files. The raw view shown by this button was:
- Non-editable (read-only text display)
- Confusing - users expected it to be editable like Raw mode
- Redundant - users can use the view mode selector to switch to actual Raw mode

The "Expand All" and "Collapse All" buttons remain for the tree view navigation.

## Settings Location

**Settings > Editor > Code Folding > Show Fold Indicators**

When enabled, fold indicators appear in the gutter for:
- **Markdown files:** Headings, code blocks, list hierarchies
- **JSON/YAML/TOML files:** Indentation-based fold regions

## Future Work

When actual fold/collapse functionality is implemented:
1. Change the default back to `true`
2. Update the tooltip to remove the "visual only" disclaimer
3. Consider re-adding the Raw View button if useful

## Testing

The following tests verify the new behavior:

```rust
#[test]
fn test_folding_show_indicators_default_false() {
    let settings = Settings::default();
    assert!(!settings.folding_show_indicators);
    assert!(settings.folding_enabled);  // Detection still enabled
}

#[test]
fn test_folding_show_indicators_backward_compatibility() {
    let json = r#"{"theme": "dark"}"#;
    let settings: Settings = serde_json::from_str(json).unwrap();
    assert!(!settings.folding_show_indicators);  // New default for old configs
}

#[test]
fn test_folding_show_indicators_explicit_true() {
    let json = r#"{"folding_show_indicators": true}"#;
    let settings: Settings = serde_json::from_str(json).unwrap();
    assert!(settings.folding_show_indicators);  // Can still be enabled
}
```

## Files Changed

| File | Change |
|------|--------|
| `src/config/settings.rs` | Changed `folding_show_indicators` default to `false`, added tests |
| `src/ui/settings.rs` | Updated tooltip text |
| `src/markdown/tree_viewer.rs` | Removed Raw View toggle button |
