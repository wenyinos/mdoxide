# MDOxide

**English** | [简体中文](README.zh-CN.md)

A native WYSIWYG Markdown editor written in Rust, built with `egui`/`eframe` — no WebView, no Electron, no JavaScript runtime.

MDOxide reimplements [MarkText](https://github.com/marktext/marktext)'s editing experience in Rust. It started from an MIT-licensed egui editor; full attribution is in [`NOTICE`](NOTICE).

The name: **Oxide** (Rust's oxidized-iron identity) + **MD** (Markdown).

## Status

Feature-complete for the v0.1.0 scope. M0–M4 are done: baseline fork and audit, module reduction, MarkText visual identity, interaction parity (including a real fcitx5 IME walkthrough), and export quality. M5 — packaging and release polish — is in progress. See [`docs/plans/`](docs/plans/) for the implementation plan and audit reports.

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

## Planned

- Release packaging verification (RPM / DEB / Windows NSIS)
- PDF v1 limitations: Mermaid blocks, image placeholders, bidi reordering

## Build

Requirements: Rust 1.92 (pinned in `rust-toolchain.toml`), plus the usual Linux GUI dev packages (`gtk3-devel`, `libxkbcommon-devel`, `wayland-devel` on Fedora).

```bash
cargo run --release        # run
cargo test --release       # 1601 tests, 0 failures
cargo build --release      # binary at target/release/mdoxide
```

## License

MIT. MDOxide reimplements the UX of MarkText and contains code derived from an MIT-licensed upstream project; see [`NOTICE`](NOTICE) for attribution details.
