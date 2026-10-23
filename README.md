# MDOxide

**English** | [简体中文](README.zh-CN.md)

A native WYSIWYG Markdown editor written in Rust, built with `egui`/`eframe` — no WebView, no Electron, no JavaScript runtime.

MDOxide reimplements [MarkText](https://github.com/marktext/marktext)'s editing experience in Rust. It started from an MIT-licensed egui editor; full attribution is in [`NOTICE`](NOTICE).

The name: **Oxide** (Rust's oxidized-iron identity) + **MD** (Markdown).

## Status

**v0.1.0 released.** All six milestones are complete: baseline fork and audit, module reduction, MarkText visual identity, interaction parity (including a real fcitx5 IME walkthrough), export quality, and packaging. Release artifacts (RPM, DEB, Windows installer) are built by GitHub Actions.

## What works today

- WYSIWYG markdown editing (per-block rendered/editing switch, commit-boundary undo)
- Raw / Rendered / Split view modes
- MarkText-style UI: frameless title bar, centred tab strip, three-tab sidebar, selection format toolbar, focus and typewriter modes
- Bundled Source Han Sans for reliable CJK rendering; fcitx5 IME verified on Wayland (preedit, candidate window, backspace, commit, Esc)
- CommonMark + GFM via `comrak`; syntax highlighting via `syntect`
- Native Mermaid rendering (11 diagram types) and Mermaid SVG in HTML export
- RaTeX math rendering in the editor and in HTML/PDF exports
- HTML and PDF export with embedded CJK fonts, tables, code blocks, links, and formula images
- File tree, workspace mode, find & replace (regex)
- 10 UI languages

## Known limitations

- PDF v1: Mermaid blocks render as code blocks, images are placeholders, and bidi reordering is not applied
- macOS is not supported

## Build

Requirements: Rust 1.92 (pinned in `rust-toolchain.toml`), plus the usual Linux GUI dev packages (`gtk3-devel`, `libxkbcommon-devel`, `wayland-devel` on Fedora).

```bash
cargo run --release        # run
cargo test --release       # 1601 tests, 0 failures
cargo build --release      # binary at target/release/mdoxide
```

## License

MIT. MDOxide reimplements the UX of MarkText and contains code derived from an MIT-licensed upstream project; see [`NOTICE`](NOTICE) for attribution details.
