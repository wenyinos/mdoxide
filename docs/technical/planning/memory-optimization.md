# Memory Optimization Plan

> **Target Version:** v0.2.6  
> **Status:** Complete (Exceeded Goal)  
> **Priority:** Medium-High

## Results Summary

**Target:** Reduce idle RAM from ~250MB to ~100-150MB  
**Achieved:** ~80MB idle (68% reduction)

| Fix Applied | Impact |
|-------------|--------|
| Task 6: Viewer state HashMap cleanup on tab close | Prevented unbounded growth |
| Task 7: Custom allocators (mimalloc/jemalloc) | ~10-30% reduction |
| Task 8: CJK font lazy loading | ~50-100MB savings |

Task 9 (undo history limits) was **deferred** as the target was exceeded without it.

---

## Original Problem Statement

Ferrite's idle RAM usage was approximately **250MB**, which was higher than expected for a native Rust application. While reasonable for a feature-rich GUI editor (compared to Electron apps at 500MB-1GB+), there were optimization opportunities to reduce this to a target of **~100-150MB**.

## Analysis Summary

Memory usage breakdown (estimated):

| Component | Est. Memory | Issue Type |
|-----------|-------------|------------|
| CJK Fonts (system) | 50-100 MB | Eager loading |
| egui Font Atlases | 20-50 MB | CJK glyph rasterization |
| Syntect | 20-40 MB | All languages loaded |
| System allocator fragmentation | 10-30 MB | No custom allocator |
| Per-tab state leak | 0-20 MB | **Memory leak** |
| Undo history (5 tabs) | 5-25 MB | Full snapshots |
| Mermaid cache | 5-10 MB | Bounded (OK) |
| Base egui/eframe | 10-15 MB | Normal |
| **Total Estimated** | **120-290 MB** | |

---

## Issues & Fixes

### 1. Memory Leak in Tab State (CONFIRMED) 🔴

**Location:** `src/app.rs:206-212`

```rust
tree_viewer_states: HashMap<usize, TreeViewerState>,
csv_viewer_states: HashMap<usize, CsvViewerState>,
sync_scroll_states: HashMap<usize, SyncScrollState>,
```

**Problem:** When tabs are closed via `force_close_tab()`, these HashMaps are never cleaned up. Tab IDs are never reused, so entries accumulate indefinitely.

**Fix:** Add cleanup in `FerriteApp` when `AppState::force_close_tab()` is called:

```rust
// When a tab is closed, clean up associated state
fn cleanup_tab_state(&mut self, tab_id: usize) {
    self.tree_viewer_states.remove(&tab_id);
    self.csv_viewer_states.remove(&tab_id);
    self.sync_scroll_states.remove(&tab_id);
}
```

**Impact:** Low-Medium (prevents unbounded growth over long sessions)

---

### 2. System Allocator Fragmentation 🟠

**Problem:** Rust's default system allocator (Windows HeapAlloc, Linux malloc) fragments memory over time. After many allocations/deallocations (opening files, undo/redo, parsing), the heap reports high usage even when actual data is smaller.

**Fix:** Add a custom allocator like `mimalloc` or `jemalloc`:

```toml
# Cargo.toml
[target.'cfg(not(target_env = "msvc"))'.dependencies]
jemallocator = "0.5"

[target.'cfg(target_env = "msvc")'.dependencies]
mimalloc = { version = "0.1", default-features = false }
```

```rust
// main.rs
#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[cfg(target_env = "msvc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;
```

**Impact:** 10-30% reduction in reported memory, better long-term stability

**Considerations:**
- `jemalloc` doesn't work well on Windows MSVC, so use `mimalloc` there
- Increases binary size slightly (~100KB)
- Test on all platforms before shipping

---

### 3. CJK Font Loading Strategy 🟠

**Location:** `src/fonts.rs:217-269`

**Problem:** All 4 CJK font variants (Korean, Simplified Chinese, Traditional Chinese, Japanese) are loaded eagerly at startup, consuming 50-100MB even if the user never types CJK text.

**Current behavior:**
```rust
fn load_cjk_fonts(fonts: &mut FontDefinitions) -> CjkFontState {
    // Always loads Korean font (~10-25MB)
    // Always loads Simplified Chinese font (~15-25MB)
    // Always loads Traditional Chinese font (~15-25MB)
    // Always loads Japanese font (~10-20MB)
}
```

#### Option A: Detection-Based Loading (Recommended)

Only load CJK fonts when CJK characters are detected in the document:

```rust
/// Check if text contains CJK characters
fn contains_cjk(text: &str) -> bool {
    text.chars().any(|c| {
        let cp = c as u32;
        // CJK Unified Ideographs and common ranges
        (0x4E00..=0x9FFF).contains(&cp) ||  // CJK Unified
        (0x3040..=0x309F).contains(&cp) ||  // Hiragana
        (0x30A0..=0x30FF).contains(&cp) ||  // Katakana
        (0xAC00..=0xD7AF).contains(&cp) ||  // Korean Hangul
        (0x3400..=0x4DBF).contains(&cp)     // CJK Extension A
    })
}

/// Load CJK font on-demand when needed
fn ensure_cjk_font_loaded(ctx: &egui::Context, preference: CjkFontPreference) {
    // Check if already loaded
    if CJK_FONTS_LOADED.load(Ordering::Relaxed) {
        return;
    }
    
    // Load only the preferred CJK font (not all 4)
    let font_data = match preference {
        CjkFontPreference::Korean => load_system_font(&["Malgun Gothic", ...]),
        CjkFontPreference::SimplifiedChinese => load_system_font(&["Microsoft YaHei", ...]),
        // ...
    };
    
    // Hot-reload font into egui
    ctx.set_fonts(...);
    CJK_FONTS_LOADED.store(true, Ordering::Relaxed);
}
```

**Pros:**
- Zero memory cost for users who don't use CJK
- Loads only the needed font variant
- Automatic, no user action required

**Cons:**
- Detection adds small CPU overhead per document
- First CJK character may cause brief lag while font loads
- Need to scan document on open/edit

#### Option B: User Opt-in via Settings

Don't load CJK fonts by default. Add a setting in Settings → Appearance:

```
☐ Enable CJK font support
   (Required for Chinese, Japanese, Korean text. Uses ~20-30MB RAM)
```

**Pros:**
- Simplest implementation
- No detection overhead
- User has full control

**Cons:**
- CJK users must manually enable
- Poor out-of-box experience for CJK users

#### Recommended Approach

**Hybrid:** Use Option A (detection-based) with a settings toggle to force-enable.

1. By default, scan document content for CJK on load
2. If CJK detected, load only the preferred font (based on `CjkFontPreference` setting)
3. Add setting to pre-load CJK fonts for users who know they'll need them

**Implementation notes:**
- Detection is fast: ~1ms for 100KB document
- Load font in background thread to avoid UI stutter
- Cache detection result per tab to avoid re-scanning

---

### 4. Syntect Lazy Loading 🟡

**Location:** `src/markdown/syntax.rs:147-150`

**Problem:** All 40+ syntax grammars and themes are loaded at startup:

```rust
let syntax_set = SyntaxSet::load_defaults_newlines();  // ~20-30MB
let theme_set = ThemeSet::load_defaults();              // ~5-10MB
```

**Fix Options:**

1. **Use smaller default set:**
   ```rust
   let syntax_set = SyntaxSet::load_defaults_nonewlines(); // Slightly smaller
   ```

2. **Lazy-load per language:** Only load grammars when first needed. Syntect supports this but requires more complex initialization.

3. **Embedded subset:** Bundle only commonly-used languages (Rust, Python, JS, TS, JSON, YAML, HTML, CSS, Markdown, Shell) instead of all 40+.

**Impact:** 10-20MB savings

**Complexity:** Medium (syntect's API makes lazy loading awkward)

---

### 5. Undo History Optimization 🟡

**Location:** `src/state.rs:1078-1080`

**Problem:** Each undo entry stores a complete copy of the document:

```rust
max_undo_size: 100,  // Up to 100 full document copies per tab
```

For a 50KB file with 100 undo entries = 5MB per tab.

**Fix Options:**

1. **Reduce limit:** `max_undo_size: 50` (easy, saves ~50% per tab)

2. **Diff-based undo:** Store deltas instead of full content (complex, significant savings)

3. **Size-aware limit:** Reduce undo history for large files:
   ```rust
   let max_undo = if content.len() > 100_000 { 20 } else { 100 };
   ```

**Recommended:** Option 1 + Option 3 for quick wins.

---

### 6. egui Temp Data Cleanup 🟡

**Location:** `src/editor/widget.rs:55-64`

**Problem:** `SyntaxHighlightCache` is stored in egui's global context keyed by `egui::Id`, but entries aren't cleaned up when tabs close.

```rust
struct SyntaxHighlightCache {
    cache: HashMap<egui::Id, SyntaxCacheEntry>,
    galley_cache: HashMap<egui::Id, GalleyCacheEntry>,
    deferred_state: HashMap<egui::Id, DeferredHighlightState>,
}
```

**Fix:** Add periodic cleanup of stale entries, or clear entries when tabs close.

---

## Implementation Priority

| Fix | Effort | Impact | Priority | Status |
|-----|--------|--------|----------|--------|
| Tab state memory leak | Easy | Medium | **P1** | ✅ Done (Task 6) |
| Custom allocator | Easy | Medium | **P1** | ✅ Done (Task 7) |
| CJK font lazy loading | Medium | High | **P2** | ✅ Done (Task 8) |
| Reduce undo history | Easy | Low-Medium | **P2** | Deferred (Task 9) |
| Syntect lazy loading | Medium | Medium | **P3** | Not needed |
| egui temp data cleanup | Medium | Low | **P3** | Deferred (Task 10) |

---

## Testing & Validation

### Measuring Memory

**Windows:**
```powershell
# Task Manager shows "Memory (Private Working Set)"
# Or use Process Explorer for detailed breakdown
```

**Linux:**
```bash
# Resident Set Size
ps -o rss= -p $(pgrep ferrite)

# Or use heaptrack for allocation profiling
heaptrack ./ferrite
```

**macOS:**
```bash
# Use Activity Monitor or:
ps -o rss= -p $(pgrep -i ferrite)
```

### Benchmarks to Track

1. **Idle memory** - Launch app, open one empty file, wait 10s
2. **Working memory** - Open 5 medium files (50KB each), edit each
3. **Peak memory** - Open large workspace with 100+ files
4. **Long session** - Open/close 50 tabs, check for growth

---

## References

- [mimalloc GitHub](https://github.com/microsoft/mimalloc)
- [jemalloc GitHub](https://github.com/jemalloc/jemalloc)
- [egui memory management](https://docs.rs/egui/latest/egui/struct.Memory.html)
- [syntect performance](https://github.com/trishume/syntect#performance)
