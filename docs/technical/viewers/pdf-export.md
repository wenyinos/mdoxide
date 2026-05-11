# PDF Export

Self-contained PDF export for Markdown documents, built on the
[`krilla`](https://crates.io/crates/krilla) crate. This is the v1
implementation of the design recorded in
[`pdf-export-pipeline.md`](../planning/pdf-export-pipeline.md).

## Overview

- **Backend:** native Rust (`krilla` 0.7) — same author as the existing
  `hayro` PDF viewer, so the project ships consistent PDF tooling.
- **Inputs:** the active tab's Markdown source (parsed via `comrak`).
- **Outputs:** a single self-contained PDF file containing subsetted Inter
  and JetBrains Mono fonts and all rendered text/lines/rectangles.
- **Surface:** ribbon `Export → 📄 Export as PDF`,
  shortcut **Ctrl/Cmd + Shift + P**, and Command Palette (**Export PDF**)
  opens the options dialog before save. **Print preview** (same renderer and
  options, in-app viewer tab) is documented in [Print preview](./print-preview.md).

## Architecture

```
src/export/pdf/
├── mod.rs        ← public re-exports
├── options.rs    ← PdfExportOptions, PdfPageSize, PdfMargins, PdfMarginPreset
├── fonts.rs      ← bundled Inter + JetBrains Mono → krilla::text::Font
├── theme.rs      ← ThemeColors → PdfTheme adapter (print-friendly default)
└── render.rs     ← comrak AST → DrawOp buffer → krilla page replay
```

### Two-pass renderer

`render.rs` is a two-pass design that sidesteps krilla's `Page<'doc>` /
`Surface<'page>` lifetime nesting:

1. **Layout pass.** Walk the comrak AST top-down, lay blocks out vertically,
   wrap inline runs into visual lines, paginate when a block doesn't fit,
   and push a `DrawOp` (text / filled-rect / outline / horizontal-rule) into
   the *current* `PageBuf`.
2. **Replay pass.** In `Renderer::finish`, open one krilla `Page` per
   `PageBuf`, replay the draw ops onto its `Surface`, attach link
   annotations, and finalize the document.

This keeps the layout code free of borrow-checker contortions and makes the
draw operations cheaply cloneable / inspectable.

### Fonts

Inter (proportional) and JetBrains Mono (monospace) are reused from the
same `include_bytes!` slices that `src/fonts.rs` already loads for the egui
UI. krilla subsets them on export, so the on-disk PDF only carries the
glyphs that the document actually used.

### Theme

`PdfTheme::print_default` is used by default — a clean white page with
near‑black text, intended for printing. Users can opt in to the active
editor theme via the **Use the active editor theme** checkbox, which adapts
backgrounds, headings, links, and code colors via
`PdfTheme::from_theme_colors`.

## Configuration

`PdfExportOptions` is persisted to `Settings::pdf_export_options`:

| Field                  | Type                | Default                    |
| ---------------------- | ------------------- | -------------------------- |
| `page_size`            | `PdfPageSize`       | locale‑derived (A4 vs Letter) |
| `margin_preset`        | `PdfMarginPreset`   | `Comfortable` (~19 mm)     |
| `custom_margins`       | `PdfMargins`        | comfortable preset         |
| `page_break_before_h1` | `bool`              | `false`                    |
| `use_theme_colors`     | `bool`              | `false`                    |
| `include_page_numbers` | `bool`              | `true`                     |
| `open_after_export`    | `bool`              | `false`                    |

The export options dialog (`render_pdf_export_dialog`) is rendered as a
modal `egui::Window` from `src/app/dialogs.rs`. Edits persist to
`Settings` even if the user cancels, so a user who only wanted to tweak
defaults is not punished.

## Supported content

- Headings H1–H6 (with H1/H2 underline rule)
- Paragraphs with bold / italic / strike-through inline runs
- Bullet, ordered, and task lists (with `☑` / `☐` markers)
- Block quotes (with left border bar that follows page breaks)
- Code blocks (mono font in a tinted panel; **no** syntax colors yet)
- Inline `code`
- Tables (equal‑width columns, header tint, cell borders)
- Horizontal rules
- Hyperlinks (PDF link annotations point to the URL)
- Images: rendered as italic placeholders for v1
- Page numbering footer

## Known v1 limitations

These are explicitly **deferred** to follow-up work, all called out in the
design doc roadmap:

- **Mermaid diagrams** are rendered as plain code blocks. The SVG-emit path
  (krilla-svg) is roadmap Phase 4.
- **Syntect-colored code blocks** are monochrome.
- **Raster image embedding** is a follow-up; only placeholders today.
- **Bidi reordering / shaping** for RTL scripts is not applied — Latin /
  CJK in the bundled fonts will render correctly, but Arabic / Hebrew may
  render LTR.
- **Text wrapping** uses approximate per-character widths (0.5 × font size
  for Inter, 0.6 × for JetBrains Mono). Real shaping via rustybuzz is
  planned.
- **Multi-page block quotes** only get the left bar drawn correctly when
  start and end fit on the same page (the renderer estimates intermediate
  pages).
- Color emoji is not painted; the bundled fonts only include monochrome
  glyphs.

## Error handling

- A toast is shown for every failure (no document, render error, write
  error).
- If the rendered PDF cannot be written to disk, any partially-written file
  at the target path is removed via `std::fs::remove_file` so the user is
  not left with a corrupt file.
- Last-used directory is persisted separately (`last_pdf_export_directory`)
  and falls back to `last_export_directory` then to the recent-files list.

## Tests

`src/export/pdf/{options,fonts,render,theme}.rs` each carry unit tests.
The renderer test loads the bundled fonts, renders a small document, and
checks that the result starts with `%PDF-` and is non-trivial in size.
The H1-page-break test verifies the option actually causes a longer
document.

## Related

- [PDF Export Pipeline (decision doc)](../planning/pdf-export-pipeline.md)
- [PDF Viewer](./pdf-viewer.md) — read-only side
- [Document Export (HTML)](./document-export.md) — sibling export pathway
