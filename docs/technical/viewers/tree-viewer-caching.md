# Tree Viewer Parse and Raw View Caching

The JSON/YAML/TOML tree viewer (`src/markdown/tree_viewer.rs`) caches structured parse results and the raw-view display buffer so unchanged tab content does not trigger a full re-parse or a fresh `String` allocation every frame.

## Blake3 content hash

Each `TreeViewer::show` pass computes one `blake3` digest (`[u8; 32]`) over the tab’s UTF-8 bytes. That value gates both caches.

## Parse cache

`TreeViewerState` stores:

- `cached_tree` — last successful `TreeNode`
- `cached_parse_error` — last `ParseError` when parse failed
- `parsed_content_hash` — digest when those fields were filled
- `parsed_file_type` — `StructuredFileType` used for that parse

If the current digest and file type match, `parse_structured_content` is not called. A change in extension-backed type (same tab) forces a refresh via the file-type check.

The interactive tree temporarily `take()`s the cached `TreeNode` while rendering so the UI can hold `&mut TreeViewer` while mutating the tree, then stores it back.

## Raw view cache

`raw_view_text` (`Option<String>`) and `raw_view_text_hash` mirror the CSV viewer’s raw buffer pattern: when the digest matches, the existing `String` is fed to `TextEdit::multiline` without calling `to_string()` on the full content each frame.

## Invalidation

Any edit to tab content changes the digest; the next frame rebuilds parse and/or raw caches as needed. Per-tab `TreeViewerState` in `AppState` avoids cross-tab leakage.

## Related

- [Markdown AST caching](../markdown/markdown-ast-caching.md) — global markdown AST cache
- [CSV raw view caching](./csv-raw-view-caching.md) — blake3-guarded raw CSV buffer
