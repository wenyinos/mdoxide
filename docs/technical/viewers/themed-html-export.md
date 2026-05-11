# Themed HTML export

Standalone HTML export that tracks the editor theme, optional `prefers-color-scheme` (Auto), accent-aware CSS, fenced-code highlighting via syntect through comrak, and flowchart Mermaid as inline SVG.

## User-facing behavior

- **Dialog:** Settings are persisted in `Settings.html_export_options` (`src/export/html_options.rs`). UI: `render_html_export_dialog` in `src/app/dialogs.rs`, toggled by `UiState.show_html_export_dialog` (shortcut opens the dialog; export runs from there).
- **Self-contained vs linked:** Controlled by `HtmlExportOptions.self_contained`, `ImageHandling`, and post-processing in `src/export/html.rs`.
- **Clipboard:** `Copy as HTML` continues to use the fragment/clipboard path in `src/export/clipboard.rs` (unchanged contract).

## Implementation map

| Concern | Location |
|--------|----------|
| Full document assembly | `generate_html_document_export`, `generate_html_document` (`src/export/html.rs`) |
| Theme resolution (single / auto light+dark) | `HtmlThemeResolution`, `resolve_html_theme_for_export`, `theme_rules_inner` (`html.rs`) |
| Syntax highlighting in fences | `FerriteHtmlHighlighter` — `SyntaxHighlighterAdapter`, syntect `append_highlighted_html_for_styled_line` into a `String` then write (`html.rs`) |
| Mermaid | `extract_mermaid_fences`, `inject_mermaid_exports`; flowchart SVG `try_flowchart_svg_snippet` (`src/export/flowchart_svg.rs`) |
| Flowchart types for SVG | Re-exported from `src/markdown/mermaid/mod.rs`; diagram kind `detect_mermaid_diagram_type` from `src/markdown/mod.rs` |
| Images / relative links | `postprocess_images`, `postprocess_links` (`html.rs`); shared enums in `src/export/options.rs` |
| PDF sibling (theme colors) | `PdfTheme::from_theme_colors` — see [`pdf-export.md`](./pdf-export.md) |

## Related

- End-user and clipboard overview: [`document-export.md`](./document-export.md)
