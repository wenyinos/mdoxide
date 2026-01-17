# Multi-Encoding File Support

## Overview

Ferrite v0.2.6 adds comprehensive character encoding support, allowing users to open, edit, and save files in various encodings beyond UTF-8. The encoding is auto-detected on file open and can be manually changed via the status bar.

## Key Files

- `src/state.rs` - Tab struct encoding fields, detection logic, encode/decode methods
- `src/app.rs` - Status bar encoding UI with dropdown picker
- `Cargo.toml` - Added `encoding_rs` and `chardetng` dependencies

## Implementation Details

### Tab Struct Changes

Three new fields added to `Tab`:

```rust
pub struct Tab {
    // ... existing fields ...
    
    /// Detected encoding when file was opened (e.g., "UTF-8", "WINDOWS-1252")
    /// None for new/unsaved documents created in-app
    pub detected_encoding: Option<&'static str>,
    
    /// Original file bytes for re-decoding when user changes encoding
    /// Empty for new documents created in-app
    pub original_bytes: Vec<u8>,
    
    /// Currently selected encoding label (used for save operations)
    /// Defaults to "utf-8" for new documents
    pub current_encoding: &'static str,
}
```

### Encoding Detection Flow

1. **Read file as bytes** - `std::fs::read(&path)` instead of `read_to_string()`
2. **Check for BOM** - `encoding_rs::Encoding::for_bom(&bytes)` detects UTF-8/16 BOMs
3. **Fallback to chardetng** - Statistical detection for non-BOM files
4. **Decode to String** - `encoding.decode_without_bom_handling()` for BOM-detected files

```rust
// BOM detection takes priority
if let Some((bom_encoding, bom_len)) = encoding_rs::Encoding::for_bom(&bytes) {
    let (decoded, _) = bom_encoding.decode_without_bom_handling(&bytes[bom_len..]);
    // Use bom_encoding
} else {
    // Use chardetng detection
    let mut detector = chardetng::EncodingDetector::new();
    detector.feed(&bytes, true);
    let detected = detector.guess(None, true);
    let (decoded, _, _) = detected.decode(&bytes);
}
```

### Supported Encodings

```rust
pub const COMMON_ENCODINGS: &'static [&'static str] = &[
    "utf-8", "windows-1252", "iso-8859-1", "shift_jis", 
    "euc-jp", "gbk", "euc-kr", "iso-8859-15", 
    "utf-16le", "utf-16be",
];
```

### Re-encoding on Save

When saving, content is encoded from UTF-8 String to bytes using the selected encoding:

```rust
pub fn encode_content(&self) -> Vec<u8> {
    if let Some(encoding) = encoding_rs::Encoding::for_label(self.current_encoding.as_bytes()) {
        let (bytes, _, _) = encoding.encode(&self.content);
        bytes.into_owned()
    } else {
        self.content.as_bytes().to_vec() // Fallback to UTF-8
    }
}
```

### Manual Encoding Change

Users can re-decode file content with a different encoding:

```rust
pub fn set_encoding(&mut self, new_encoding: &'static str) -> Result<(), String> {
    if self.original_bytes.is_empty() {
        return Err("No original bytes available".to_string());
    }
    
    if let Some(encoding) = encoding_rs::Encoding::for_label(new_encoding.as_bytes()) {
        let (decoded, _) = encoding.decode_without_bom_handling(&self.original_bytes);
        self.content = decoded.into_owned();
        self.current_encoding = new_encoding;
        Ok(())
    } else {
        Err(format!("Unsupported encoding: {}", new_encoding))
    }
}
```

### Status Bar UI

The encoding is displayed as a clickable button in the status bar. Clicking opens a dropdown with common encodings:

```rust
// In app.rs status bar
let encoding_btn = ui.button(tab.encoding_display_name());
if encoding_btn.clicked() {
    ui.memory_mut(|mem| mem.toggle_popup(popup_id));
}

egui::popup_below_widget(ui, popup_id, &encoding_btn, |ui| {
    for &enc in Tab::COMMON_ENCODINGS {
        if ui.selectable_label(enc == tab.current_encoding, enc.to_uppercase()).clicked() {
            pending_encoding_change = Some(enc);
            ui.memory_mut(|mem| mem.close_popup());
        }
    }
});
```

## Dependencies Used

- `encoding_rs = "0.8"` - Fast character encoding conversion (same library used by Firefox)
- `chardetng = "0.1"` - Encoding detection (same algorithm as Firefox)

## Session Restoration

When restoring sessions, files are re-read from disk with encoding detection:

```rust
enum ResolvedContent {
    /// Content recovered from crash backup (already UTF-8)
    Recovered(String),
    /// Content loaded from disk with encoding info
    FromDisk {
        content: String,
        original_bytes: Vec<u8>,
        encoding: &'static str,
    },
}
```

## Testing

Test files are provided in `test_md/encoding_tests/`:

| File | Encoding | Notes |
|------|----------|-------|
| `utf8_no_bom.md` | UTF-8 | Standard UTF-8 |
| `utf8_with_bom.md` | UTF-8 + BOM | EF BB BF prefix |
| `windows1252.txt` | Windows-1252 | Curly quotes, Euro symbol |
| `latin1.txt` | ISO-8859-1 | Western European |
| `shiftjis.txt` | Shift-JIS | Japanese |
| `eucjp.txt` | EUC-JP | Japanese Unix |
| `gbk.txt` | GBK | Simplified Chinese |
| `euckr.txt` | EUC-KR | Korean |
| `utf16le.txt` | UTF-16 LE | With FF FE BOM |
| `utf16be.txt` | UTF-16 BE | With FE FF BOM |

Regenerate with: `python test_md/encoding_tests/create_test_files.py`

## Usage

1. **Open file** - Encoding auto-detected, shown in status bar
2. **Check encoding** - Look at status bar (e.g., "UTF-8", "SHIFT_JIS")
3. **Change encoding** - Click encoding button, select from dropdown
4. **Save** - File saved in currently selected encoding

## Limitations

- UTF-16 files require BOM for reliable detection
- Some similar encodings may be mis-detected (e.g., ISO-8859-1 vs Windows-1252)
- New documents always default to UTF-8
- Original bytes are kept in memory for re-decoding (increases memory per tab)
