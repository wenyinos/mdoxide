# Print preview

In-app paginated preview of Markdown as PDF using the **same** pipeline as **[PDF export](./pdf-export.md)** (`render_markdown_to_pdf`, `Settings::pdf_export_options`), so layout matches a saved PDF.

## User-facing surface

| Entry | Detail |
|--------|--------|
| Ribbon | `Export → 🖨 Print preview…` (Markdown ribbon only) |
| Shortcut | Default **Ctrl/Cmd + Alt + P** (`ShortcutCommand::PrintPreview`) |
| Command palette | **Print preview** |
| i18n | `menu.file.print_preview`, `ribbon.print_preview`, `shortcuts.commands.print_preview`, `print_preview.tab_title` / `untitled` |

## Implementation

1. **`FerriteApp::handle_print_preview`** ([`src/app/export.rs`](../../../src/app/export.rs)) resolves `PdfTheme` like export (`use_theme_colors` vs `PdfTheme::print_default`), calls `render_markdown_to_pdf` with `base_dir` from the active tab path parent, writes bytes to **`%TEMP%/ferrite-print-preview-<nanos>.pdf`**.
2. **`AppState::open_pdf_tab`** opens a normal **`TabKind::PdfViewer`** tab; the handler then sets [`PdfViewerState`](../../../src/state.rs) **`display_title`** (e.g. `doc.md — Print preview`) and **`ephemeral_temp_file = true`**.
3. **Lifecycle:** `ephemeral_temp_file` skips the tab in **`capture_session_state`** and **`force_close_tab`** deletes the temp path (best-effort `remove_file`).
4. **Rendering:** Existing hayro PDF viewer (`central_panel.rs` reads the file path per page) — no second PDF layout implementation.

## See also

- [PDF Export](./pdf-export.md) — options dialog, krilla renderer, `PdfExportOptions`
- [PDF Viewer](./pdf-viewer.md) — viewer tab UX (pages, zoom)
