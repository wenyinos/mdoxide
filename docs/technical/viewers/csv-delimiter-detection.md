# CSV Delimiter Detection

## Overview

Automatic delimiter detection for CSV/TSV files with manual override capability. The system analyzes file content to detect the most likely delimiter (comma, tab, semicolon, or pipe) and allows users to manually override the detection via the status bar.

## Key Files

- `src/markdown/csv_viewer.rs` - Delimiter detection algorithm and CsvViewerState
- `src/app.rs` - Status bar delimiter picker UI
- `src/config/session.rs` - Per-file delimiter persistence

## Implementation Details

### Delimiter Detection Algorithm

The `detect_delimiter()` function analyzes the first 10 lines of the file and scores each candidate delimiter:

```rust
pub const DELIMITERS: &[u8] = &[b',', b'\t', b';', b'|'];

pub fn detect_delimiter(content: &str) -> DelimiterInfo {
    // Sample first 10 lines
    // Score each delimiter by:
    // 1. Column consistency across lines (major factor)
    // 2. Successful CSV parse without errors
    // 3. Reasonable column count (2-50 preferred)
    // 4. Non-empty cells bonus
}
```

Scoring criteria:
- **Consistency score**: Lines with matching column counts score higher
- **Column count**: Single-column results are penalized (likely wrong delimiter)
- **Parse success**: CSV parse errors reduce the score
- **Non-empty cells**: Files with more data in columns score higher

### CsvViewerState

```rust
pub struct CsvViewerState {
    delimiter_override: Option<u8>,  // Manual override
    detected_delimiter: Option<u8>,  // Cached auto-detection
    // ... other fields
}
```

Methods:
- `effective_delimiter()` - Returns override or detected delimiter
- `set_delimiter(u8)` - Set manual override
- `clear_delimiter_override()` - Return to auto-detect

### Status Bar UI

When a CSV/TSV file is open in Rendered mode:
- Shows `Delim: ,` (or appropriate symbol)
- Shows `✓` when manually overridden
- Click opens dropdown with:
  - "Auto-detect" option
  - Manual delimiter choices (Comma, Tab, Semicolon, Pipe)

### Session Persistence

The `SessionTabState` includes:
```rust
pub csv_delimiter: Option<u8>,  // Manual delimiter override
```

Delimiter preferences are:
- Saved when the session is saved
- Restored when the file is reopened
- Matched by file path during restoration

## Split View Support

CSV/TSV files support split view mode:
- Left pane: Raw text editor
- Right pane: Table viewer with delimiter detection

## Dependencies Used

- `csv` crate - CSV parsing and delimiter handling

## Usage

1. Open a CSV/TSV file
2. Delimiter is auto-detected and applied
3. To override: click delimiter indicator in status bar
4. Select desired delimiter from dropdown
5. Table re-parses with new delimiter
6. Preference is saved per-file in session

## Test Files

- `test_md/test_data.csv` - Comma-separated
- `test_md/test_data.tsv` - Tab-separated  
- `test_md/test_data_semicolon.csv` - Semicolon-separated
