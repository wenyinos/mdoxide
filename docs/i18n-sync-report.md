# I18n Sync Report

Generated: 2026-01-20

## Summary

All locale files have been synchronized to match `en.yaml` structure (388 keys).

---

## de.yaml

- **Keys added:** 0
- **Keys removed:** 0

**Status:** Already in sync. File structure matches `en.yaml` exactly.

---

## ja.yaml

- **Keys added:** 0
- **Keys removed:** 0

**Status:** Already in sync. File structure matches `en.yaml` exactly.

---

## zh_Hans.yaml

- **Keys added:** 0 (all keys already existed, but some were in duplicate "additions" sections)
- **Keys removed:** 0 (duplicate section declarations were consolidated)

**Restructuring performed:**
The file had structural issues with duplicate YAML sections appended at the bottom ("additions" sections). These have been consolidated into the proper structure:

1. **Removed duplicate section declarations:**
   - `status` section (was declared twice)
   - `find` section (was declared twice)
   - `workspace` section (was declared twice)
   - `csv` section (was declared twice)
   - `widgets` section (was declared twice)
   - `mermaid` section (was declared twice)
   - `pipeline` section (was declared twice)
   - `tooltip` section (was declared twice)
   - `ribbon` section (was declared twice)

2. **Preserved all existing translations** - No translation values were lost

3. **Keys with empty values "" (need translation):**
   - `status.untitled`
   - `status.no_file`
   - `find.title`
   - `find.title_replace`
   - `find.title_find`
   - `find.close_tooltip`
   - `find.hide_replace`
   - `find.show_replace`
   - `find.prev_tooltip`
   - `find.next_tooltip`
   - `find.replace_tooltip`
   - `find.replace_all_tooltip`
   - `find.keyboard_hints`
   - `notification.restored_auto_save`
   - `notification.auto_save_discarded`
   - `notification.session_restored`
   - `tree_viewer.show_raw`
   - `tree_viewer.parse_error`
   - `git.tracked`
   - `git.staged_modified`
   - `git.ignored`
   - `git.conflict`
   - `recovery.auto_save.title`
   - `recovery.auto_save.backup_found`
   - `recovery.auto_save.restore_question`
   - `recovery.auto_save.restore`
   - `recovery.auto_save.discard`
   - `recovery.session.title`
   - `recovery.session.crash_detected`
   - `recovery.session.restore_question`
   - `recovery.session.restore`
   - `recovery.session.start_fresh`
   - `recovery.session.restore_tooltip`
   - `recovery.session.start_fresh_tooltip`
   - `recovery.untitled`
   - `common.ok`
   - `common.copy`
   - `common.error`
   - `common.dismiss`
   - `widgets.code_block.copy_tooltip`
   - `widgets.code_block.finish_tooltip`
   - `widgets.code_block.edit_tooltip`
   - `widgets.table.add_row`
   - `widgets.table.add_column`
   - `widgets.table.align_left`
   - `widgets.table.align_center`
   - `widgets.table.align_right`
   - `widgets.table.delete_column_label`
   - `widgets.table.align_label`
   - `widgets.table.align_none`
   - `widgets.link.edit`
   - `widgets.link.open`
   - `widgets.link.copy`
   - `widgets.link.text_label`
   - `widgets.link.url_label`
   - `widgets.link.copy_tooltip`
   - `csv.delimiter_auto`
   - `csv.error`
   - `csv.select_delimiter`
   - `csv.header_row`
   - `csv.has_headers_yes`
   - `csv.has_headers_no`
   - `csv.show_raw`
   - `pipeline.title`
   - `pipeline.command_placeholder`
   - `pipeline.run`
   - `pipeline.recent`
   - `pipeline.no_output`
   - `pipeline.running`
   - `pipeline.truncated`
   - `pipeline.close_tooltip`
   - `pipeline.cancel_tooltip`
   - `pipeline.run_tooltip`
   - `pipeline.stdout`
   - `pipeline.stderr`
   - `pipeline.hint`
   - `pipeline.no_output_success`
   - `tab.reveal_in_explorer`
   - `mermaid.badge`
   - `mermaid.empty`
   - `workspace.close_folder`
   - `workspace.new_file`
   - `workspace.new_folder`
   - `workspace.rename`
   - `workspace.delete`
   - `workspace.refresh`
   - `workspace.recent_folders`
   - `zen.enter`
   - `zen.exit`
   - `tooltip.fullscreen_exit`
   - `tooltip.fullscreen_enter`
   - `tooltip.settings`
   - `tooltip.recent_items`
   - `tooltip.about_help`
   - `tooltip.git_branch`
   - `tooltip.new_tab`
   - `ribbon.format_document`
   - `ribbon.validate_syntax`
   - `ribbon.pipeline`
   - `ribbon.hide_info_panel`
   - `ribbon.show_info_panel`
   - `ribbon.toggle_outline`
   - `ribbon.copy_html_tooltip`
   - `ribbon.export_pdf`
   - `ribbon.coming_soon`

**Status:** Restructured successfully. All 388 keys present with proper ordering.

---

## Status: All locale files are now in sync

All four locale files (`en.yaml`, `de.yaml`, `ja.yaml`, `zh_Hans.yaml`) now have:
- Identical key structure (388 keys)
- Same key ordering
- Valid YAML syntax (no duplicate sections)

Empty string values `""` indicate keys that need translation in Weblate.
