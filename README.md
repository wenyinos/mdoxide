# MDOxide

**English** | [简体中文](README.zh-CN.md)

A native WYSIWYG Markdown editor written in Rust, built with `egui`/`eframe` — no WebView, no Electron, no JavaScript runtime.

MDOxide reimplements [MarkText](https://github.com/marktext/marktext)'s editing experience in Rust. It started from an MIT-licensed egui editor; full attribution is in [`NOTICE`](NOTICE).

The name: **Oxide** (Rust's oxidized-iron identity) + **MD** (Markdown).

## Status

Early development. M0 (baseline fork + rename + code audit) is complete; M1 (module reduction, stability) and M2 (MarkText visual identity) are in progress. See [`docs/plans/`](docs/plans/) for the implementation plan and the M0 audit report.

## What works today

- WYSIWYG markdown editing (per-block rendered/editing switch, commit-boundary undo)
- CommonMark + GFM via `comrak`; syntax highlighting via `syntect`
- Raw / Rendered / Split view modes
- Native Mermaid rendering (11 diagram types, pure Rust)
- HTML and PDF export
- File tree, workspace mode, find & replace (regex)
- 10 UI languages

## Planned

- MarkText visual identity: frameless title bar, tab strip, three-tab sidebar, bubble format toolbar
- Typewriter / focus modes aligned with Muya behaviour
- Bundled Source Han Sans (思源黑体) for reliable CJK rendering
- RPM / DEB / Windows NSIS packaging

## Build

Requirements: Rust 1.92 (pinned in `rust-toolchain.toml`), plus the usual Linux GUI dev packages (`gtk3-devel`, `libxkbcommon-devel`, `wayland-devel` on Fedora).

```bash
cargo run --release        # run
cargo test --release       # 1602 tests, 0 failures
cargo build --release      # binary at target/release/mdoxide
```

## License

MIT. MDOxide reimplements the UX of MarkText and contains code derived from an MIT-licensed upstream project; see [`NOTICE`](NOTICE) for attribution details.
