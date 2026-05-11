# Mermaid insert toolbar

The bottom **format toolbar** (Raw and Split raw pane) includes a **Mermaid…** combo box next to the table-of-contents control. Each entry dispatches `MarkdownFormatCommand::InsertMermaid(MermaidTemplateKind)` through the same deferred path as other format actions (`RibbonAction::Format` in `src/app/central_panel.rs`).

## Templates

Starter diagram bodies live in one module: `src/markdown/mermaid/templates.rs` (`MermaidTemplateKind` and `snippet_body()`). They are wrapped as fenced mermaid code blocks in `apply_mermaid_snippet_format` in `src/markdown/formatting.rs` (adds surrounding newlines when inserting mid-line).

## i18n

Labels use `format_toolbar.mermaid_*` keys in `locales/en.yaml`.
