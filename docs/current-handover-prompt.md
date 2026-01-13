# Session Handover - Ferrite v0.2.5

## Rules

- Never auto-update this file - only update when explicitly requested
- Complete entire task before requesting next instruction
- Run `cargo build` / `cargo check` after changes to verify code compiles
- Follow existing code patterns and conventions
- Update task status via Task Master when starting (`in-progress`) and completing (`done`)
- Use Context7 MCP tool to fetch library documentation when needed (e.g., rust-i18n, egui)
- Document by feature (e.g., `editor-widget.md`), not by task (e.g., `task-1.md`)
- Update `docs/index.md` when adding new documentation

## Environment

- **Project:** Ferrite
- **Path:** G:\DEV\markDownNotepad
- **GitHub:** https://github.com/OlaProeis/Ferrite
- **Version:** Working on v0.2.5
- **Tech Stack:** Rust, egui 0.28, eframe 0.28, comrak 0.22, clap 4

---

## Current Task

| Field | Value |
|-------|-------|
| **ID** | 1 |
| **Title** | Add rust-i18n dependency and setup locales directory structure |
| **Status** | `pending` |
| **Priority** | High |
| **Dependencies** | None |

### Description

Integrate rust-i18n crate and create locales directory with en.yaml as base language. This is the foundation for all i18n work in v0.2.5.

### Implementation Notes

1. **Add dependency to Cargo.toml:**
   ```toml
   rust-i18n = "3"  # Check latest version on crates.io
   ```

2. **Create locales directory structure:**
   ```
   locales/
   └── en.yaml    # English (base language)
   ```

3. **Configure rust-i18n in main.rs or lib.rs:**
   ```rust
   rust_i18n::i18n!("locales");
   ```

4. **Create initial en.yaml with a few test strings:**
   ```yaml
   app:
     name: "Ferrite"
     version: "v0.2.5"
   menu:
     file: "File"
     edit: "Edit"
   ```

5. **Test basic usage:**
   ```rust
   use rust_i18n::t;
   let text = t!("menu.file");  // Returns "File"
   ```

### Test Strategy

| Test | Approach |
|------|----------|
| Compilation | `cargo build` passes without warnings |
| Basic usage | Add a test `t!()` call somewhere visible |
| Fallback | Missing keys should fallback gracefully |

---

## Key Files

| File | Purpose |
|------|---------|
| `Cargo.toml` | Add rust-i18n dependency |
| `src/main.rs` | Add i18n! macro initialization |
| `locales/en.yaml` | English translation file (create new) |
