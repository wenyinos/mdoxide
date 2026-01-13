# PRD: Ferrite v0.2.2 - Stability, CLI & Quality of Life

## Product Overview

Ferrite v0.2.2 is a focused release addressing bugs reported after the v0.2.1 launch, improving command-line interface (CLI) usability, and adding quality-of-life features requested by early users.

## Problem Statement

After launching v0.2.1 on Reddit/HN, users reported several issues:
1. **Installation failures** on Ubuntu 22.04 LTS due to glibc version mismatch
2. **CJK text rendering** - Korean/Chinese/Japanese characters appear as boxes
3. **Undo behavior bugs** - Ctrl+Z resets scroll, loses focus, needs double-press
4. **CLI limitations** - Can't open files from terminal, no --version flag, verbose logging
5. **UX friction** - Must manually enable split view for each new tab

This release prioritizes stability and developer experience over new features.

## Target Users

1. Linux users (especially Ubuntu 22.04 LTS)
2. International users (Korean, Chinese, Japanese speakers)
3. CLI-focused developers who launch editors from terminal
4. Power users who prefer split/preview view as default

## Goals

1. Fix all critical bugs reported in GitHub issues
2. Provide standard CLI interface (--help, --version, file arguments)
3. Reduce friction for common workflows
4. Improve code quality in mermaid.rs module

## Non-Goals

1. New major features (save for v0.3.0)
2. Mermaid crate extraction (v0.3.0)
3. Custom editor widget (v0.3.0)
4. Wikilinks support (v0.3.0)

---

## Features

### 1. Bug Fixes (P0 - Critical)

#### 1.1 Ubuntu 22.04 Compatibility (Issue #6) ✅ DONE
- **Status:** Fix merged in release.yml
- **Problem:** .deb built on Ubuntu 24.04 requires glibc 2.39, but 22.04 has 2.35
- **Solution:** Changed CI to build on `ubuntu-22.04` runner
- **Files:** `.github/workflows/release.yml`

#### 1.2 CJK Character Rendering (Issue #7)
- **Problem:** Korean/Chinese/Japanese characters appear as empty boxes (□)
- **Root Cause:** Bundled fonts (Inter, JetBrains Mono) lack CJK glyphs
- **Solution Options:**
  - A) System font fallback (preferred - no binary size increase)
  - B) Bundle subset CJK font (~1-2 MB)
  - C) Bundle full Noto Sans KR (~15 MB) - PR #8 approach
- **Recommended:** Try system fonts first (Windows: "Malgun Gothic", macOS: "Apple SD Gothic Neo", Linux: "Noto Sans CJK")
- **Files:** `src/fonts.rs`
- **Testing:** Create test file with Korean (한글), Chinese (中文), Japanese (日本語)

#### 1.3 Undo/Redo Behavior (Issue #5)
- **Problem:** Ctrl+Z has multiple issues:
  - Resets scroll position to top of document
  - Editor loses keyboard focus (must click to type again)
  - Requires double-press to actually undo
- **Root Cause:** Likely full document refresh instead of minimal state update
- **Solution:** Investigate undo implementation, preserve scroll/focus state
- **Files:** `src/editor/widget.rs`, `src/state.rs`
- **Testing:** Type text, scroll down, delete text, Ctrl+Z - should restore text AND stay at same scroll position

#### 1.4 UTF-8 Tree Viewer Crash
- **Problem:** String slicing panic when displaying JSON/YAML with multi-byte characters
- **Root Cause:** Byte-based string slicing on UTF-8 text
- **Solution:** Use char-based indexing or handle grapheme boundaries
- **Files:** `src/markdown/tree_viewer.rs`
- **Testing:** Open JSON with Norwegian (øæå), emoji (🎉), Chinese characters

---

### 2. CLI Improvements (P1 - High)

#### 2.1 Command-Line File Opening (Issue #9)
- **Feature:** Open files directly from terminal
- **Usage:**
  ```
  ferrite myfile.md              # Open single file
  ferrite file1.md file2.md      # Open multiple files as tabs
  ferrite .                       # Open current directory as workspace
  ```
- **Implementation:**
  - Add `clap` crate for argument parsing
  - Parse positional arguments as file paths
  - Open each as a new tab on startup
- **Files:** `src/main.rs`, `Cargo.toml`

#### 2.2 Version/Help Flags (Issue #10)
- **Feature:** Standard CLI flags
- **Usage:**
  ```
  ferrite --version, -V    # Print "Ferrite 0.2.2"
  ferrite --help, -h       # Print usage info
  ```
- **Implementation:**
  - Use `clap` derive macros
  - Version from Cargo.toml via env!("CARGO_PKG_VERSION")
- **Files:** `src/main.rs`, `Cargo.toml`
- **Note:** About dialog already exists (Help → About), this adds CLI equivalent

#### 2.3 Configurable Log Level (Issue #11)
- **Problem:** Launching Ferrite prints ~39 log lines to stderr, cluttering terminal
- **Feature:** Add `log_level` setting in config.json
- **Levels:** `DEBUG`, `INFO`, `WARN`, `ERROR`, `OFF`
- **Default:** `WARN` (show only warnings and errors)
- **Implementation:**
  - Add field to config struct
  - Set log filter based on config on startup
- **Files:** `src/config/settings.rs`, `src/main.rs`

---

### 3. UX Improvements (P1 - High)

#### 3.1 Default View Mode Setting (Issue #3)
- **Problem:** Split view must be enabled manually for each new tab
- **Feature:** Global setting for default view mode
- **Options:**
  - `Raw` - Editor only (current default)
  - `Rendered` - Preview only
  - `Split` - Side-by-side
- **Implementation:**
  - Add `default_view_mode` to config
  - Apply when creating new tabs
  - Respect per-tab overrides
- **Files:** `src/config/settings.rs`, `src/app.rs`, `src/ui/settings.rs`

---

### 4. Mermaid Code Quality (P2 - Medium)

#### 4.1 Rendering Performance
- **Problem:** Complex diagrams (50+ nodes) can be slow
- **Solution:** Profile and optimize hot paths in mermaid.rs
- **Files:** `src/markdown/mermaid.rs`

#### 4.2 Code Cleanup
- **Problem:** Unused code warnings, large monolithic file
- **Solution:**
  - Remove dead code
  - Consider splitting into submodules (parser.rs, layout.rs, render.rs)
  - Add documentation comments
- **Files:** `src/markdown/mermaid.rs`

---

## Technical Requirements

### Dependencies to Add
- `clap` with derive feature for CLI parsing (small, well-maintained)

### Dependencies to Consider
- None - avoid adding dependencies for this release

### Testing Requirements
1. Manual test on Ubuntu 22.04 LTS
2. Manual test CJK rendering on Windows/macOS/Linux
3. Test undo/redo with scroll position preservation
4. Test CLI flags work correctly
5. Test config.json log_level actually filters logs

### Backward Compatibility
- Config changes must have sensible defaults
- Existing config.json files should work unchanged

---

## Implementation Priority

### Must Ship (Blockers)
1. ✅ Ubuntu 22.04 compatibility (done)
2. CJK character rendering
3. Undo/redo behavior

### Should Ship
4. CLI file opening
5. Version/help flags
6. Log level config
7. Default view mode

### Nice to Have
8. UTF-8 tree viewer fix
9. Mermaid performance
10. Mermaid code cleanup

---

## Success Metrics

1. No installation failures on Ubuntu 22.04
2. Korean/Chinese/Japanese text renders correctly
3. Ctrl+Z works without scroll reset or focus loss
4. `ferrite --version` prints version
5. `ferrite file.md` opens the file
6. Default log output is minimal (WARN level)

---

## GitHub Issues Reference

| Issue | Title | Priority |
|-------|-------|----------|
| #3 | Default split view option | P1 |
| #5 | Ctrl+Z behaves weirdly | P0 |
| #6 | Ubuntu 22.04 .deb install | P0 ✅ |
| #7 | CJK characters not displaying | P0 |
| #9 | CLI file opening | P1 |
| #10 | Version/help flags | P1 |
| #11 | Log level config | P1 |

---

## Open Questions

1. **CJK fonts:** System fallback vs bundled font? (Recommend system first)
2. **CLI parser:** Use `clap` or hand-roll simple parser?
3. **Log level:** Should it also be settable via CLI flag `--log-level`?
4. **Default view:** Should it be in Settings UI or just config.json?

---

## Appendix: Files Likely to Change

```
src/main.rs              # CLI parsing, log init
src/config/settings.rs   # New config fields
src/fonts.rs             # CJK fallback
src/editor/widget.rs     # Undo/redo fix
src/state.rs             # Undo state management
src/app.rs               # Default view mode
src/markdown/tree_viewer.rs  # UTF-8 fix
src/markdown/mermaid.rs  # Code cleanup
.github/workflows/release.yml  # Already updated
Cargo.toml               # Add clap dependency
```

---

## Appendix: Items Awaiting External Response

These items have open Pull Requests or discussions - wait for contributor replies before proceeding:

### PR #8 - CJK Font Support (Korean/Chinese/Japanese)
- **Status:** Awaiting contributor response
- **Issue:** PR adds 15.7 MB Noto Sans KR font, significant binary size increase
- **Asked:** Whether contributor wants to explore alternatives:
  - System font fallback (no size increase)
  - Font subsetting (reduce to ~1-2 MB)
  - Keep current approach if size is acceptable
- **Action:** Wait for reply before deciding implementation approach for Issue #7

### PR #2 - macOS Binary Naming Fix
- **Status:** Awaiting contributor response  
- **Issue:** Renamed macos-x64 to macos-arm64, but build is actually on ARM runner
- **Asked:** Whether contributor wants to:
  - A) Build for both architectures (x64 + ARM)
  - B) Keep ARM-only with correct naming
- **Action:** Wait for reply before merging

### Note for Task Generation
When implementing CJK support (Issue #7), check PR #8 status first. If contributor prefers system fonts, implement that approach instead of bundling fonts.
