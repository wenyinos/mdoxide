# UI Ribbon Redesign - Design C

**Implementation Date:** January 9, 2026  
**Status:** Complete  
**Version:** v0.2.2 (candidate)

## Overview

This document describes the "Design C" ribbon redesign that streamlines the Ferrite UI by:

1. Moving view controls to the title bar
2. Introducing dropdown menus for less-frequently used actions
3. Reducing ribbon button count from ~25 to ~15
4. Improving discoverability through better organization

## Changes Summary

### Title Bar Integration

New controls added to the title bar (right side, before window buttons):

| Control | Position | Behavior |
|---------|----------|----------|
| **Auto-Save Indicator (⏱/⏸)** | After filename | Click to toggle; green=ON, muted=OFF |
| **View Mode Segment** | Before Zen Mode | 3-button toggle: Raw \| Split \| Rendered |
| **Zen Mode (🧘/🔲)** | Before Settings | Toggle button |
| **Settings (⚙)** | Before window buttons | Opens settings panel |

### Ribbon Streamlining

#### Replaced with Dropdowns

1. **Save Dropdown (💾▾)**
   - Save (Ctrl+S)
   - Save As (Ctrl+Shift+S)

2. **Export Dropdown** (Markdown only)
   - Export as HTML (Ctrl+Shift+E)
   - Copy as HTML
   - Export as PDF (disabled, future)

#### Removed from Ribbon (Moved to Title Bar)

- View Mode toggle (📝/||/👁)
- Auto-Save toggle (⏱/⏸)
- Zen Mode toggle (🧘)
- Settings button (⚙)

#### Removed from Ribbon (Moved to Settings Panel)

- Line Numbers toggle (🔢)
- Sync Scroll toggle (🔗)
- Theme Cycle button (🎨)

### File Changes

| File | Changes |
|------|---------|
| `src/ui/view_segment.rs` | **NEW**: ViewModeSegment and TitleBarButton widgets |
| `src/ui/mod.rs` | Export new widgets |
| `src/ui/ribbon.rs` | Replaced 25+ buttons with ~15, added dropdowns |
| `src/app.rs` | Integrated title bar controls, handle actions |

## New Components

### ViewModeSegment Widget

A three-button segmented control for switching view modes:

```
┌─────┬────┬─────┐
│ 📝  │ || │ 👁  │
└─────┴────┴─────┘
  Raw  Split Rendered
```

**Features:**
- File-type aware: Split disabled for non-markdown structured files
- Visual feedback: Selected segment highlighted
- Tooltips with keyboard shortcut info

**Usage:**
```rust
let segment = ViewModeSegment::new();
if let Some(action) = segment.show(ui, current_mode, file_type, is_dark) {
    match action {
        ViewSegmentAction::SetRaw => { /* ... */ }
        ViewSegmentAction::SetSplit => { /* ... */ }
        ViewSegmentAction::SetRendered => { /* ... */ }
    }
}
```

### TitleBarButton Widget

Compact buttons styled for the title bar area:

```rust
// Standard toggle button
if TitleBarButton::show(ui, "🧘", "Zen Mode (F11)", is_active, is_dark).clicked() {
    // Handle click
}

// Auto-save indicator with special styling
if TitleBarButton::show_auto_save(ui, auto_save_enabled, is_dark).clicked() {
    // Toggle auto-save
}
```

## Keyboard Shortcuts

All existing keyboard shortcuts continue to work:

| Shortcut | Action |
|----------|--------|
| Ctrl+S | Save |
| Ctrl+Shift+S | Save As |
| Ctrl+E | Cycle View Mode (Raw → Split → Rendered) |
| Ctrl+Shift+E | Export as HTML |
| Ctrl+, | Open Settings |
| F11 | Toggle Zen Mode |

## Visual Comparison

### Before (Old Ribbon)
```
┌──────────────────────────────────────────────────────────────────────────────┐
│ ◀│File 📄📂📁🔎⚡💾📥⏱│Edit ↩↪│Format B I <>... │View 📝🔢🔗🧘│Tools│Export│⚙🎨│
└──────────────────────────────────────────────────────────────────────────────┘
  25+ buttons, cluttered, view controls buried in ribbon
```

### After (Design C)
```
┌──────────────────────────────────────────────────────────────────────────────┐
│ 📝 Document.md ⏱        │              │ [📝][||][👁] │ 🧘 │ ⚙ │  _ □ ×   │
├──────────────────────────────────────────────────────────────────────────────┤
│ ◀│ 📄 📂 📁 💾▾ │ ↩ ↪ │ B I <> [~] H▾ - 1. > {} │ 🔍 📑 │ Export ▾ │      │
└──────────────────────────────────────────────────────────────────────────────┘
  ~15 buttons, view controls in title bar, dropdowns for grouped actions
```

## Testing

### Manual Test Checklist

- [ ] Title bar controls visible and responsive
- [ ] View segment shows correct mode
- [ ] View segment switches modes correctly
- [ ] View segment disabled segments for non-markdown (no Split)
- [ ] Auto-save indicator toggles and shows correct state
- [ ] Settings button opens settings panel
- [ ] Zen mode toggle works
- [ ] Save dropdown shows both options
- [ ] Export dropdown shows (Markdown only)
- [ ] All keyboard shortcuts still work
- [ ] Ribbon collapse still works

### Automated Tests

Located in `src/ui/view_segment.rs`:
- `test_view_mode_segment_new`
- `test_view_segment_action_equality`
- `test_view_mode_segment_default`

## Future Considerations

1. **PDF Export**: Export dropdown includes disabled "Export as PDF" placeholder
2. **Custom Themes**: Settings panel theme selector could be enhanced
3. **Ribbon Customization**: User-configurable ribbon groups/buttons
4. **Touch Support**: Larger touch targets for tablet use

## Related Documentation

- [Split View](./split-view.md)
- [Zen Mode](./zen-mode.md)
- [Auto-Save](./auto-save.md)
- [Settings Panel](../settings.md)
