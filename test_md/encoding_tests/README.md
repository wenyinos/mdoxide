# Encoding Test Files

This folder contains test files in various character encodings for testing Ferrite's multi-encoding support.

## Test Files

| File | Encoding | Description |
|------|----------|-------------|
| `utf8_no_bom.md` | UTF-8 | Standard UTF-8 without BOM |
| `utf8_with_bom.md` | UTF-8 with BOM | UTF-8 with EF BB BF BOM |
| `windows1252.txt` | Windows-1252 | Western European (includes €, curly quotes) |
| `latin1.txt` | ISO-8859-1 | Latin-1 (subset of Windows-1252) |
| `shiftjis.txt` | Shift-JIS | Japanese encoding |
| `eucjp.txt` | EUC-JP | Japanese Unix encoding |
| `gbk.txt` | GBK | Simplified Chinese |
| `euckr.txt` | EUC-KR | Korean encoding |
| `utf16le.txt` | UTF-16 LE | Little Endian with FF FE BOM |
| `utf16be.txt` | UTF-16 BE | Big Endian with FE FF BOM |

## Testing Procedure

1. Open each file in Ferrite
2. Check that the encoding is correctly detected (shown in status bar)
3. Verify content displays correctly (no mojibake/garbled text)
4. Try changing encoding via status bar dropdown
5. Save and verify the file preserves the encoding

## Regenerating Test Files

Run the Python script to regenerate all test files:

```bash
python create_test_files.py
```

## Notes

- UTF-16 files require BOM for reliable detection
- Some encodings (like Shift-JIS) may be detected as similar encodings
- Empty files default to UTF-8
