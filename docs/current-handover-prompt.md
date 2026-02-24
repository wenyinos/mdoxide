# Handover: i18n Empty-String Bug (All Non-EN Locales) + List Wrapping Bug

## Rules (DO NOT UPDATE)
- Never auto-update this file - only update when explicitly requested
- Run `cargo build` after changes to verify code compiles
- Follow existing code patterns and conventions
- Use Context7 MCP tool to fetch library documentation when needed
- Document by feature (e.g., memory-optimization.md), not by task
- Update docs/index.md when adding new documentation
- **Branch**: `master`

---

## Current Task (Priority): Remove Empty-String Translations from All Locale Files

**All non-English locale files contain empty-string values (`""`) that display as blank text instead of falling back to English.**
- **Priority**: High (user-facing — UI shows blank labels/text in German, Chinese, and others)
- **Status**: In progress — Japanese is fixed, all others still broken
- **Repro**: Settings → Language → Deutsch (or 简体中文); then open Settings, Outline sidebar.

### The Bug

UI elements show **blank/empty text** instead of translated or English fallback strings. This affects German (de), Chinese (zh_Hans), Spanish (es), Estonian (et), Norwegian (nb_NO), and Portuguese (pt).

### Root Cause (Confirmed)

`rust_i18n`'s `fallback = "en"` only triggers when a key is **completely absent** from a locale file. An empty string `""` is treated as a **valid translation** — it displays nothing.

In the previous session, missing keys were added to locale files with `""` as placeholder values. This backfired: instead of falling back to English, the UI now shows blank text.

**Empty string counts per file:**

| File | Empty `""` values | Status |
|------|-------------------|--------|
| `locales/ja.yaml` | **0** | **Working** (fully translated, no empty strings) |
| `locales/de.yaml` | 246 | **Broken** |
| `locales/zh_Hans.yaml` | 282 | **Broken** |
| `locales/es.yaml` | 553 | **Broken** |
| `locales/nb_NO.yaml` | 606 | **Broken** |
| `locales/et.yaml` | 727 | **Broken** |
| `locales/pt.yaml` | 717 | **Broken** |

### How i18n Works Here

- **Crate**: `rust_i18n`; macro `t!("key.path")`; keys live in `locales/*.yaml`.
- **Loading**: Compile-time (`rust_i18n::i18n!("locales", fallback = "en");` in `src/main.rs`). **Full rebuild required** after YAML edits.
- **Fallback**: Added in previous session. When a key is **missing** from a locale, it falls back to `en.yaml`. But empty strings `""` are **not** missing — they are valid values that show nothing.
- **Key rule**: If a translation doesn't exist, the key must be **omitted entirely** (not present in the file), so the fallback mechanism works.

### The Fix

**For every non-English locale file**, remove all lines where the value is an empty string `""`. Keep the YAML structure valid (parent keys with children stay, but leaf keys with `""` values must go).

**Approach** (in order of priority — fix de and zh_Hans first as user-reported):

1. **`locales/de.yaml`** — Remove all `key: ""` lines. This file has real German translations for most keys; the 246 empty strings are newer keys added as placeholders. Simply deleting those lines will make them fall back to English.

2. **`locales/zh_Hans.yaml`** — Same approach: remove all `key: ""` lines. The 282 empty strings are placeholders; existing Chinese translations should be preserved.

3. **Remaining files** (`es.yaml`, `et.yaml`, `nb_NO.yaml`, `pt.yaml`) — Same pattern. These have even more empty strings (553–727) because they had fewer translations to begin with.

**Important rules for the fix:**
- **Do NOT edit `locales/en.yaml`** — it is the reference and must not be changed.
- **Do NOT edit `locales/ja.yaml`** — it is working correctly with zero empty strings.
- **Do NOT replace empty strings with English text** — just delete the entire line. The fallback mechanism handles it.
- **Keep parent keys** that still have non-empty children. Only remove leaf-level `key: ""` entries.
- **Preserve all existing translations** — only remove lines where the value is literally `""`.
- After removing empty strings, verify that all remaining values are **double-quoted** strings. Unquoted non-ASCII values can cause YAML parsing failures.

### Verification

After editing each file:
1. Run `cargo build` to recompile (locales load at compile time).
2. Run the app, switch to the language, and check:
   - Settings nav labels (Terminal, About) show English fallback or translated text (not blank).
   - Outline tabs (Links, Hub) show text (not blank).
   - No raw keys visible (like `settings.terminal.title`).

### Key Files

| File | Purpose |
|------|---------|
| `locales/de.yaml` | German — 246 empty strings to remove |
| `locales/zh_Hans.yaml` | Chinese — 282 empty strings to remove |
| `locales/es.yaml` | Spanish — 553 empty strings to remove |
| `locales/et.yaml` | Estonian — 727 empty strings to remove |
| `locales/nb_NO.yaml` | Norwegian — 606 empty strings to remove |
| `locales/pt.yaml` | Portuguese — 717 empty strings to remove |
| `locales/en.yaml` | **Reference — DO NOT EDIT** |
| `locales/ja.yaml` | **Working — DO NOT EDIT** |
| `src/main.rs` | Has `fallback = "en"` already configured (line 30) |

### What Was Already Done (Previous Session)

1. **`src/main.rs`**: Added `fallback = "en"` to `rust_i18n::i18n!()` macro — this is the single most important fix enabling graceful degradation.
2. **`locales/ja.yaml`**: Complete rewrite — all values have actual Japanese translations, no empty strings. **This is why Japanese works.**
3. **`locales/de.yaml`**: Rewritten to match `en.yaml` structure, but new/untranslated keys were set to `""` instead of being omitted — **this is why German is still broken**.
4. **Other locale files**: Missing sections were added with `""` placeholders — same problem.

### Lesson Learned

**Never use empty strings as translation placeholders.** If a translation doesn't exist, omit the key entirely so `rust_i18n`'s fallback mechanism provides the English string. Empty strings are valid translations that show nothing.

---

## Also Pending: Preview List Item Text Wrapping Bug

**Fix Preview Text Wrapping — List Items Break All Wrapping**
- **Priority**: Critical
- **Status**: Not started
- **GitHub Issue**: [#82](https://github.com/OlaProeis/Ferrite/issues/82)
- **Test file**: `test_md/test_table_wrapping.md` (or create a new list-focused test file)

### The Bug

In the split/preview markdown editor, list items break text wrapping for themselves and all subsequent content. The `max_line_width` setting is also ignored once a list item is present.

### Observed Behavior (4 stages)

1. **Plain text only** — wraps correctly in both editor and preview. No issues.
2. **Add a `-` to start a list (no content yet)** — the paragraph text ABOVE the list marker suddenly renders as a heading/bold in the preview, and text wrapping breaks entirely in the preview pane.
3. **Type short content for the list item** — preview partially recovers; looks okay with short text.
4. **Long text in the list item** — the list item itself refuses to wrap, and ALL content after it also stops wrapping. The `max_line_width` setting is completely ignored.

### Root Cause

List item rendering uses `ui.horizontal()` so the layout expands to fit content and does not respect parent width. Same pattern as the table bug that was fixed by capturing `ui.available_width()` and constraining the inner widget.

### Where the Bug Lives

Two code paths in `src/markdown/editor.rs`:
1. **`render_list_item_with_structural_keys()`** — ~line 2145, ~2256 `ui.horizontal()` + `TextEdit::multiline` with `desired_width(f32::INFINITY)`
2. **`render_list_item()`** — ~line 3724, same pattern

Both need the same fix: capture available width before horizontal, pass constrained width to TextEdit. See table fix in `render_table()` for reference. Formatted list display path ~2687 (`horizontal_wrapped`) may also need verification.

### Key Files (list wrapping)

| File | Purpose |
|------|---------|
| `src/markdown/editor.rs` | `render_list_item_with_structural_keys()` ~2145–2400, `render_list_item()` ~3724–3900, list containers, formatted display ~2687 |

---

## Recently Completed

- **Japanese i18n fix** — Full rewrite of `ja.yaml` with all values translated and properly quoted; zero empty strings
- **Fallback locale** — Added `fallback = "en"` to `rust_i18n::i18n!()` in `src/main.rs`
- **Locale structure alignment** — All locale files now match `en.yaml` key structure (but with empty string problem)
- **Table background & layout fix** — Shape::Noop placeholder technique for backgrounds, `top_down` cell layout, removed max-height constraint
- **Column resizing** — Draggable column separators with proportional width persistence
- **Table wrapping foundation** — manual layout with proportional column widths, text wrapping via `TextEdit::multiline` + `LayoutJob`

## Previously Completed

- **Task 29**: Always show view mode bar for all editor tabs (DONE)
- **Task 26**: Windows MSI installer overhaul (DONE)
- **Task 20 + 21**: Vim mode settings toggle, status bar indicator (DONE)
- **Task 19**: Lazy CSV row parsing with byte-offset indexing (DONE)
- **Task 30**: Light mode text readability fix (DONE)

---

## Environment

- **Project**: Ferrite (Markdown editor)
- **Language**: Rust
- **GUI Framework**: egui 0.28
- **Branch**: `master`
- **Build**: `cargo build`
- **Version**: v0.2.7 (in progress)
