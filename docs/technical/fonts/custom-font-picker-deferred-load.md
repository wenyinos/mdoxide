# Custom Font Picker: Deferred Load (macOS / Issue #133)

## Purpose

Avoid loading a system font until the user **explicitly picks** a family in Settings → Editor → **Custom System Font**. Eliminates spurious “Font failed to load … Reverted to Inter” toasts on some **macOS** setups where the first name from enumeration does not resolve the same way as `font-kit` load ([GitHub #133](https://github.com/OlaProeis/Ferrite/issues/133)).

## Behavior

| Phase | `EditorFont` | Font reload (`reload_fonts`) | Editor styling |
|-------|----------------|------------------------------|----------------|
| User selects **Custom** | `Custom("")` | Treat as no custom (`custom_name()` → `None`) | Inter-equivalent (`get_*_font_family` pending branch) |
| User picks a row in the combo | `Custom("Family Name")` | Load via `load_system_font_by_name` | `FONT_CUSTOM` |

Empty or whitespace-only custom names are ignored by `non_empty_custom_font_name()` inside font definition builders. `Settings::sanitize()` maps `Custom("")` to `Inter` after load so stale configs do not stay in limbo.

## UI

- Combo shows i18n placeholder `settings.editor.custom_font_pick_placeholder` until a font is chosen.
- “Font not found” appears only when a **non-empty** stored name is absent from the enumerated list.

## Related docs

- [Custom Font Selection](../editor/custom-font-selection.md) — full picker and CJK context  
- [v0.3.0 regression matrix](../platform/v0.3.0-regression-matrix.md) — **FNT-6** manual check (Intel macOS)

## Key code

| Location | Role |
|----------|------|
| `src/ui/settings.rs` | `EditorFont::Custom(String::new())` when enabling Custom; combo label |
| `src/config/settings.rs` | `EditorFont::custom_name()`; `sanitize()` for empty Custom |
| `src/fonts.rs` | `non_empty_custom_font_name`; pending `Custom` → Inter in `get_styled_font_family` / `get_base_font_family` |
