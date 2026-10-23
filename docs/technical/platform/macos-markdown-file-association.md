# macOS Markdown File Association

## Overview

MDOxide’s macOS `.app` bundle declares the standard Markdown UTI (`net.daringfireball.markdown`) so Finder can offer **Open With → MDOxide** and users can set MDOxide as the default app for `.md` and related extensions. This uses both **imported type declarations** and **document types** merged into `Info.plist` at bundle time.

## Key Files

- `assets/macos/info_plist_ext.xml` — Plist fragment merged by `cargo-bundle`: `UTImportedTypeDeclarations` for the Markdown UTI, plus `CFBundleDocumentTypes` for editor role and extensions.
- `Cargo.toml` — `[package.metadata.bundle]` → `osx_info_plist_exts = ["assets/macos/info_plist_ext.xml"]`.
- `.github/workflows/release.yml` — macOS release jobs run `cargo bundle --release` (Intel uses `--target x86_64-apple-darwin`); the merged plist ships inside `MDOxide.app` in DMG/tar artifacts.

## Implementation Details

- **UTI:** `net.daringfireball.markdown` (reference: Daring Fireball Markdown), conforming to `public.plain-text`.
- **Extensions:** `md`, `markdown`, `mdown`, `mkd`, `mkdn`.
- **`UTImportedTypeDeclarations`:** Imports the UTI so the system knows how `.md` files relate to the declared type when resolving handlers.
- **`CFBundleDocumentTypes`:** Registers MDOxide as an editor for that UTI (and related content types), alongside JSON, YAML, TOML, and plain text entries in the same fragment.

CI does not patch `Info.plist` separately; `cargo-bundle` reads `osx_info_plist_exts` from `Cargo.toml` on every build.

## Dependencies Used

- **cargo-bundle** — Produces `MDOxide.app` and merges `info_plist_ext.xml` into `Contents/Info.plist`.

## Usage

### Verify the merged plist (macOS)

After a local bundle build:

```bash
cargo install cargo-bundle   # once
cargo bundle --release
plutil -p target/release/bundle/osx/MDOxide.app/Contents/Info.plist | head -80
```

Confirm entries for `UTImportedTypeDeclarations` / `CFBundleDocumentTypes` and `net.daringfireball.markdown`.

### Manual UX check

1. Install or run `MDOxide.app` from a release DMG or local `cargo bundle`.
2. In Finder, right-click a `.md` file → **Open With** — MDOxide should appear.
3. Optionally set as default — double-click should open MDOxide.

## Related

- [macOS .app Bundle CI](./macos-app-bundle-ci.md) — CI packaging and `cargo-bundle` setup.
- GitHub: [#102](https://github.com/OlaProeis/MDOxide/issues/102)
