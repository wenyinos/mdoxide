# PRD: Ferrite v0.2.6.1 - Patch & Stability

## Overview

v0.2.6.1 is a patch release focusing on:
1. **Bug fixes** reported after v0.2.6 release
2. **PR #74 integration** - Terminal & Productivity Hub with performance optimization
3. **Code signing** - Windows builds via SignPath.io

---

## P0 - Critical Bug Fixes

### #46 - Keyboard Shortcut Conflict (Ctrl+B)

**Problem:** Ctrl+B is assigned to both "Bold" formatting AND "Toggle File Tree", causing unexpected behavior when formatting text.

**Location:** `src/config/settings.rs` lines 606 and 625

**Fix:**
- Keep Ctrl+B for Bold (standard markdown convention)
- Change File Tree toggle to Ctrl+Shift+E (matches VS Code)
- Audit all shortcuts for other conflicts

**Test:** Select text, press Ctrl+B → text should become bold, file tree should NOT toggle.

---

### #73 - Undo Doesn't Work After Formatting

**Problem:** After selecting text and applying a format (like bold), Ctrl+Z deletes the text entirely instead of undoing just the format.

**Expected:** Ctrl+Z should revert `**text**` back to `text`
**Actual:** Ctrl+Z removes the text completely

**Root Cause:** Likely the formatting operation isn't being recorded properly in the undo history, or it's being grouped with prior text input.

**Fix:**
- Ensure format operations are recorded as discrete undo entries
- Test undo/redo cycle for all formatting operations (bold, italic, strikethrough, etc.)

**Test:** 
1. Type "hello world"
2. Select "world"
3. Press Ctrl+B (bold)
4. Press Ctrl+Z
5. Expected: "hello world" (bold removed, text remains)

---

## P1 - Markdown Rendering Fixes

### #71 - Multiline Blockquotes Render Separately

**Problem:** Blockquotes with explicit line breaks render as separate blockquotes instead of one continuous block.

**Input:**
```markdown
> The first line plus two blanks  
> followed by the second line
```

**Expected:** Single blockquote spanning two lines
**Actual:** Two separate blockquote boxes

**Fix:** Review blockquote parsing/rendering in `src/markdown/` - ensure consecutive `>` lines are grouped when separated by soft breaks (two trailing spaces).

**Test:** Verify multiline blockquotes render as single block with proper line breaks inside.

---

### #72 - Keep Selection After Formatting

**Problem:** When you select text and apply formatting (bold, italic, etc.), the selection is lost. To apply multiple formats, you must re-select the text each time.

**Expected:** Selection remains after formatting, allowing chained operations
**Actual:** Selection clears after each format application

**Fix:** 
- After applying format, restore selection to the formatted text range
- Account for added syntax characters (e.g., `**` adds 4 chars to selection boundaries)

**Test:**
1. Select "hello"
2. Press Ctrl+B → "**hello**" should still be selected
3. Press Ctrl+I → "***hello***" should still be selected

---

## P2 - CJK Typography

### #20 & #26 - Chinese/Japanese Paragraph Indentation

**Problem:** Chinese and Japanese typography conventions require first-line paragraph indentation:
- Chinese: 2 full-width characters (2em)
- Japanese: 1 full-width character (1em)

Currently, the entire paragraph indents instead of just the first line.

**Fix:**
- Implement `text-indent` style CSS equivalent for rendered paragraphs
- Apply based on detected language or user setting
- Only indent first line, not entire paragraph

**Test:** 
1. Create markdown with Chinese text paragraphs
2. Enable CJK indentation in settings
3. Rendered view should show 2em indent on first line only

---

## P3 - PR #74 Integration

PR #74 adds significant new features that require optimization before release:

### Memory Optimization (BLOCKER)

**Current:** ~300MB baseline memory (4x increase from ~75MB)
**Target:** ≤150MB baseline (2x increase acceptable)

**Investigation Tasks:**
- [ ] Profile memory allocation (terminal buffers, screen buffer, task storage)
- [ ] Cap terminal scrollback at 10,000 lines with circular buffer
- [ ] Review Tokio runtime thread configuration
- [ ] Verify PTY handles are properly dropped on terminal close
- [ ] Test with large task lists (1,000+ tasks)

### UI Polish

**Required Changes:**
- [ ] Remove "Coming Soon" placeholders (AI Assistant, Database Tools, SSH Sessions)
- [ ] Integrate terminal toggle into existing toolbar (not separate View menu)
- [ ] Ensure panel styling matches Ferrite design language
- [ ] Verify dark/light theme support for new panels

### Error Handling

- [ ] Surface PTY spawn errors to user (toast notification)
- [ ] Log workspace sync failures
- [ ] Add recovery hints for common errors

---

## P4 - Code Signing

### Windows Code Signing via SignPath.io

**Goal:** Eliminate Windows Defender false positives by signing all Windows artifacts.

**Status:** Pending approval from SignPath.io OSS program

**Artifacts to Sign:**
- `ferrite.exe` (standalone executable)
- `ferrite-x.x.x-x64.msi` (MSI installer)

**Configuration:** `.signpath/artifact-configuration.xml` already prepared

---

## Testing Checklist

Before release:

- [ ] All P0 bugs verified fixed
- [ ] Multiline blockquotes render correctly
- [ ] Selection persists after formatting
- [ ] CJK indentation works (Chinese 2em, Japanese 1em)
- [ ] Memory usage ≤150MB baseline
- [ ] Terminal feature works (spawn, input, output, close)
- [ ] Productivity hub works (tasks, timer, notes)
- [ ] No regressions in existing features
- [ ] Cross-platform test (Windows required, Linux/macOS if available)

---

## Out of Scope (Deferred)

- #63 - CJK Font Crash (already fixed)
- #57 - Portable Crash (already fixed)
- Diagnostics panel (track separately)
- Command palette (future enhancement)
- Advanced terminal features (SSH, AI integration)

---

## Release Notes Draft

### v0.2.6.1 - Patch & Stability

**Bug Fixes:**
- Fixed Ctrl+B shortcut conflict (Bold vs File Tree toggle)
- Fixed undo not working after applying text formatting
- Fixed multiline blockquotes rendering as separate blocks
- Selection now persists after applying formatting

**New Features:**
- Integrated terminal emulator (Ctrl+` to toggle)
- Productivity hub with tasks, Pomodoro timer, and notes (Ctrl+Shift+H)
- Chinese/Japanese first-line paragraph indentation

**Improvements:**
- Memory optimization for terminal feature
- Windows builds now code signed (reduces antivirus false positives)

---

**Document Version:** 1.0
**Created:** February 4, 2026
**Status:** Draft - Ready for Review
