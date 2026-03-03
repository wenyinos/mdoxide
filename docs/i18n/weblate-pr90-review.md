# Weblate PR 90 Review – Regression Check

**PR:** [OlaProeis/Ferrite#90](https://github.com/OlaProeis/Ferrite/pull/90) – Translations update from Hosted Weblate  
**Reviewed:** 2026-02-26  
**Context:** Previous cleanup fixed duplicate YAML sections, wrong key structure, and ordering in locale files (see `docs/i18n/i18n-sync-report.md`).

---

## Summary

**Do not merge PR 90 as-is.** It reintroduces structural problems that were fixed earlier: keys are placed under the wrong parent sections in `zh_Hans.yaml`, so the app would look up e.g. `about.title` but find only `find.about.title` or `settings.about.title`. There are also minor typos and one broken line-wrap in `de.yaml`.

---

## 1. Critical: Wrong key structure in `zh_Hans.yaml`

rust-i18n expects keys to match `en.yaml` exactly (e.g. `about.title`, `terminal.title`, `stats.summary`, `widgets.list.decrease_level`). In PR 90, many new keys are added under the wrong parent, so the paths no longer match.

### 1.1 `about` and `terminal` nested under `settings`

- **In PR:** Keys are added as `settings.about.*` and `settings.terminal.*` (e.g. after `settings.editor.paragraph_indent_custom_desc`).
- **In en.yaml:** `about` and `terminal` are top-level sections (e.g. `about.title`, `terminal.rename`).
- **Effect:** Lookups for `t!("about.title")` and `t!("terminal.rename")` will not find these; they are under `settings.*`.

### 1.2 `about` and `terminal` nested under `find`

- **In PR:** A block with `about:` and `terminal:` is added under the Find panel section (`find`).
- **Effect:** Creates `find.about.*` and `find.terminal.*` instead of top-level `about.*` and `terminal.*`. Duplicate wrong placement of the same sections.

### 1.3 `stats` keys placed under `outline`

- **In PR:** Keys such as `summary`, `detach_tooltip`, `backlinks_unavailable`, `productivity_unavailable`, `tab_links`, `tab_hub`, and the `json_*` keys are added under `outline`.
- **In en.yaml:** These live under `stats` (e.g. `stats.summary`, `stats.detach_tooltip`, `stats.backlinks_unavailable`).
- **Effect:** `t!("stats.summary")` etc. will not resolve; they exist only as `outline.*`.

### 1.4 `widgets.list` placed under `csv` (or as top-level)

- **In PR:** A `list:` block with `decrease_level`, `increase_level`, `remove_item`, `add_item` is added after `# CSV Viewer` with 2-space indent, so it is either under `csv` or a top-level `list`.
- **In en.yaml:** These are `widgets.list.*` (under `widgets`).
- **Effect:** `t!("widgets.list.decrease_level")` will not find the new entries.

---

## 2. de.yaml – Typos and formatting

Existing CodeRabbit review comments on PR 90:

- **Line ~192 (`auto_save_tooltip`):** Typo `temp-Dateinen` → should be `temporären Dateien` (or at least `temp-Dateien`).
- **Line ~610 (`pipeline.hint`):** Typo `durchgelitet` → `durchgeleitet`.
- **Line ~541 zh_Hans (`nothing_to_redo`):** Should say “没有可重做的操作” (redo) not “没有可撤销的操作” (undo).

In addition:

- **`pipeline.hint` in de.yaml:** The multi-line value in the PR breaks the word “extrahieren” across lines (“JSON-Array-Inhalt” on one line and “extrahieren” on the next). That should be one continuous line or a proper YAML multi-line block so the sentence and word are not split.

---

## 3. What is fine in PR 90

- **de.yaml:** New notification keys (`zen_enabled`, `fullscreen_exit`, etc.) are correctly added under `notification:` (same section as in en.yaml). New top-level `welcome` and `terminal` sections at the end match en.yaml.
- **zh_Hans.yaml:** New `status` keys (e.g. `vim_mode`, `cursor_position`, `file_tooltip`), the large block of `notification` keys, and the new `widgets.table` and `welcome` / `terminal` content are in the correct sections. Line-wrapping in de.yaml for long strings is generally acceptable if the value stays one logical string.

---

## 4. Recommendations

1. **Do not merge PR 90** until the structural issues in `zh_Hans.yaml` are fixed.
2. **Fix zh_Hans.yaml in Weblate or in a follow-up PR:**
   - Move every `about.*` and `terminal.*` key out of `settings` and `find` into top-level `about:` and `terminal:` with the same key paths as in `en.yaml`.
   - Move `summary`, `detach_tooltip`, `backlinks_unavailable`, `productivity_unavailable`, `tab_links`, `tab_hub`, and all `json_*` keys from under `outline` to under `stats`.
   - Move the `list` block (decrease_level, increase_level, remove_item, add_item) under `widgets`, so they are `widgets.list.*`.
3. **Fix de.yaml:** Apply the CodeRabbit typo fixes and fix the `pipeline.hint` line break so “extrahieren” is not split.
4. **Weblate configuration:** Ensure the Weblate component uses `en.yaml` as the single source of truth and that translations are committed under the same top-level keys and section order as in `en.yaml`, so future PRs do not reintroduce wrong nesting or duplicate sections.

---

## 5. Reference: Previous cleanup (i18n-sync-report.md)

- Removed duplicate YAML section declarations (e.g. status, find, workspace, csv, widgets, mermaid, pipeline, tooltip, ribbon declared twice in zh_Hans).
- Consolidated “additions” blocks at the bottom into the canonical structure.
- Enforced identical key set (388 keys) and same key ordering across locale files.
- Removed orphaned/old key names in et.yaml and nb_NO.yaml.

PR 90’s structural issues are the same kind of problem: keys under the wrong parent and/or wrong section order, which breaks `t!("key.path")` lookups.

---

## 6. How we fixed it (2026-02-26)

Instead of merging the Weblate PR as-is or losing the new translations:

1. **Do not merge PR 90** — keep it closed.
2. **Applied the new translations on top of current `master`** in this repo:
   - **de.yaml:** Added the missing `notification.*` block (all keys from PR), fixed typos (`temp-Dateinen` → `temporären Dateien`, `durchgelitet` → `durchgeleitet`), added `widgets.table` tooltips and `welcome` / `terminal` sections.
   - **zh_Hans.yaml:** Added missing `status.*` keys, `settings.editor.vim_mode*`, `settings.about.*` and `settings.terminal.*` under `settings`, `stats.*` keys (summary, detach_tooltip, json_*, etc.), full `notification.*` block, `widgets.table` and `widgets.list` keys, fixed `nothing_to_redo` to "没有可重做的操作", and added `welcome` / `terminal` sections.
3. **Result:** All new Weblate content is preserved and key paths match `en.yaml`.

**Next steps:** Commit the updated `locales/de.yaml` and `locales/zh_Hans.yaml`, then push to `master` or open a PR. Close PR 90 with a note that translations were integrated via this fix. After merging, Weblate will pull the corrected structure from `master`.

---

## 7. Why we can't just follow what Weblate produces

The app expects **fixed key paths**. Every string is looked up with `t!("key.path")`, for example:

- `t!("about.title")` → About dialog title  
- `t!("notification.zen_enabled")` → toast text  
- `t!("stats.summary")` → stats panel  
- `t!("widgets.list.decrease_level")` → list widget  

If Weblate writes the same content under different paths (e.g. `find.about.title` or `outline.summary`), those lookups return nothing and the UI shows missing or fallback text. So we **cannot** change the app to “follow what Weblate produces” when that structure is wrong; the code is written against the structure defined in `en.yaml`. The only robust approach is to **make Weblate produce files that match `en.yaml`’s structure**.

---

## 8. Avoiding this next time: configure Weblate to follow en.yaml

**Yes — the next Weblate push will recreate the same issue unless the Weblate component is configured so that the English file is the single source of structure.**

Do this in the **Ferrite UI** component on [hosted.weblate.org](https://hosted.weblate.org/projects/ferrite/ferrite-ui/):

1. **Component configuration** → **Manage** (or **Settings**).
2. Set **Monolingual base language file** (or **Template for new translations**) to **`locales/en.yaml`**.
   - This makes Weblate use `en.yaml` as the template: same keys, same nesting, same order. New languages and new keys are created by copying this structure; translators only fill in values.
3. Set **File mask** to match locale files, e.g. **`locales/*.yaml`** (or whatever the project uses), and ensure the base language is **English** and points at **`locales/en.yaml`**.
4. When **new keys** are added to the repo, they should be added first to **`en.yaml`** in the main repo, then merged to `master`. Weblate then pulls and sees the new keys in the template; it will add them in the **same place** in each translation file. Translators fill them in; no structural drift.

If the component was set up **without** a monolingual base (e.g. each language file treated independently), Weblate can end up appending or placing new keys in the wrong sections, which is what happened in PR 90.

**After changing the component:**

- Merge the corrected `locales/de.yaml` and `locales/zh_Hans.yaml` to `master` (as in section 6).
- Weblate pulls from `master` and now has the right structure.
- Future commits from Weblate should only change **values**, not key paths or section order.

**Check after the next Weblate PR:** Confirm that only **values** changed and that no new top-level or nested sections appear in the wrong place (e.g. no `find.about`, no `outline.summary`). If the component is set up correctly, this should hold.
