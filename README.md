# MDOxide

**English** | [简体中文](README.zh-CN.md)

A native WYSIWYG Markdown editor written in Rust, built with `egui`/`eframe` — no WebView, no Electron, no JavaScript runtime.

MDOxide reimplements [MarkText](https://github.com/marktext/marktext)'s editing experience in Rust. It started from an MIT-licensed egui editor; full attribution is in [`NOTICE`](NOTICE).

The name: **Oxide** (Rust's oxidized-iron identity) + **MD** (Markdown).

## Status

**v0.8.0 released.** All six milestones are complete. This release ports the usable parts of upstream Ferrite v0.3.1: HTML `<details>` and aligned blocks, Mermaid popup/gitGraph/autonumber, rendered block navigation, custom editor/rendered/terminal fonts with Nerd Font fallback, settings search, file-tree polish, native title bar and Linux primary-selection paste. Release artifacts are built by GitHub Actions for Linux x86_64/aarch64 and Windows x64/ARM64.

## What works today

- WYSIWYG markdown editing (per-block rendered/editing switch, commit-boundary undo)
- Raw / Rendered / Split view modes
- MarkText-style chrome: 32px frameless title bar with hamburger application menu, quick view-mode switch, focus and settings shortcuts, centred document path, 28px tab strip with accent underline, selection format toolbar, typewriter mode
- Text context menus in Raw and Rendered modes: copy / cut / paste / select all, plus "Copy All (Plain Text)" and "Copy as Markdown" in preview
- Focus mode dims inactive blocks; Zen mode hides the sidebar and status bar but keeps the tab strip visible
- Typography tuned to MarkText: 15px body text at 1.6× line height; table cells wrap and grow to fit their content
- Bundled Source Han Sans for reliable CJK rendering; fcitx5 IME verified on Wayland (preedit, candidate window, backspace, commit, Esc)
- CommonMark + GFM via `comrak`; syntax highlighting via `syntect`
- Native Mermaid rendering (11 diagram types) and Mermaid SVG in HTML export
- RaTeX math rendering in the editor and in HTML/PDF exports
- HTML and PDF export with embedded CJK fonts, tables, code blocks, links, and formula images
- File tree, workspace mode, find & replace (regex)
- Revision-driven editor/cache invalidation; one external-change entry point
- Single `Document` owner for body, path, encoding and undo history (`Tab` delegates to it)
- Rich Markdown blocks: `<details>/<summary>`, aligned `<div>`, `kbd`/`sup`/`sub`, click-to-open Mermaid popup, full gitGraph attributes
- Rendered-view keyboard navigation, preview lock and frontmatter property cards
- Independent editor/rendered/terminal fonts, including custom system fonts and Nerd Font glyph fallback
- Per-row settings search, runtime stats (fonts/Mermaid cache/terminal), native title bar option and Linux primary-selection paste
- Raw edits are recorded as operations (no per-keystroke full-document conversion or diff)
- Durable atomic writes for config, session and recovery files; previous config kept as backup
- Crash-recovery snapshots are serialized on a background writer; workspace indexing is debounced and single-worker
- Code execution capped at 2 MB with process-tree termination; HTML active content is opt-in; terminal layout auto-load is off by default
- 10 UI languages (1006 aligned translation keys)

## Known limitations

- PDF v1: Mermaid blocks render as code blocks, images are placeholders, and bidi reordering is not applied
- Preview text selection is per block; cross-block selection is not supported (use "Copy All" for the whole document)
- macOS is not supported

## Build

Requirements: stable Rust (specified in `rust-toolchain.toml`), plus the usual Linux GUI dev packages (`gtk3-devel`, `libxkbcommon-devel`, `wayland-devel` on Fedora).

```bash
cargo run --release        # run
cargo test --release       # full test suite
cargo build --release      # binary at target/release/mdoxide
```

## License

MIT. MDOxide reimplements the UX of MarkText and contains code derived from an MIT-licensed upstream project; see [`NOTICE`](NOTICE) for attribution details.
