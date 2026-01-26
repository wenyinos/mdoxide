# Handover: v0.2.6 Bug Fixes

## Rules (DO NOT UPDATE)
- Never auto-update this file - only update when explicitly requested
- Run `cargo build` after changes to verify code compiles
- Follow existing code patterns and conventions
- Use Context7 MCP tool to fetch library documentation when needed
- Document by feature (e.g., memory-optimization.md), not by task
- Update docs/index.md when adding new documentation
- **Branch**: `feature/ferrite-editor`

---

## Current Task

**Task 55: Fix Ctrl+G Go to Line Not Working**

Ctrl+G keyboard shortcut for Go to Line dialog doesn't work. Need to verify shortcut is registered and dialog appears.

### Problem
- Pressing Ctrl+G does nothing
- Should open a Go to Line dialog

### Investigation Areas
1. Check if GoToLine shortcut is registered in `settings.rs`
2. Check if `KeyboardAction::GoToLine` is handled
3. Verify the Go to Line dialog exists and is wired up
4. May need to implement the dialog if missing

### Dialog Requirements
- Open small input dialog
- Accept line number
- Navigate to that line on Enter
- Handle invalid input gracefully (out of range, non-numeric)

### Test Strategy
1. Press Ctrl+G - dialog should appear
2. Enter line number - should navigate to that line
3. Enter invalid number - should show error or clamp
4. Escape should close dialog

---

## Key Files for Task 55

| File | Purpose |
|------|---------|
| `src/config/settings.rs` | Keyboard shortcuts registration |
| `src/app.rs` | Main app, keyboard action handling |
| `src/ui/dialogs.rs` | Dialog implementations |

### Areas to Investigate
- Search for `GoToLine` or `go_to_line` in codebase
- Check `KeyboardAction` enum for existing action
- Look at how other dialogs (Find, Replace) are triggered and displayed

---

## Environment

- **Project**: Ferrite (Markdown editor)
- **Language**: Rust
- **GUI Framework**: egui 0.28
- **Branch**: `feature/ferrite-editor`
- **Build**: `cargo build`
