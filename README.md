# MDOxide

**English** | [简体中文](README.zh-CN.md)

A native WYSIWYG Markdown editor written in Rust, built with `egui`/`eframe` — no WebView, no Electron, no JavaScript runtime.

MDOxide reimplements [MarkText](https://github.com/marktext/marktext)'s editing experience in Rust. It started from an MIT-licensed egui editor; full attribution is in [`NOTICE`](NOTICE).

The name: **Oxide** (Rust's oxidized-iron identity) + **MD** (Markdown).

## Status

**v0.3.2 released.** All six milestones are complete: baseline fork and audit, module reduction, MarkText visual identity, interaction parity (including a real fcitx5 IME walkthrough), export quality, and packaging. Release artifacts are built by GitHub Actions for Linux x86_64/aarch64 and Windows x64/ARM64.

## What works today

- WYSIWYG markdown editing (per-block rendered/editing switch, commit-boundary undo)
- Raw / Rendered / Split view modes
- MarkText-style chrome: 32px frameless title bar with hamburger application menu, quick view-mode switch, focus and settings shortcuts, centred document path, 28px tab strip with accent underline, selection format toolbar, typewriter mode
- Focus mode dims inactive blocks; Zen mode hides the sidebar and status bar but keeps the tab strip visible
- Typography tuned to MarkText: 15px body text at 1.6× line height; table cells wrap and grow to fit their content
- Bundled Source Han Sans for reliable CJK rendering; fcitx5 IME verified on Wayland (preedit, candidate window, backspace, commit, Esc)
- CommonMark + GFM via `comrak`; syntax highlighting via `syntect`
- Native Mermaid rendering (11 diagram types) and Mermaid SVG in HTML export
- RaTeX math rendering in the editor and in HTML/PDF exports
- HTML and PDF export with embedded CJK fonts, tables, code blocks, links, and formula images
- File tree, workspace mode, find & replace (regex)
- 10 UI languages (996 aligned translation keys)

## Known limitations

- PDF v1: Mermaid blocks render as code blocks, images are placeholders, and bidi reordering is not applied
- macOS is not supported

## Build

Requirements: Rust 1.92 (pinned in `rust-toolchain.toml`), plus the usual Linux GUI dev packages (`gtk3-devel`, `libxkbcommon-devel`, `wayland-devel` on Fedora).

```bash
cargo run --release        # run
cargo test --release       # full test suite
cargo build --release      # binary at target/release/mdoxide
```

## License

MIT. MDOxide reimplements the UX of MarkText and contains code derived from an MIT-licensed upstream project; see [`NOTICE`](NOTICE) for attribution details.
