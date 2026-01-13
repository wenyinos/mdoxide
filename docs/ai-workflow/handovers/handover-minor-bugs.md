# Bug Fix Handover - Minor UI Issues

## Overview

This handover covers 4 minor UI bugs that should be quick fixes. Complete all 4 in this session.

---

## Environment

- **Project:** Sleek Markdown Editor (Rust + egui)
- **Path:** G:\DEV\markDownNotepad
- **Run:** `cargo run` to test changes

---

## Bug 1: Recent Files Menu - Double Click Required on First Open

**Location:** Status bar (bottom left) - click on file path to open recent files menu

**Problem:** The first time you click the file path to open the recent files popup menu, nothing happens. You have to click twice. After that it works with single click.

**Likely Cause:** The popup/menu state isn't being toggled correctly on first interaction, or focus isn't being set properly.

**Key Files:**
- `src/app.rs` - Look for recent files menu rendering and click handling
- Search for: `recent_files`, `popup`, `file_path` click handler

**Fix:** Ensure the menu state toggles correctly on first click. May need to request focus or ensure the popup ID is consistent.

---

## Bug 2: Recent Files Menu - Text Unreadable in Light Mode

**Problem:** In light mode, the recent files menu text is very light gray on a white background - nearly invisible. See the screenshot showing file paths that are barely readable.

**Screenshot:** `assets/bug-recent-files-light-mode.png` - Shows paths like `G:\DEV\markDownNotepad\.taskmaster` but text color is too light.

**Key Files:**
- `src/app.rs` - Recent files menu rendering
- `src/theme/mod.rs` or `src/theme/light.rs` - Theme colors

**Fix:** The menu text color needs to use proper theme-aware colors. Look for hardcoded light gray colors in the recent files menu and replace with `theme_colors.text.primary` or similar.

---

## Bug 3: Theme Toggle Should Skip "System" Option

**Location:** Status bar (bottom right) - theme toggle button (sun/moon icon)

**Current Behavior:** Clicking cycles through: Light → Dark → System → Light...

**Expected Behavior:** Clicking should only toggle: Light ↔ Dark (skip System entirely)

**Key Files:**
- `src/app.rs` - Theme toggle button click handler
- `src/config/settings.rs` - Theme enum definition

**Fix:** In the theme toggle click handler, change the cycling logic:
```rust
// Current (wrong):
Theme::Light => Theme::Dark,
Theme::Dark => Theme::System,
Theme::System => Theme::Light,

// Fixed:
Theme::Light => Theme::Dark,
Theme::Dark => Theme::Light,
Theme::System => Theme::Light, // If somehow on System, go to Light
```

---

## Bug 4: Settings Icon Vertical Alignment

**Location:** Status bar (bottom right) - the gear icon for Settings

**Problem:** The settings gear icon is positioned too high compared to the theme toggle icon next to it. They should be vertically centered/aligned.

**Screenshot:** `assets/bug-icon-alignment.png` - Shows "Settings" label with two icons - the gear is noticeably higher than the theme icon.

**Key Files:**
- `src/app.rs` - Status bar rendering, specifically where the settings button is rendered

**Fix:** Look for the settings icon button rendering. It may need:
- Consistent vertical alignment with `ui.with_layout()` using `Layout::centered_and_justified()`
- Or explicit vertical centering in the horizontal layout
- Check if one icon has different padding/margins than the other

---

## Bug 5: Change View Mode Shortcut (Ctrl+Shift+V Conflicts)

**Problem:** `Ctrl+Shift+V` is the paste shortcut on many systems, so it conflicts with the "Toggle View Mode" shortcut.

**Current:** Ctrl+Shift+V → Toggle between Raw/Rendered view

**Suggested New Shortcut:** `Ctrl+E` (common for "Edit mode" toggle) or `F5` (common for preview)

**Key Files:**
- `src/app.rs` - Keyboard shortcut handling
- `src/ui/ribbon.rs` - Tooltip text mentioning the shortcut

**Fix:** 
1. Change the keyboard shortcut detection from `Ctrl+Shift+V` to the new shortcut
2. Update ALL tooltip strings that mention the old shortcut
3. Search for: `Ctrl+Shift+V`, `ctrl`, `shift`, `v`, `ToggleViewMode`

---

## Testing Checklist

After fixes, verify:

- [ ] Recent files menu opens on FIRST click
- [ ] Recent files menu text is readable in Light mode
- [ ] Theme toggle only switches between Light ↔ Dark
- [ ] Settings and theme icons are vertically aligned in status bar
- [ ] New view mode shortcut works (old one should NOT)
- [ ] All tooltips show correct new shortcut
- [ ] `cargo build` passes with no new warnings

---

## Key Search Terms

```
recent_files
popup
theme_colors.text
Theme::System
settings.*icon
ToggleViewMode
Ctrl+Shift+V
```

@src/app.rs @src/ui/ribbon.rs @src/theme/mod.rs @src/config/settings.rs
