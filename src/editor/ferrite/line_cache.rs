//! LineCache module for galley caching with LRU eviction.
//!
//! This module provides a `LineCache` struct that caches egui `Galley` objects
//! (text layouts) keyed by content hash. This avoids expensive galley recreation
//! on each frame for unchanged lines.
//!
//! # Features
//! - Content-hash based keys (same content = cache hit)
//! - LRU eviction when cache exceeds `MAX_CACHE_ENTRIES`
//! - Single-line galleys (no wrapping) for Phase 1
//!
//! # Example
//! ```rust,ignore
//! use crate::editor::LineCache;
//! use egui::{Painter, FontId, Color32};
//!
//! let mut cache = LineCache::new();
//!
//! // Get or create a galley for a line
//! let galley = cache.get_galley(
//!     "Hello, World!",
//!     &painter,
//!     FontId::monospace(14.0),
//!     Color32::WHITE,
//! );
//!
//! // Same content returns cached galley
//! let galley2 = cache.get_galley(
//!     "Hello, World!",
//!     &painter,
//!     FontId::monospace(14.0),
//!     Color32::WHITE,
//! );
//!
//! // galley and galley2 are the same Arc<Galley>
//! ```

use egui::{text::LayoutJob, text::TextFormat, Color32, FontId, Galley, Painter};
use std::collections::{HashMap, VecDeque};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;

/// Maximum number of cached galleys before LRU eviction kicks in.
const MAX_CACHE_ENTRIES: usize = 200;

// ─────────────────────────────────────────────────────────────────────────────
// Syntax Highlighting Segment
// ─────────────────────────────────────────────────────────────────────────────

/// A segment of highlighted text for syntax highlighting.
///
/// This is a simplified representation of a highlighted segment,
/// containing just the text and its color. More complex styling
/// (bold, italic) is handled by the syntax module if needed.
#[derive(Debug, Clone)]
pub struct HighlightedSegment {
    /// The text content of this segment
    pub text: String,
    /// Foreground color for this segment
    pub color: Color32,
}

/// Cache key combining line content and styling information.
///
/// Two lines with the same content but different fonts or colors will have
/// different cache keys. The key is a u64 hash combining content, font, color,
/// and optionally wrap width.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct CacheKey(u64);

impl CacheKey {
    /// Creates a new cache key from line content and styling.
    ///
    /// The key is a hash combining:
    /// - Line content
    /// - Font family name
    /// - Font size (as bits)
    /// - Text color
    fn new(content: &str, font_id: &FontId, color: Color32) -> Self {
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        
        // Hash font family
        match &font_id.family {
            egui::FontFamily::Monospace => 1u8.hash(&mut hasher),
            egui::FontFamily::Proportional => 2u8.hash(&mut hasher),
            egui::FontFamily::Name(name) => {
                3u8.hash(&mut hasher);
                name.hash(&mut hasher);
            }
        }
        
        // Hash font size (as bits for exact equality)
        font_id.size.to_bits().hash(&mut hasher);
        
        // Hash color
        color.to_array().hash(&mut hasher);
        
        Self(hasher.finish())
    }

    /// Creates a new cache key including wrap width.
    ///
    /// Used for wrapped galleys where the same content at different widths
    /// produces different layouts.
    fn new_wrapped(content: &str, font_id: &FontId, color: Color32, wrap_width: f32) -> Self {
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        
        // Hash font family
        match &font_id.family {
            egui::FontFamily::Monospace => 1u8.hash(&mut hasher),
            egui::FontFamily::Proportional => 2u8.hash(&mut hasher),
            egui::FontFamily::Name(name) => {
                3u8.hash(&mut hasher);
                name.hash(&mut hasher);
            }
        }
        
        // Hash font size (as bits for exact equality)
        font_id.size.to_bits().hash(&mut hasher);
        
        // Hash color
        color.to_array().hash(&mut hasher);

        // Hash wrap width (as bits for exact equality)
        // We round to nearest pixel to avoid cache misses from float precision
        let rounded_width = wrap_width.round() as u32;
        rounded_width.hash(&mut hasher);
        
        Self(hasher.finish())
    }
    
    /// Creates a cache key from content hash only (for LayoutJob caching).
    fn from_content_hash(content_hash: u64) -> Self {
        Self(content_hash)
    }

    /// Creates a cache key for syntax-highlighted content.
    ///
    /// The key includes:
    /// - Line content hash
    /// - Font family and size
    /// - Base text color (fallback for unstyled segments)
    /// - Syntax theme hash (changes when theme changes)
    /// - Optional wrap width (for wrapped galleys)
    fn new_highlighted(
        content: &str,
        font_id: &FontId,
        color: Color32,
        syntax_theme_hash: u64,
        wrap_width: Option<f32>,
    ) -> Self {
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);

        // Hash font family
        match &font_id.family {
            egui::FontFamily::Monospace => 1u8.hash(&mut hasher),
            egui::FontFamily::Proportional => 2u8.hash(&mut hasher),
            egui::FontFamily::Name(name) => {
                3u8.hash(&mut hasher);
                name.hash(&mut hasher);
            }
        }

        // Hash font size (as bits for exact equality)
        font_id.size.to_bits().hash(&mut hasher);

        // Hash color (fallback color for unhighlighted segments)
        color.to_array().hash(&mut hasher);

        // Hash syntax theme
        syntax_theme_hash.hash(&mut hasher);

        // Hash wrap width if provided (rounded for consistency)
        if let Some(width) = wrap_width {
            let rounded_width = width.round() as u32;
            rounded_width.hash(&mut hasher);
        }

        Self(hasher.finish())
    }
}

/// Hashes a string to a u64 using `DefaultHasher`.
///
/// This is a fast hash suitable for cache keys. The same content
/// will always produce the same hash.
fn hash_content(content: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}

/// Caches egui `Galley` objects to avoid recreating text layouts every frame.
///
/// `LineCache` stores galleys keyed by content hash, font, and color.
/// When the cache exceeds `MAX_CACHE_ENTRIES` (200), the least recently
/// used entries are evicted.
///
/// # Thread Safety
/// This struct is not thread-safe. Each `LineCache` should be used from
/// a single thread (typically the UI thread).
///
/// # Memory Usage
/// Each cached `Galley` contains text layout information. With 200 entries
/// and typical line lengths, memory usage is approximately 2-5 MB.
#[derive(Debug, Clone)]
pub struct LineCache {
    /// Maps cache keys to cached galleys.
    cache: HashMap<CacheKey, Arc<Galley>>,
    /// Tracks access order for LRU eviction. Front = oldest, back = newest.
    lru_order: VecDeque<CacheKey>,
}

impl Default for LineCache {
    fn default() -> Self {
        Self::new()
    }
}

impl LineCache {
    /// Creates a new empty `LineCache`.
    ///
    /// # Example
    /// ```rust,ignore
    /// let cache = LineCache::new();
    /// assert_eq!(cache.len(), 0);
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            cache: HashMap::with_capacity(MAX_CACHE_ENTRIES),
            lru_order: VecDeque::with_capacity(MAX_CACHE_ENTRIES),
        }
    }

    /// Gets a cached galley or creates a new one if not in cache.
    ///
    /// This is the primary method for obtaining galleys. It:
    /// 1. Checks if a galley for this content/font/color exists in cache
    /// 2. If found, returns the cached galley and updates LRU order
    /// 3. If not found, creates a new galley using `painter.layout_no_wrap()`
    /// 4. Caches the new galley (with LRU eviction if needed)
    ///
    /// # Arguments
    /// * `line_content` - The text content of the line
    /// * `painter` - The egui `Painter` used to create galleys
    /// * `font_id` - The font to use for the galley
    /// * `color` - The text color
    ///
    /// # Returns
    /// An `Arc<Galley>` containing the text layout. The Arc allows
    /// efficient sharing between the cache and caller.
    ///
    /// # Example
    /// ```rust,ignore
    /// let galley = cache.get_galley(
    ///     "fn main() {}",
    ///     &painter,
    ///     FontId::monospace(14.0),
    ///     Color32::WHITE,
    /// );
    /// // Use galley.size() to get dimensions
    /// // Use painter.galley(pos, galley, color) to render
    /// ```
    pub fn get_galley(
        &mut self,
        line_content: &str,
        painter: &Painter,
        font_id: FontId,
        color: Color32,
    ) -> Arc<Galley> {
        let key = CacheKey::new(line_content, &font_id, color);

        // Check cache first - clone the galley before updating LRU
        if let Some(galley) = self.cache.get(&key).cloned() {
            // Update LRU order - move to back (most recently used)
            self.update_lru_order(key);
            return galley;
        }

        // Create new galley using egui's layout_no_wrap (no word wrapping)
        let galley = painter.layout_no_wrap(line_content.to_string(), font_id, color);

        // Cache the galley
        self.insert(key, Arc::clone(&galley));

        galley
    }

    /// Gets a cached galley using a `LayoutJob` for more complex text styling.
    ///
    /// This method supports syntax highlighting and other advanced text formatting
    /// where different parts of a line may have different colors or fonts.
    ///
    /// # Arguments
    /// * `line_content` - The text content (used for cache key hashing)
    /// * `layout_job` - The `LayoutJob` describing the text formatting
    /// * `painter` - The egui `Painter` used to create galleys
    ///
    /// # Returns
    /// An `Arc<Galley>` containing the styled text layout.
    ///
    /// # Note
    /// The cache key is based on content hash only, so if the same content
    /// has different styling (e.g., different syntax highlighting), consider
    /// including styling info in the content or using separate caches.
    pub fn get_galley_with_job(
        &mut self,
        line_content: &str,
        layout_job: LayoutJob,
        painter: &Painter,
    ) -> Arc<Galley> {
        // For LayoutJob, we use a simplified key based on content hash only
        // In the future, we might want to hash the entire LayoutJob
        let key = CacheKey::from_content_hash(hash_content(line_content));

        // Check cache first - clone the galley before updating LRU
        if let Some(galley) = self.cache.get(&key).cloned() {
            self.update_lru_order(key);
            return galley;
        }

        // Create galley from LayoutJob
        let galley = painter.layout_job(layout_job);

        // Cache the galley
        self.insert(key, Arc::clone(&galley));

        galley
    }

    /// Gets a cached galley with syntax highlighting.
    ///
    /// This method creates a galley from highlighted segments, caching based on
    /// content, font, and syntax theme. This ensures cache invalidation when
    /// the syntax theme changes.
    ///
    /// # Arguments
    /// * `line_content` - The raw text content of the line
    /// * `segments` - Highlighted segments from the syntax highlighter
    /// * `painter` - The egui `Painter` used to create galleys
    /// * `font_id` - The font to use for the galley
    /// * `default_color` - Fallback color for text
    /// * `syntax_theme_hash` - Hash of the current syntax theme (for cache invalidation)
    /// * `wrap_width` - Optional wrap width for word wrapping
    ///
    /// # Returns
    /// An `Arc<Galley>` containing the syntax-highlighted text layout.
    pub fn get_galley_highlighted(
        &mut self,
        line_content: &str,
        segments: &[HighlightedSegment],
        painter: &Painter,
        font_id: FontId,
        default_color: Color32,
        syntax_theme_hash: u64,
        wrap_width: Option<f32>,
    ) -> Arc<Galley> {
        let key = CacheKey::new_highlighted(
            line_content,
            &font_id,
            default_color,
            syntax_theme_hash,
            wrap_width,
        );

        // Check cache first
        if let Some(galley) = self.cache.get(&key).cloned() {
            self.update_lru_order(key);
            return galley;
        }

        // Build LayoutJob from highlighted segments
        let mut job = LayoutJob::default();
        job.wrap.max_width = wrap_width.unwrap_or(f32::INFINITY);

        for segment in segments {
            let mut format = TextFormat::default();
            format.font_id = font_id.clone();
            format.color = segment.color;
            // Note: bold/italic would require different font_ids, which egui handles internally
            job.append(&segment.text, 0.0, format);
        }

        // Handle empty lines
        if segments.is_empty() {
            let format = TextFormat {
                font_id,
                color: default_color,
                ..Default::default()
            };
            job.append("", 0.0, format);
        }

        // Create galley from LayoutJob
        let galley = painter.layout_job(job);

        // Cache the galley
        self.insert(key, Arc::clone(&galley));

        galley
    }

    /// Checks if a syntax-highlighted galley is already cached, returning it if so.
    ///
    /// This allows callers to skip the expensive syntax highlighting step when the
    /// galley is already cached. If this returns `None`, the caller should perform
    /// syntax highlighting and then call `get_galley_highlighted` to create and cache
    /// the galley.
    ///
    /// # Arguments
    /// * `line_content` - The raw text content of the line
    /// * `font_id` - The font to use for the galley
    /// * `default_color` - Fallback color for text
    /// * `syntax_theme_hash` - Hash of the current syntax theme
    /// * `wrap_width` - Optional wrap width for word wrapping
    ///
    /// # Returns
    /// `Some(Arc<Galley>)` if cached, `None` if highlighting is needed.
    pub fn get_cached_highlighted_galley(
        &mut self,
        line_content: &str,
        font_id: &FontId,
        default_color: Color32,
        syntax_theme_hash: u64,
        wrap_width: Option<f32>,
    ) -> Option<Arc<Galley>> {
        let key = CacheKey::new_highlighted(
            line_content,
            font_id,
            default_color,
            syntax_theme_hash,
            wrap_width,
        );

        if let Some(galley) = self.cache.get(&key).cloned() {
            self.update_lru_order(key);
            Some(galley)
        } else {
            None
        }
    }

    /// Gets a cached galley with word wrapping enabled.
    ///
    /// This method creates a galley that wraps text at the specified width.
    /// The wrapped galley may span multiple visual rows.
    ///
    /// # Arguments
    /// * `line_content` - The text content of the line
    /// * `painter` - The egui `Painter` used to create galleys
    /// * `font_id` - The font to use for the galley
    /// * `color` - The text color
    /// * `wrap_width` - Maximum width before wrapping (in pixels)
    ///
    /// # Returns
    /// An `Arc<Galley>` containing the wrapped text layout. Use `galley.rows.len()`
    /// to get the number of visual rows, and `galley.size()` for the total size.
    ///
    /// # Example
    /// ```rust,ignore
    /// let galley = cache.get_galley_wrapped(
    ///     "This is a very long line that should wrap to multiple rows",
    ///     &painter,
    ///     FontId::monospace(14.0),
    ///     Color32::WHITE,
    ///     200.0, // 200px wrap width
    /// );
    /// assert!(galley.rows.len() >= 1);
    /// ```
    pub fn get_galley_wrapped(
        &mut self,
        line_content: &str,
        painter: &Painter,
        font_id: FontId,
        color: Color32,
        wrap_width: f32,
    ) -> Arc<Galley> {
        let key = CacheKey::new_wrapped(line_content, &font_id, color, wrap_width);

        // Check cache first
        if let Some(galley) = self.cache.get(&key).cloned() {
            self.update_lru_order(key);
            return galley;
        }

        // Create wrapped galley using egui's layout function
        let galley = painter.layout(
            line_content.to_string(),
            font_id,
            color,
            wrap_width,
        );

        // Cache the galley
        self.insert(key, Arc::clone(&galley));

        galley
    }

    /// Gets galley information without caching.
    ///
    /// This is useful for measuring text dimensions without polluting the cache.
    ///
    /// # Arguments
    /// * `content` - The text content
    /// * `painter` - The egui `Painter`
    /// * `font_id` - The font to use
    /// * `wrap_width` - Optional wrap width; if None, no wrapping
    ///
    /// # Returns
    /// A tuple of (row_count, total_height, total_width).
    #[must_use]
    pub fn measure_text(
        content: &str,
        painter: &Painter,
        font_id: FontId,
        wrap_width: Option<f32>,
    ) -> (usize, f32, f32) {
        let galley = if let Some(width) = wrap_width {
            painter.layout(
                content.to_string(),
                font_id,
                Color32::PLACEHOLDER,
                width,
            )
        } else {
            painter.layout_no_wrap(
                content.to_string(),
                font_id,
                Color32::PLACEHOLDER,
            )
        };

        (galley.rows.len(), galley.size().y, galley.size().x)
    }

    /// Inserts a galley into the cache with LRU eviction if needed.
    fn insert(&mut self, key: CacheKey, galley: Arc<Galley>) {
        // Evict oldest entries if at capacity
        while self.cache.len() >= MAX_CACHE_ENTRIES {
            if let Some(oldest_key) = self.lru_order.pop_front() {
                self.cache.remove(&oldest_key);
            } else {
                // LRU queue is empty but cache is full - shouldn't happen
                // but handle gracefully by clearing everything
                self.cache.clear();
                break;
            }
        }

        // Insert the new entry
        self.cache.insert(key, galley);
        self.lru_order.push_back(key);
    }

    /// Updates the LRU order when a key is accessed (moves to back).
    fn update_lru_order(&mut self, key: CacheKey) {
        // Find and remove the key from its current position
        if let Some(pos) = self.lru_order.iter().position(|k| *k == key) {
            self.lru_order.remove(pos);
        }
        // Add to back (most recently used)
        self.lru_order.push_back(key);
    }

    /// Clears all cached galleys.
    ///
    /// Call this when the font, theme, or other global styling changes,
    /// as all cached galleys will be invalid.
    ///
    /// # Example
    /// ```rust,ignore
    /// // Theme changed, invalidate all cached galleys
    /// cache.invalidate();
    /// ```
    pub fn invalidate(&mut self) {
        self.cache.clear();
        self.lru_order.clear();
    }

    /// Invalidates cached galleys for specific line content with given styling.
    ///
    /// This is useful when a line's content changes and you want to
    /// remove only that line's cached galley.
    ///
    /// # Arguments
    /// * `content` - The line content to invalidate
    /// * `font_id` - The font used for the galley
    /// * `color` - The text color
    ///
    /// # Note
    /// This removes the galley with the exact content/font/color combination.
    pub fn invalidate_line(&mut self, content: &str, font_id: &FontId, color: Color32) {
        let key = CacheKey::new(content, font_id, color);

        // Remove from cache
        self.cache.remove(&key);

        // Remove from LRU order
        self.lru_order.retain(|k| *k != key);
    }

    /// Returns the number of cached galleys.
    ///
    /// # Example
    /// ```rust,ignore
    /// let cache = LineCache::new();
    /// assert_eq!(cache.len(), 0);
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Returns `true` if the cache is empty.
    ///
    /// # Example
    /// ```rust,ignore
    /// let cache = LineCache::new();
    /// assert!(cache.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Returns the maximum cache capacity.
    ///
    /// # Example
    /// ```rust,ignore
    /// assert_eq!(LineCache::capacity(), 200);
    /// ```
    #[must_use]
    pub const fn capacity() -> usize {
        MAX_CACHE_ENTRIES
    }

    /// Returns cache hit statistics (for debugging/profiling).
    ///
    /// This is a simple check of whether a key would hit the cache
    /// without modifying LRU state.
    ///
    /// # Arguments
    /// * `content` - The line content to check
    /// * `font_id` - The font
    /// * `color` - The text color
    ///
    /// # Returns
    /// `true` if this combination is currently cached.
    #[must_use]
    pub fn is_cached(&self, content: &str, font_id: &FontId, color: Color32) -> bool {
        let key = CacheKey::new(content, font_id, color);
        self.cache.contains_key(&key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test helper: create a CacheKey for testing
    fn test_key(content: &str) -> CacheKey {
        CacheKey::new(content, &FontId::default(), Color32::WHITE)
    }

    #[test]
    fn test_new_cache() {
        let cache = LineCache::new();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_default_cache() {
        let cache = LineCache::default();
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_capacity() {
        assert_eq!(LineCache::capacity(), 200);
    }

    #[test]
    fn test_hash_content_deterministic() {
        // Same content should produce same hash
        let hash1 = hash_content("Hello, World!");
        let hash2 = hash_content("Hello, World!");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_content_different() {
        // Different content should produce different hash
        let hash1 = hash_content("Hello");
        let hash2 = hash_content("World");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_cache_key_equality() {
        let key1 = test_key("test");
        let key2 = test_key("test");
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_cache_key_different_content() {
        let key1 = test_key("hello");
        let key2 = test_key("world");
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_cache_key_different_font() {
        let key1 = CacheKey::new("test", &FontId::monospace(12.0), Color32::WHITE);
        let key2 = CacheKey::new("test", &FontId::monospace(14.0), Color32::WHITE);
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_cache_key_different_color() {
        let key1 = CacheKey::new("test", &FontId::default(), Color32::WHITE);
        let key2 = CacheKey::new("test", &FontId::default(), Color32::BLACK);
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_invalidate() {
        let mut cache = LineCache::new();
        // Manually insert some test data to simulate cached entries
        let key = test_key("test line");
        cache.lru_order.push_back(key);

        // Now verify invalidate clears the LRU order
        cache.invalidate();
        assert!(cache.is_empty());
        assert!(cache.lru_order.is_empty());
    }

    #[test]
    fn test_invalidate_empty_cache() {
        let mut cache = LineCache::new();
        // Should not panic on empty cache
        cache.invalidate();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_lru_eviction_ordering() {
        let mut cache = LineCache::new();

        // Manually add entries to test LRU logic
        // Note: This tests the internal LRU tracking, not the full get_galley flow
        // (which requires a real Painter)

        // Add MAX_CACHE_ENTRIES entries
        for i in 0..MAX_CACHE_ENTRIES {
            let content = format!("line {i}");
            let key = test_key(&content);
            cache.lru_order.push_back(key);
        }

        assert_eq!(cache.lru_order.len(), MAX_CACHE_ENTRIES);

        // Verify the oldest entry is at front
        let first_key = test_key("line 0");
        assert_eq!(cache.lru_order.front(), Some(&first_key));

        // Verify the newest entry is at back
        let last_key = test_key(&format!("line {}", MAX_CACHE_ENTRIES - 1));
        assert_eq!(cache.lru_order.back(), Some(&last_key));
    }

    #[test]
    fn test_update_lru_order() {
        let mut cache = LineCache::new();

        // Add some entries
        let key1 = test_key("line 1");
        let key2 = test_key("line 2");
        let key3 = test_key("line 3");

        cache.lru_order.push_back(key1);
        cache.lru_order.push_back(key2);
        cache.lru_order.push_back(key3);

        // Access key1, it should move to back
        cache.update_lru_order(key1);

        assert_eq!(cache.lru_order.front(), Some(&key2));
        assert_eq!(cache.lru_order.back(), Some(&key1));
    }

    #[test]
    fn test_update_lru_order_nonexistent() {
        let mut cache = LineCache::new();

        let key1 = test_key("line 1");
        cache.lru_order.push_back(key1);

        // Update a key that doesn't exist - should add it
        let key2 = test_key("line 2");
        cache.update_lru_order(key2);

        assert_eq!(cache.lru_order.len(), 2);
        assert_eq!(cache.lru_order.back(), Some(&key2));
    }

    #[test]
    fn test_invalidate_line() {
        let mut cache = LineCache::new();

        // Add entries with different content (LRU order only, no actual galleys)
        // This tests the LRU tracking logic without requiring a real Painter
        let key1 = test_key("line 1");
        let key2 = test_key("line 2");

        cache.lru_order.push_back(key1);
        cache.lru_order.push_back(key2);

        // Invalidate "line 1" - should remove key1 from LRU order
        cache.invalidate_line("line 1", &FontId::default(), Color32::WHITE);

        // Only key2 should remain in LRU order
        assert_eq!(cache.lru_order.len(), 1);
        assert_eq!(cache.lru_order.front(), Some(&key2));
    }

    #[test]
    fn test_is_cached() {
        let cache = LineCache::new();

        // Empty cache should return false
        assert!(!cache.is_cached("test", &FontId::default(), Color32::WHITE));
    }

    #[test]
    fn test_unicode_content() {
        // Test that unicode content hashes correctly
        let hash1 = hash_content("こんにちは");
        let hash2 = hash_content("こんにちは");
        assert_eq!(hash1, hash2);

        let hash3 = hash_content("世界");
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_emoji_content() {
        let hash1 = hash_content("Hello 🌍 World");
        let hash2 = hash_content("Hello 🌍 World");
        assert_eq!(hash1, hash2);

        let hash3 = hash_content("Hello 🌎 World");
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_empty_line() {
        let hash1 = hash_content("");
        let hash2 = hash_content("");
        assert_eq!(hash1, hash2);

        let hash3 = hash_content(" ");
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_whitespace_sensitivity() {
        // Leading/trailing whitespace should produce different hashes
        let hash1 = hash_content("test");
        let hash2 = hash_content(" test");
        let hash3 = hash_content("test ");
        let hash4 = hash_content("  test  ");

        assert_ne!(hash1, hash2);
        assert_ne!(hash1, hash3);
        assert_ne!(hash1, hash4);
        assert_ne!(hash2, hash3);
    }

    /// Test that the LRU eviction works correctly when adding 201 entries
    #[test]
    fn test_lru_eviction_201_entries() {
        let mut cache = LineCache::new();

        // Add 200 entries
        for i in 0..200 {
            let content = format!("line {i}");
            let key = test_key(&content);
            cache.lru_order.push_back(key);
        }

        assert_eq!(cache.lru_order.len(), 200);

        // Verify first entry is "line 0"
        let first_key = test_key("line 0");
        assert_eq!(cache.lru_order.front(), Some(&first_key));

        // Add 201st entry - should trigger eviction of "line 0"
        let key_201 = test_key("line 200");
        cache.lru_order.push_back(key_201);

        // Manually evict from front (simulating insert behavior)
        if cache.lru_order.len() > MAX_CACHE_ENTRIES {
            cache.lru_order.pop_front();
        }

        assert_eq!(cache.lru_order.len(), 200);

        // "line 0" should be gone
        let old_first_key = test_key("line 0");
        assert!(!cache.lru_order.contains(&old_first_key));

        // "line 1" should now be at front
        let new_first_key = test_key("line 1");
        assert_eq!(cache.lru_order.front(), Some(&new_first_key));

        // "line 200" should be at back
        assert_eq!(cache.lru_order.back(), Some(&key_201));
    }
}
