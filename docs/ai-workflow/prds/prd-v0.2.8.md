# PRD: Ferrite v0.2.8 - Performance, Accessibility, Text Shaping & Bug Fixes

## Overview

v0.2.8 focuses on rendered view performance (the top user-reported blocker), bug fixes for IME and markdown rendering, new markdown rendering settings, executable code blocks (deferred from v0.2.7), Unicode text shaping (Phase 2), and LSP integration foundations.

**Theme:** Performance, accessibility, text shaping, code intelligence, and new subsystems.

**Scope:** Rendered view AST caching + viewport culling, 3 targeted bug fixes, strict line breaks setting, executable code blocks (opt-in), HarfRust text shaping integration, LSP client infrastructure, traditional menu bar, and additional format support.

**Depends on:** v0.2.7 (released March 2026) — specifically Phase 1 complex script font loading, FerriteEditor virtual scrolling, Mermaid caching patterns, and `LineCache` LRU infrastructure.

---

## P0 - Rendered View Performance (#105)

*Two users reported rendered view as unusable on 6k+ line / 50k+ word files. This is the highest-priority item for v0.2.8.*

### Problem

The rendered/preview mode re-parses the full markdown AST and builds all egui widgets for the entire document every frame (60 times/second). There is no caching and no viewport culling. On large documents this creates severe lag for both scrolling and idle states, while the raw editor (FerriteEditor) handles the same files smoothly via virtual scrolling.

### Markdown AST Caching

**Requirement:** Cache the parsed `MarkdownDocument` and only re-parse when the document content actually changes.

**Implementation:**
- Compute a blake3 content hash of the document text (same pattern as Mermaid diagram caching in `src/markdown/mermaid/mod.rs` — `MermaidCache`)
- Store the cached `MarkdownDocument` alongside the hash
- On each frame: compare current content hash to cached hash; skip `parse_markdown()` on match
- Invalidation: any edit to `Tab.content` changes the hash, triggering a re-parse on the next frame
- Location: `src/markdown/editor.rs` (rendered view entry point) or a new `src/markdown/cache.rs`

**Test:**
1. Open a large markdown file (5k+ lines) in rendered view
2. Scroll without editing — no re-parse should occur (verify via log or perf counter)
3. Edit a character — re-parse triggers once, then caching resumes
4. Toggle between tabs — each tab maintains its own cached AST

---

### Rendered View Viewport Culling

**Requirement:** Only construct egui widgets for blocks visible in the viewport plus an overscan buffer. Off-screen blocks should be replaced with `ui.allocate_space()` using cached heights.

**Implementation:**
- Switch from `ScrollArea::vertical().show()` to `.show_viewport()` in the rendered view scroll area
- Maintain a `Vec<f32>` of block heights (one per top-level AST node), following the `cumulative_heights` pattern in `src/editor/ferrite/view.rs` and `CachedVisibleRows` in `src/markdown/csv_viewer.rs`
- For each frame: determine which block indices fall within `viewport_rect.y_range()` plus overscan (e.g. 500px above/below)
- Call `render_node()` only for visible blocks; call `ui.allocate_space(vec2(available_width, cached_height))` for off-screen blocks
- First render of a block: measure and store its height; subsequent off-screen passes use the cached value
- Location: `src/markdown/editor.rs` rendering loop

**Test:**
1. Open a 6k-line markdown file in rendered view — frame rate should stay above 30fps
2. Scroll smoothly — content appears as it enters viewport with no visual glitches
3. Resize window — block heights update correctly
4. Compare CPU usage with and without culling on a large file

---

### Block-Level Height Cache

**Requirement:** Cache measured heights per rendered block so that off-screen blocks don't need re-measurement, and the scrollbar can position accurately.

**Implementation:**
- Key each block's height on its content hash (blake3 of the block's markdown source text)
- Use an LRU cache following the `LineCache` pattern in `src/editor/ferrite/line_cache.rs`
- Invalidation: when a block's content changes, its hash changes, evicting the stale entry
- The height cache enables accurate `total_content_height` for scrollbar sizing without measuring every block
- Location: `src/markdown/cache.rs` or extend the AST cache module

**Test:**
1. Scroll through a large file — scrollbar thumb position should be accurate and stable
2. Edit a paragraph — only that block's height is re-measured; other cached heights remain
3. Memory usage stays bounded (LRU eviction prevents unbounded growth)

---

## P1 - Bug Fixes

### macOS .md File Association (#102)

**Problem:** macOS Finder "Open With" and double-click don't work for `.md` files because the app bundle's `Info.plist` is missing the `UTImportedTypeDeclarations` block needed to declare the `net.daringfireball.markdown` UTI.

**Fix:**
- Add `UTImportedTypeDeclarations` to `assets/macos/info_plist_ext.xml`
- Declare UTI `net.daringfireball.markdown` conforming to `public.plain-text`
- Cover extensions: `md`, `markdown`, `mdown`, `mkd`, `mkdn`

**Location:** `assets/macos/info_plist_ext.xml`, CI bundle workflow

**Test:**
1. Build `.app` bundle with `cargo bundle --release`
2. Right-click a `.md` file in Finder → "Open With" lists Ferrite
3. Set Ferrite as default → double-click `.md` opens in Ferrite

---

### Windows IME Candidate Box Mispositioning (#103, #15)

**Problem:** Chinese/Japanese/Korean IME candidate selection box appears at the wrong position (bottom of screen) or is invisible on Windows 11. Root cause: the custom FerriteEditor sets `IMEOutput` with local widget coordinates instead of applying `layer_transform_to_global()` like egui's built-in `TextEdit`.

**Fix:**
- In `src/editor/ferrite/editor.rs`, where `IMEOutput` is set, apply the `to_global` coordinate transform:
  ```rust
  let to_global = ui.ctx().layer_transform_to_global(ui.layer_id());
  o.ime = Some(IMEOutput {
      rect: to_global * rect,
      cursor_rect: to_global * cursor_rect,
  });
  ```
- If the coordinate transform alone is insufficient (z-order issue with `with_decorations(false)`), investigate Win32 z-order workarounds for the candidate popup

**Location:** `src/editor/ferrite/editor.rs` — IME output section

**Test:**
1. Windows 11 with Microsoft Pinyin (or Sogou) IME
2. Switch to Chinese input mode in Ferrite
3. Type pinyin → candidate box appears directly below the cursor
4. Test in both light and dark themes
5. Test with custom window decorations enabled/disabled

---

### Double-Dash (`--`) False Setext Headings and Line Collapsing

**Problem:** Two related rendering bugs when `--` appears in markdown:
1. Text followed by `--` on the next line is interpreted as a setext H2 heading (e.g. `Hvordan\n--` renders "Hvordan" as H2). The v0.2.7 `fix_false_setext_headings()` in `parser.rs` only catches single-dash (`trimmed == "-"`), not `"--"`.
2. Multiple lines starting with `-- ` collapse into a single paragraph because CommonMark treats soft line breaks as spaces.

**Fix:**
- Extend `fix_false_setext_headings()` in `src/markdown/parser.rs` to also treat `"--"` (and possibly `"---"` outside frontmatter context) as false setext underlines
- For the line collapsing issue: preprocess `\n-- ` to `\n\n-- ` to force paragraph breaks, or post-process the AST to split paragraphs at `-- ` boundaries
- Choose the approach that minimizes interference with legitimate markdown constructs (horizontal rules `---` in proper context)

**Location:** `src/markdown/parser.rs` — `fix_false_setext_headings()` and potentially a new preprocessing step

**Test:**
1. `Hvordan\n--` → renders as paragraph "Hvordan" followed by text "--", NOT as an H2 heading
2. `-- skal\n-- jeg\n-- asd` → renders as three separate lines, not collapsed into one paragraph
3. `---` between blank lines → still renders as a horizontal rule (no regression)
4. YAML frontmatter `---` delimiters → still recognized correctly

---

## P2 - Markdown Rendering Settings

### Strict Line Breaks Toggle

**Feature:** Add "Strict line breaks" toggle to Settings → Editor. When enabled, single newlines in markdown source render as hard line breaks (`<br>`) instead of being treated as spaces.

**Requirements:**
- New `strict_line_breaks: bool` field in `Settings` struct (default: `false`)
- Wire to Comrak's `render.hardbreaks` option when parsing markdown for rendered view
- Toggle in Settings → Editor section (alongside word wrap, line numbers, etc.)
- Follows the Obsidian model: spec-compliant soft breaks are the default, but users can opt into newline-as-linebreak behavior

**Implementation:**
- Add field to `src/config/settings.rs` → `Settings` struct
- Pass setting value when constructing Comrak options in `src/markdown/parser.rs`
- Add toggle UI in `src/ui/settings.rs` → Editor section

**Test:**
1. Default off: `line one\nline two` renders as "line one line two" (single paragraph)
2. Toggle on: `line one\nline two` renders as "line one<br>line two" (two visual lines)
3. Setting persists across restarts
4. Rendered, split, and WYSIWYG views all respect the setting

---

### Strict Line Breaks on Welcome Page

**Feature:** Add the strict line breaks toggle to the Welcome page editor settings section so new users can configure this preference on first launch.

**Requirements:**
- Add toggle to Welcome page alongside existing editor settings (word wrap, line numbers, minimap, bracket matching, syntax highlighting)
- Same behavior and visual style as the other Welcome page toggles
- Value syncs with the `Settings.strict_line_breaks` field

**Location:** `src/ui/welcome.rs` (or equivalent Welcome panel module)

**Test:**
1. Fresh install → Welcome page shows strict line breaks toggle
2. Toggle it on → setting is persisted; rendered view uses hard breaks

---

## P3 - Executable Code Blocks

*Security note: Code execution is **opt-in and disabled by default**. Deferred from v0.2.7.*

### Run Button on Code Blocks

**Feature:** Add a `▶ Run` button to fenced code blocks in rendered/split view.

**Requirements:**
- Small "Run" button rendered on eligible code blocks (those with recognized language tags: `bash`, `sh`, `shell`, `python`, `py`)
- Button only visible when code execution is enabled in settings
- Click triggers execution; output displayed inline below the code block in a collapsible output section
- Visual states: idle, running (spinner), completed (green), error (red)

**Implementation:**
- New module: `src/code_execution/` with `mod.rs`, `runner.rs`, `output.rs`
- Settings: `code_execution_enabled: bool` (default false), `code_execution_timeout_secs: u32` (default 30)
- UI: Extend code block rendering in `src/markdown/editor.rs` or `src/markdown/widgets.rs`

---

### Shell / Bash Execution

**Feature:** Execute shell/bash code block snippets via `std::process::Command`.

**Requirements:**
- **Unix:** Execute via `sh` or `bash`
- **Windows:** Default to `powershell` or `cmd`; detect if `bash.exe` exists (WSL/Git Bash) and prefer it when available
- **CWD:** Run script in the file's parent directory (user expectation for relative paths). Fallback to workspace root if file is unsaved
- Capture stdout and stderr separately; display both in the output section
- Environment: inherit current process environment

**Implementation:**
- `src/code_execution/runner.rs` — `execute_shell()` function
- Platform detection for shell binary selection
- Spawn child process, capture output streams

**Test:**
1. Unix: `echo "hello"` in a ```bash block → output shows "hello"
2. Windows: `Write-Host "hello"` in a ```shell block → output shows "hello" (via powershell)
3. Script with `ls .` → lists files in the markdown file's directory, not the app's directory
4. Script writing to stderr → stderr shown in output (visually distinct)

---

### Python Support

**Feature:** Detect `python` or `python3` and run code blocks with the system Python interpreter.

**Requirements:**
- Detect interpreter: try `python3` first (Unix convention), fall back to `python`
- If neither found, show helpful error message: "Python not found. Install Python to run Python code blocks."
- CWD: same as shell execution (file's directory)

**Test:**
1. ```python block with `print("hello")` → output shows "hello"
2. Python not installed → error message shown (no crash)
3. Script with `import os; print(os.getcwd())` → prints the markdown file's directory

---

### Timeout Handling

**Feature:** Kill long-running scripts after a configurable timeout.

**Requirements:**
- Default timeout: 30 seconds
- Configurable in Settings → Editor → Code Execution section
- When timeout is reached: kill the child process, show "(killed after Xs)" in output
- Use `std::thread::spawn` + sleep + `Child::kill()` pattern, or `tokio::time::timeout` if using async

**Test:**
1. Run `sleep 60` with 30s timeout → process killed after 30s; output shows timeout message
2. Change timeout to 5s → process killed after 5s
3. Normal scripts completing within timeout → no interference

---

### Security Warning

**Feature:** First-run dialog explaining code execution risks.

**Requirements:**
- On first attempt to enable code execution (either in Settings or clicking Run), show a modal dialog:
  - Title: "Enable Code Execution?"
  - Body: "Code blocks will be executed on your machine with your user permissions. Only run code you trust."
  - Buttons: "Enable" / "Cancel"
- Store `code_execution_acknowledged: bool` in settings; don't show again after acknowledgment
- The dialog is purely informational/consent — no sandboxing is implemented

**Test:**
1. First enable attempt → dialog appears
2. Click "Enable" → code execution enabled; dialog won't appear again
3. Click "Cancel" → code execution remains disabled
4. Reset settings → dialog appears again on next enable attempt

---

## P4 - Unicode & Complex Script Support (Phase 2: Text Shaping Engine)

*Depends on: Phase 1 font loading from v0.2.7 (11 script families, lazy loading in `fonts.rs`)*

### HarfRust Integration for FerriteEditor

**Feature:** Integrate HarfRust (pure-Rust HarfBuzz port, v0.5.0+) into the FerriteEditor rendering pipeline for production-quality text shaping.

**Requirements:**
- Add `harfrust` crate dependency (pure Rust, no C dependencies)
- Create a shaping pipeline that converts Unicode codepoint sequences into correctly positioned, contextually-formed glyphs
- Support contextual forms for Arabic (initial/medial/final/isolated), Bengali conjunct consonants, Devanagari, Tamil, and other Indic scripts
- Integration point: between text buffer content retrieval and galley construction in the FerriteEditor rendering path
- Fallback: if shaping fails for a run, fall back to current glyph-by-glyph rendering (graceful degradation)

**Implementation:**
- New module: `src/editor/ferrite/shaping.rs` (or `src/editor/shaping/`)
- Shaping pipeline: `&str` → HarfRust shape → `Vec<ShapedGlyph>` (glyph ID + x_advance + y_advance + x_offset + y_offset)
- Font data: pass loaded font bytes from `fonts.rs` to HarfRust's `Face`
- Script/language detection: use `unicode-script` crate or HarfRust's built-in detection to set script/language on the shaping buffer

**Test:**
1. Arabic text: letters connect with proper contextual forms (isolated letter vs. initial/medial/final)
2. Bengali text: conjunct consonants render as single visual units
3. Latin text: no visual regression (shaping is a no-op for simple scripts)
4. Mixed script line: Arabic + Latin renders correctly with proper glyph positioning
5. Performance: shaping overhead negligible for typical file sizes

---

### Shaped Galley Cache

**Feature:** Extend `LineCache` to store shaped text runs (glyph IDs + positions) instead of raw character galleys.

**Requirements:**
- Cache shaped output per line, keyed on content + font + script
- Invalidation on: content change, font change, viewport resize (if width affects shaping)
- Follow existing `LineCache` LRU pattern in `src/editor/ferrite/line_cache.rs`
- Shaped runs are more expensive to compute than raw galleys, making caching more impactful

**Implementation:**
- Extend or create a parallel cache in `line_cache.rs` for shaped runs
- Cache key: (line content hash, font ID, available width)
- Cache value: `Vec<ShapedGlyph>` or equivalent shaped run data

**Test:**
1. Type Arabic text → first render shapes; subsequent frames use cache (no re-shaping)
2. Edit a line → only that line is re-shaped; others use cache
3. Change font → entire cache invalidated; re-shaping occurs
4. Memory: LRU eviction keeps cache bounded

---

### Grapheme-Cluster-Aware Cursor

**Feature:** Replace character-based cursor movement with grapheme-cluster-aware navigation.

**Requirements:**
- A single visual "character" in Bengali, Arabic, or Korean may span multiple Unicode codepoints — cursor must step over the entire cluster
- Use `unicode-segmentation` crate for grapheme cluster boundary detection
- Arrow keys (left/right) move by one grapheme cluster, not one codepoint
- Backspace deletes one grapheme cluster
- Selection boundaries snap to grapheme cluster boundaries
- Home/End behavior unchanged (line-level, not affected)

**Implementation:**
- Add `unicode-segmentation` crate dependency
- Modify cursor movement in `src/editor/ferrite/cursor.rs` and input handling
- Replace `char_indices()` navigation with `grapheme_indices()` where applicable

**Test:**
1. Bengali conjunct: cursor treats the entire conjunct as one unit (single arrow press skips over it)
2. Korean jamo: cursor moves by syllable block, not individual jamo
3. Emoji with ZWJ (e.g. family emoji): cursor treats as single grapheme
4. Latin text: no change in behavior (each character is one grapheme)
5. Backspace on a multi-codepoint grapheme: deletes the entire cluster

---

### Shaped Text Measurement

**Feature:** Update word wrap, line width calculation, and scroll offset computation to use shaped advance widths instead of per-character metrics.

**Requirements:**
- Word wrap: use sum of shaped glyph advances for line width, not sum of individual character widths
- Line width calculation for scrollbar horizontal range: use shaped widths
- Cursor x-position: computed from cumulative shaped glyph advances up to cursor position
- Click-to-position: reverse map pixel x to glyph index using shaped advance widths

**Implementation:**
- Modify `src/editor/ferrite/view.rs` word wrap logic
- Modify cursor positioning and click mapping in `src/editor/ferrite/mouse.rs`
- Use shaped runs from cache for all width calculations

**Test:**
1. Arabic text with ligatures: line width reflects actual visual width (shorter than sum of isolated character widths)
2. Word wrap: wraps at correct position based on shaped widths
3. Click in shaped text: cursor lands at correct position
4. Horizontal scrollbar: reflects actual content width with shaped text

---

## P5 - LSP Integration (Language Server Protocol)

*Detailed plan: [docs/lsp-integration-plan.md](../../lsp-integration-plan.md)*

### Phase 1: Infrastructure

**Feature:** Core LSP client infrastructure — server lifecycle management, transport, and UI indicators.

**Requirements:**
- Auto-detect language server by file extension (e.g. `.rs` → `rust-analyzer`, `.py` → `pylsp`, `.ts` → `typescript-language-server`)
- Spawn server as child process via stdio on workspace open
- Graceful fallback: if server binary not found, show dismissable notification with install instructions; no crash
- Server lifecycle: start on workspace open, restart on crash (with backoff), shutdown on workspace close
- LSP toggle per workspace: opt-in, off by default
- Status bar indicator: server name + state (ready / indexing / not found / disabled)

**Implementation:**
- New module: `src/lsp/` with `mod.rs`, `manager.rs`, `transport.rs`, `detection.rs`, `state.rs`
- Threading: LSP manager runs on dedicated background thread (or tokio task); communicates with UI via `mpsc` channels
- `LspState` in `AppState`: `server_status: HashMap<String, ServerStatus>`
- Status bar: extend `src/app/status_bar.rs` with LSP indicator
- Settings: `lsp_enabled: bool` (default false), `lsp_server_overrides: HashMap<String, String>` in `src/config/settings.rs`
- JSON-RPC message framing: Content-Length header + JSON body over stdio

**Test:**
1. Open Rust workspace with `rust-analyzer` installed, LSP enabled → status bar shows "rust-analyzer ready"
2. Open workspace without language server → dismissable notification appears; status bar shows "not found"
3. LSP disabled → no server spawned; no notifications
4. Kill language server process → Ferrite detects crash, attempts restart
5. Close workspace → server process is shut down cleanly

---

### Phase 2: Inline Diagnostics

**Feature:** Display error/warning squiggles under affected text with hover tooltips.

**Requirements:**
- Receive `textDocument/publishDiagnostics` notifications from server
- Draw underline squiggles: red for errors, yellow for warnings, blue for info/hints
- Hover over squiggle → tooltip showing diagnostic message and severity
- Incremental document sync: send `textDocument/didChange` with changed ranges only (not full document)
- Diagnostic count in status bar: "2 errors, 1 warning"

**Implementation:**
- Diagnostics stored in `LspState.diagnostics: HashMap<PathBuf, Vec<Diagnostic>>`
- Document sync: map FerriteEditor buffer changes to LSP `TextDocumentContentChangeEvent` with range + text
- Squiggle rendering: in FerriteEditor paint phase, for each visible line check diagnostics and draw wavy underlines
- Tooltip: on hover, check if cursor is within a diagnostic range; show popup
- Location: `src/lsp/state.rs` for data, `src/editor/ferrite/rendering/` for squiggle painting

**Test:**
1. Rust file with compile error → red squiggle under the error location
2. Hover over squiggle → tooltip shows error message
3. Fix the error and save → squiggle disappears (server sends updated diagnostics)
4. Status bar shows diagnostic counts
5. Large file: diagnostics only drawn for visible lines (no performance impact)

---

### Phase 3: Hover & Go to Definition

**Feature:** Show documentation on hover and navigate to symbol definitions.

**Requirements:**
- Hover: show documentation/type info popup on mouse hover over identifiers, with configurable delay (default: 500ms)
- Go to Definition: F12 or Ctrl+Click jumps to the symbol's definition
- If definition is in a different file, open it in a new tab and navigate to the line
- If definition is in the same file, scroll to it

**Implementation:**
- Send `textDocument/hover` request on hover (debounced)
- Send `textDocument/definition` request on F12 / Ctrl+Click
- Hover popup: styled markdown rendering of the hover response (many servers return markdown)
- Navigation: reuse existing tab/file opening logic in `src/app/file_ops.rs`

**Test:**
1. Hover over a Rust function → popup shows function signature and doc comment
2. F12 on a function call → jumps to function definition
3. Ctrl+Click on an imported symbol → opens the defining file in a new tab
4. Hover with no server → no popup (graceful)

---

### Phase 4: Autocomplete

**Feature:** Completion popup triggered by typing or Ctrl+Space.

**Requirements:**
- Trigger on typing (debounced, e.g. 150ms after last keystroke) or explicit Ctrl+Space
- Popup: list of completion items, navigable with arrow keys, Enter to accept, Esc to dismiss
- Show completion kind icons (function, variable, type, keyword, snippet)
- Request cancellation: stale completions from a previous cursor position are discarded
- Text edits: apply the completion item's `textEdit` or `insertText` to the buffer

**Implementation:**
- Send `textDocument/completion` requests with debouncing
- Track request IDs; ignore responses for outdated positions
- Completion UI: custom popup widget in `src/editor/ferrite/` or `src/ui/`
- Apply completion: translate LSP text edits to FerriteEditor buffer operations

**Test:**
1. Type `std::` in a Rust file → completion popup shows module members
2. Arrow down + Enter → inserts the selected completion
3. Esc → dismisses popup
4. Fast typing → only the latest completion request is shown (no stale results)
5. Ctrl+Space with empty prefix → shows all available completions

---

### LSP Settings

**Feature:** LSP configuration in Settings panel.

**Requirements:**
- Per-language server path override (for users with custom installs)
- LSP enable/disable toggle (workspace-level)
- All processing local (no network calls from Ferrite itself)

**Location:** New "Language Servers" section in `src/ui/settings.rs`

---

## P6 - Traditional Menu Bar (#59)

### Alt-Key Menu Access

**Feature:** Traditional File/Edit/View menus toggled via Alt key (VS Code style).

**Requirements:**
- Press Alt → menu bar appears (overlaying or replacing the ribbon momentarily)
- Menus: File (New, Open, Save, Save As, Close, Exit), Edit (Undo, Redo, Cut, Copy, Paste, Find, Replace), View (Raw, Split, Rendered, Zen Mode, Toggle Outline, Toggle Terminal)
- Click menu item or press the underlined letter to activate
- Alt again or Esc → menu bar hides
- Does not replace the ribbon permanently — the ribbon remains the primary UI; Alt-key menus are for accessibility and keyboard-first users

**Implementation:**
- New module or extension: `src/ui/menu_bar.rs`
- State: `menu_bar_visible: bool` in `AppState` or `UiState`
- Render conditionally in `src/app/title_bar.rs` or above the central panel
- Key handling: intercept bare Alt key press in `src/app/keyboard.rs`

**Test:**
1. Press Alt → menu bar appears with File, Edit, View
2. Alt+F → File menu opens; press N → New File
3. Esc → menu bar hides
4. All menu items trigger the correct actions (same as existing keyboard shortcuts)

---

### Accessibility Keyboard Navigation

**Feature:** Full keyboard navigation for all menu items.

**Requirements:**
- Arrow keys navigate between menus (left/right) and within menu items (up/down)
- Enter activates the selected item
- Underlined mnemonics (e.g. Alt+F for File)
- Focus trapping while menu is open (Tab doesn't leave the menu)

**Test:**
1. Alt → Right arrow → moves to Edit menu
2. Down arrow → highlights first item; Enter → activates it
3. Alt+E → Edit menu opens directly

---

## P7 - Additional Format Support

*Lower priority. Can be partially deferred to v0.2.9 if scope is too large.*

### XML Tree Viewer

**Feature:** Open `.xml` files with syntax highlighting and hierarchical tree view.

**Requirements:**
- Detect `.xml` files and apply XML syntax highlighting in raw view
- Tree view: reuse the existing JSON/YAML tree viewer infrastructure in `src/markdown/tree_viewer.rs`
- Display element attributes as part of tree nodes (e.g. `<div class="foo">` shows as `div` with attribute `class="foo"`)
- Expand/collapse, path copying, same UX as JSON/YAML tree

**Implementation:**
- Add XML parsing dependency (e.g. `quick-xml` or `roxmltree` crate)
- Extend `tree_viewer.rs` to accept XML data as a third format
- Add `FileType::Xml` to `state.rs`

**Test:**
1. Open a `.xml` file → syntax highlighting in raw view
2. Switch to rendered/tree view → hierarchical tree with expand/collapse
3. Element with attributes → attributes shown in tree node

---

### Configuration File Support

**Feature:** Parse and display `.ini`, `.conf`, `.cfg`, `.properties`, and `.env` files.

**Requirements:**
- Syntax highlighting for INI-style files (sections, keys, values, comments)
- For `.env` files: optional secret masking toggle (show `****` for values, click to reveal)
- Tree view: sections as parent nodes, key-value pairs as children

**Implementation:**
- Simple custom parser for INI format (section headers `[section]`, key=value pairs, `#` and `;` comments)
- `.properties`: similar to INI but without sections
- `.env`: same as properties with masking option
- Extend `FileType` enum and `tree_viewer.rs`

**Test:**
1. Open `.ini` file → syntax highlighting for sections, keys, values
2. Tree view → sections expandable with key-value children
3. `.env` file with secret masking → values hidden; click to reveal

---

### Log File Viewing

**Feature:** Recognize `.log` files with level highlighting and timestamp recognition.

**Requirements:**
- Detect `.log` files and common log formats
- Color-code log levels: `ERROR` (red), `WARN` (orange/yellow), `INFO` (blue/green), `DEBUG` (gray)
- Highlight ISO timestamps and common date formats
- No tree view needed — log files are sequential/flat

**Implementation:**
- Regex-based level and timestamp detection in the syntax highlighting layer
- Custom syntect syntax definition for log files, or post-process highlights
- Add `FileType::Log` to `state.rs`

**Test:**
1. Open a `.log` file → log levels are color-coded
2. ERROR lines are visually prominent (red text or background)
3. Timestamps highlighted
4. Standard `.log` file from a Java/Python/Rust application renders correctly

---

## Deferred (Documented, Not in v0.2.8)

- **Bidirectional scroll sync** — Editor-Preview scroll sync in Split view (deferred from v0.2.7, requires deeper investigation)
- **Click-to-edit cursor drift** — Cursor offset on mixed-format lines in rendered view
- **Wrapped line scroll stuttering** — Micro-stuttering on documents with many word-wrapped lines
- **New file frontmatter templates** — Optional frontmatter templates when creating new markdown files (deferred from v0.2.7 frontmatter work)

---

## Logical Dependency Chain

Suggested order for task breakdown and implementation:

1. **Bug fixes (P1)** — Unblock users immediately; small, targeted, low risk
2. **Rendered view performance (P0)** — Highest-priority feature; AST caching first (foundation), then viewport culling (depends on cache), then block height cache (depends on culling)
3. **Strict line breaks (P2)** — Small, self-contained settings feature; warms up on Comrak/parser integration
4. **Executable code blocks (P3)** — New subsystem; security warning first, then runner, then shell, then Python, then timeout
5. **LSP Phase 1 (P5)** — Infrastructure; must be done before diagnostics/hover/completion
6. **LSP Phase 2 (P5)** — Diagnostics; highest user value among LSP features
7. **Text shaping - HarfRust (P4)** — Core shaping integration; must precede shaped cache and cursor
8. **Text shaping - cache & cursor (P4)** — Depends on HarfRust integration
9. **LSP Phase 3-4 (P5)** — Hover, go-to-def, autocomplete; depends on Phase 1-2
10. **Menu bar (P6)** — Independent feature; moderate complexity
11. **Additional format support (P7)** — Independent; can be parallelized; lowest priority

---

## Technical Architecture Notes

| Component | Location | Notes |
|-----------|----------|-------|
| Rendered view cache | `src/markdown/cache.rs` (new) or `src/markdown/editor.rs` | AST caching, block height cache |
| Rendered view culling | `src/markdown/editor.rs` | `.show_viewport()`, overscan buffer |
| Markdown parser fixes | `src/markdown/parser.rs` | Setext heading fix, strict line breaks |
| IME coordinate fix | `src/editor/ferrite/editor.rs` | `layer_transform_to_global()` |
| macOS bundle fix | `assets/macos/info_plist_ext.xml` | UTI declaration |
| Code execution | `src/code_execution/` (new) | Runner, timeout, security |
| Text shaping | `src/editor/ferrite/shaping.rs` (new) | HarfRust integration |
| Shaped cache | `src/editor/ferrite/line_cache.rs` | Extend or parallel cache |
| Grapheme cursor | `src/editor/ferrite/cursor.rs` | `unicode-segmentation` |
| LSP client | `src/lsp/` (new) | Manager, transport, state, detection |
| LSP UI | `src/app/status_bar.rs`, `src/editor/ferrite/rendering/` | Status indicator, squiggles |
| LSP settings | `src/ui/settings.rs`, `src/config/settings.rs` | Toggle, server paths |
| Menu bar | `src/ui/menu_bar.rs` (new) | Alt-key toggle, keyboard nav |
| Settings (new fields) | `src/config/settings.rs` | `strict_line_breaks`, `code_execution_*`, `lsp_*` |
| Welcome page | `src/ui/welcome.rs` | Strict line breaks toggle |
| XML viewer | `src/markdown/tree_viewer.rs` | Extend for XML format |
| Config file viewer | `src/markdown/tree_viewer.rs` | INI/properties parsing |
| Log viewer | Syntax highlighting layer | Level/timestamp detection |

---

## New Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `harfrust` | 0.5+ | Pure-Rust text shaping (HarfBuzz port) |
| `unicode-segmentation` | 1.x | Grapheme cluster boundary detection |
| `lsp-types` | 0.97+ | LSP message type definitions |
| `quick-xml` or `roxmltree` | latest | XML parsing for tree viewer |

*Note: Verify crate availability and API stability before adding. `harfrust` is relatively new — check its maturity and test coverage. For LSP transport, evaluate `tower-lsp` (client side) vs. minimal JSON-RPC + stdio wrapper.*

---

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Rendered view culling visual glitches | Overscan buffer (500px); fallback to full rendering if block count < 100 |
| HarfRust immaturity / API instability | Pin to specific version; extensive test suite; fallback to glyph-by-glyph rendering |
| LSP scope creep (4 phases is a lot) | Phase 1-2 are MVP; Phase 3-4 can slip to v0.2.9 if needed |
| Code execution security | Opt-in, first-run warning, timeout; no sandboxing (documented limitation) |
| Grapheme cursor breaking existing keybindings | Extensive testing with Latin, CJK, Arabic, Emoji; feature-flag during development |
| Menu bar conflicts with existing keyboard shortcuts | Alt-key only activates menu when no text input is focused; defer to existing shortcuts |
| XML/INI parser edge cases | Use established crates; focus on common formats first |
| Performance regression from shaping | Shaped galley cache ensures per-frame cost is minimal after first render |

---

## Testing Checklist (Pre-Release)

- [ ] P0: Rendered view smooth on 6k+ line file; AST not re-parsed on scroll
- [ ] P0: Scrollbar accurate with viewport culling; no visual glitches
- [ ] P1: macOS .md files open via Finder "Open With" and double-click
- [ ] P1: Windows IME candidate box appears at cursor position
- [ ] P1: `--` in markdown doesn't create false headings; lines don't collapse
- [ ] P2: Strict line breaks toggle works in rendered view; persists across restart
- [ ] P3: Code execution opt-in works; security dialog shown on first enable
- [ ] P3: Shell and Python blocks execute; timeout kills long scripts
- [ ] P4: Arabic text shows contextual forms in FerriteEditor
- [ ] P4: Bengali conjuncts render correctly; cursor moves by grapheme cluster
- [ ] P4: Latin text: no visual regression from shaping
- [ ] P5: LSP server detected and started; status bar shows state
- [ ] P5: Inline diagnostics (squiggles) appear for Rust/Python errors
- [ ] P5: Hover shows documentation; F12 goes to definition
- [ ] P5: Autocomplete popup works with debounce
- [ ] P6: Alt key opens menu bar; keyboard navigation works
- [ ] P7: XML tree viewer works; INI/log highlighting works
- [ ] No regressions in editor, terminal, productivity hub, mermaid diagrams
- [ ] Cross-platform smoke test (Windows, macOS, Linux)

---

## Release Notes Draft

### v0.2.8 - Performance, Accessibility, Text Shaping & Bug Fixes

**Performance:**
- Rendered view AST caching eliminates per-frame markdown re-parsing (#105)
- Rendered view viewport culling — only visible blocks are rendered (#105)
- Block-level height cache for accurate scrollbar and efficient off-screen handling

**Bug Fixes:**
- Fixed macOS .md file association for Finder "Open With" (#102)
- Fixed Windows IME candidate box appearing at wrong position (#103, #15)
- Fixed double-dash (`--`) causing false setext headings and line collapsing

**New Features:**
- Strict line breaks toggle (Obsidian-style newline-as-linebreak)
- Executable code blocks (opt-in): Run button for shell/Python with timeout
- LSP integration: inline diagnostics, hover documentation, go-to-definition, autocomplete
- Traditional menu bar via Alt key (accessibility)
- XML tree viewer, INI/config/log file support

**Unicode & Complex Scripts (Phase 2):**
- HarfRust text shaping for Arabic, Bengali, Devanagari, Tamil, and other complex scripts
- Grapheme-cluster-aware cursor movement
- Shaped text measurement for accurate word wrap and positioning

---

**Document Version:** 1.0
**Created:** March 25, 2026
**Status:** Draft — Ready for Task Master parsing
**Next Step:** Run `task-master parse-prd docs/ai-workflow/prds/prd-v0.2.8.md` to generate tasks.
