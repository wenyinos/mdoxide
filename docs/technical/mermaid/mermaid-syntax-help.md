# Mermaid syntax help (About / F1)

Help text for Mermaid lives in **About / Help** (F1) under the **Mermaid** sidebar tab.

## Implementation

- UI: `src/ui/about.rs` — `AboutSection::Mermaid`, `show_mermaid_section`.
- Snippet source of truth: `src/markdown/mermaid/templates.rs` — `MermaidTemplateKind::snippet_body`, `snippet_fenced_block`, `mermaid_kind_menu_label` (same strings as **Insert → Mermaid…** in `src/ui/format_toolbar.rs`).
- Insert behavior: `src/markdown/formatting.rs` — `apply_mermaid_snippet_format` uses `snippet_fenced_block`.
- Copy: `locales/en.yaml` under `about.mermaid` and `about.tab.mermaid`.

External documentation link opens `https://mermaid.js.org/`.
