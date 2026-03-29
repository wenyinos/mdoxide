# Markdown AST Caching

## Overview

Caches parsed `MarkdownDocument` AST results using blake3 content hashing to avoid re-parsing unchanged markdown on every frame in the rendered view. Follows the same global singleton pattern as the existing `MermaidCacheManager`.

## Key Files

- `src/markdown/cache.rs` — Global AST cache: blake3 hash → `MarkdownDocument`
- `src/markdown/editor.rs` — Integration point: `show_rendered_editor()` calls `cache::get_or_parse()`
- `src/markdown/mod.rs` — Module export

## Implementation Details

The rendered view (`EditorMode::Rendered`) calls `parse_markdown()` every frame to build the widget tree. For unchanged content this is pure waste. The cache intercepts this with a blake3 content hash lookup.

**Cache key:** `blake3::hash(content.as_bytes())` — a 32-byte hash. No rendering parameters (font size, width) are needed since the AST is layout-independent.

**Cache storage:** `static Mutex<Option<MarkdownAstCache>>` with LRU eviction at 32 entries. Different tabs naturally produce different hashes, so per-tab isolation is automatic without explicit tab IDs.

**Cache flow:**
1. Hash the content string (sub-microsecond for typical documents)
2. Look up hash in the HashMap
3. On hit: return cloned `MarkdownDocument`, skip parsing
4. On miss: call `parse_markdown()`, store result, return clone

**Invalidation:** Content edits change the hash, causing a natural cache miss. `clear_ast_cache()` is available for global invalidation (e.g., settings changes).

## Dependencies Used

- `blake3` (already in Cargo.toml for Mermaid caching) — fast cryptographic hashing

## Usage

The cache is transparent — no API changes to `MarkdownEditor`. The `show_rendered_editor` method calls `cache::get_or_parse(self.content)` instead of `parse_markdown(self.content)` directly.

```rust
// Before: re-parses every frame
let doc = parse_markdown(self.content)?;

// After: cached by content hash
let doc = cache::get_or_parse(self.content)?;
```

To clear the cache programmatically:

```rust
crate::markdown::cache::clear_ast_cache();
```
