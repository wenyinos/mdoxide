# Custom Font Crash Prevention

Prevents application crashes when loading invalid or unsupported custom system fonts on Linux (and other platforms).

## Problem

Selecting certain system fonts via the Settings panel could crash the app because:
- `epaint` panics on invalid font data (e.g., font collections, corrupt files)
- `font-kit` may return data that isn't a single TTF/OTF font
- No validation existed between reading font bytes and passing them to egui

## Solution

### 1. Font Byte Validation (`validate_font_bytes`)

Magic-byte check applied before any font data reaches epaint:

| Magic Bytes | Format | Action |
|-------------|--------|--------|
| `\0\1\0\0` | TrueType | Accept |
| `OTTO` | OpenType (CFF) | Accept |
| `ttcf` | TTC/OTC collection | Reject — epaint can't select a face index |
| `wOFF` | WOFF | Reject |
| `wOF2` | WOFF2 | Reject |
| `%!` | Type 1 | Reject |
| Other | Unknown | Reject |

Empty files and files smaller than 4 bytes are also rejected.

### 2. Panic Protection (`catch_unwind`)

`load_system_font_by_name` wraps `FontData::from_owned()` in `std::panic::catch_unwind` as a safety net for any edge case the byte validation doesn't catch.

### 3. Graceful Fallback + Toast

- `reload_fonts()` now returns `Option<String>` — the error message on failure, `None` on success.
- On failure, the font system still loads Inter (the default) so the UI remains functional.
- `central_panel.rs` resets `settings.font_family` to `EditorFont::default()` and shows a 5-second toast.
- At startup (`app/mod.rs::new()`), failures are stored in `AppState.pending_toast` and displayed on the first frame.

### 4. Custom Font Shaping (`ttf_bytes_for_font_id_shaping`)

Added `FONT_CUSTOM` match arm so HarfRust text shaping works with custom fonts. Raw font bytes are cached in `CUSTOM_FONT_BYTES` (a `Mutex<Option<&'static [u8]>>`) when the font loads successfully, and cleared when the font changes or fails.

## Key Files

| File | Changes |
|------|---------|
| `src/fonts.rs` | `validate_font_bytes`, `load_system_font_by_name` (Result + catch_unwind), `ttf_bytes_for_font_id_shaping` (FONT_CUSTOM), `CUSTOM_FONT_BYTES` static, `LAST_CUSTOM_FONT_ERROR` static, `reload_fonts` returns `Option<String>` |
| `src/app/central_panel.rs` | Handle `reload_fonts` error: reset font, show toast |
| `src/app/mod.rs` | Handle startup font error: reset + `pending_toast` |
| `src/state.rs` | Added `pending_toast: Option<String>` to `AppState` |

## Error Flow

```
User selects font → Settings panel
  → reload_fonts(ctx, Some("BadFont"), ...)
    → create_font_definitions_with_cjk_spec(Some("BadFont"), ...)
      → load_system_font_by_name("BadFont")
        → validate_font_bytes(bytes) → Err("BadFont is .ttc collection")
        → OR catch_unwind → panic caught
      → LAST_CUSTOM_FONT_ERROR = Some(reason)
      → custom_loaded = false (falls back to Inter)
    → returns LAST_CUSTOM_FONT_ERROR.take()
  → central_panel: reset font_family, show_toast("Font failed...")
```
