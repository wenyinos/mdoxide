//! TextBuffer module for efficient text storage and manipulation.
//!
//! This module provides a `TextBuffer` struct that wraps `ropey::Rope` to provide
//! O(log n) text operations for large files, efficient line indexing, and
//! memory-efficient storage.

use ropey::Rope;
use std::borrow::Cow;

/// A text buffer backed by a rope data structure for efficient large-file editing.
///
/// `TextBuffer` wraps `ropey::Rope` to provide:
/// - O(log n) insert and delete operations
/// - O(log n) line/character index conversions
/// - Memory-efficient storage (target: <50MB for 4MB file)
/// - Unicode-correct text handling
#[derive(Debug, Clone)]
pub struct TextBuffer {
    rope: Rope,
}

impl Default for TextBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl TextBuffer {
    /// Creates a new empty `TextBuffer`.
    ///
    /// # Example
    /// ```
    /// let buffer = TextBuffer::new();
    /// assert_eq!(buffer.len(), 0);
    /// assert_eq!(buffer.line_count(), 1); // Empty buffer has one (empty) line
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self { rope: Rope::new() }
    }

    /// Creates a `TextBuffer` from a string slice.
    ///
    /// # Arguments
    /// * `content` - The initial text content for the buffer
    ///
    /// # Example
    /// ```
    /// let buffer = TextBuffer::from_string("Hello\nWorld");
    /// assert_eq!(buffer.line_count(), 2);
    /// ```
    #[must_use]
    pub fn from_string(content: &str) -> Self {
        Self {
            rope: Rope::from_str(content),
        }
    }

    /// Inserts text at the specified character position.
    ///
    /// # Arguments
    /// * `pos` - The character index at which to insert the text
    /// * `text` - The text to insert
    ///
    /// # Panics
    /// Panics if `pos` is out of bounds (i.e., `pos > len()`).
    ///
    /// # Performance
    /// Runs in O(M + log N) time, where N is the buffer length and M is the
    /// length of the inserted text.
    ///
    /// # Example
    /// ```
    /// let mut buffer = TextBuffer::from_string("Hello World");
    /// buffer.insert(5, " Beautiful");
    /// // Buffer now contains "Hello Beautiful World"
    /// ```
    pub fn insert(&mut self, pos: usize, text: &str) {
        self.rope.insert(pos, text);
    }

    /// Removes a range of text starting at the specified position.
    ///
    /// # Arguments
    /// * `pos` - The starting character index of the removal
    /// * `len` - The number of characters to remove
    ///
    /// # Panics
    /// Panics if the range `pos..pos+len` is out of bounds.
    ///
    /// # Performance
    /// Runs in O(M + log N) time, where N is the buffer length and M is the
    /// length of the removed range.
    ///
    /// # Example
    /// ```
    /// let mut buffer = TextBuffer::from_string("Hello Beautiful World");
    /// buffer.remove(5, 10); // Remove " Beautiful"
    /// // Buffer now contains "Hello World"
    /// ```
    pub fn remove(&mut self, pos: usize, len: usize) {
        if len > 0 {
            self.rope.remove(pos..pos + len);
        }
    }

    /// Returns a slice of text as a String.
    ///
    /// # Arguments
    /// * `start_char` - Start character position (inclusive)
    /// * `end_char` - End character position (exclusive)
    ///
    /// # Returns
    /// The text content between the specified character positions.
    ///
    /// # Performance
    /// O(log N + M) where M is the length of the extracted slice.
    /// Uses the rope's native slicing which is efficient.
    ///
    /// # Example
    /// ```
    /// let buffer = TextBuffer::from_string("Hello, World!");
    /// assert_eq!(buffer.slice(0, 5), "Hello");
    /// assert_eq!(buffer.slice(7, 12), "World");
    /// ```
    #[must_use]
    pub fn slice(&self, start_char: usize, end_char: usize) -> String {
        let start = start_char.min(self.len());
        let end = end_char.min(self.len());
        if start >= end {
            return String::new();
        }
        self.rope.slice(start..end).to_string()
    }

    /// Returns the content of the specified line.
    ///
    /// # Arguments
    /// * `line_idx` - The zero-indexed line number
    ///
    /// # Returns
    /// A `Cow<str>` containing the line content (including trailing newline if present).
    /// Returns `Cow::Borrowed` when the line is stored contiguously in memory,
    /// otherwise returns `Cow::Owned`.
    ///
    /// # Panics
    /// Panics if `line_idx` is out of bounds (i.e., `line_idx >= line_count()`).
    ///
    /// # Performance
    /// Runs in O(log N) time.
    ///
    /// # Example
    /// ```
    /// let buffer = TextBuffer::from_string("Line 1\nLine 2\nLine 3");
    /// assert_eq!(buffer.line(1).trim_end(), "Line 2");
    /// ```
    #[must_use]
    pub fn line(&self, line_idx: usize) -> Cow<'_, str> {
        self.rope.line(line_idx).into()
    }

    /// Returns the total number of lines in the buffer.
    ///
    /// An empty buffer is considered to have one (empty) line.
    /// A trailing newline adds an additional empty line.
    ///
    /// # Performance
    /// Runs in O(1) time.
    ///
    /// # Example
    /// ```
    /// let buffer = TextBuffer::from_string("Line 1\nLine 2");
    /// assert_eq!(buffer.line_count(), 2);
    ///
    /// let buffer_with_trailing = TextBuffer::from_string("Line 1\nLine 2\n");
    /// assert_eq!(buffer_with_trailing.line_count(), 3);
    /// ```
    #[must_use]
    pub fn line_count(&self) -> usize {
        self.rope.len_lines()
    }

    /// Converts a line index to a character offset.
    ///
    /// # Arguments
    /// * `line_idx` - The zero-indexed line number
    ///
    /// # Returns
    /// The character offset where the specified line begins.
    ///
    /// # Panics
    /// Panics if `line_idx` is out of bounds.
    ///
    /// # Performance
    /// Runs in O(log N) time.
    ///
    /// # Example
    /// ```
    /// let buffer = TextBuffer::from_string("Hello\nWorld");
    /// assert_eq!(buffer.line_to_char(0), 0);
    /// assert_eq!(buffer.line_to_char(1), 6); // After "Hello\n"
    /// ```
    #[must_use]
    pub fn line_to_char(&self, line_idx: usize) -> usize {
        self.rope.line_to_char(line_idx)
    }

    /// Converts a character offset to a line index.
    ///
    /// # Arguments
    /// * `char_idx` - The character offset
    ///
    /// # Returns
    /// The zero-indexed line number containing the specified character.
    ///
    /// # Panics
    /// Panics if `char_idx` is out of bounds (i.e., `char_idx > len()`).
    /// Note: `char_idx` can be one-past-the-end, which returns the last line index.
    ///
    /// # Performance
    /// Runs in O(log N) time.
    ///
    /// # Example
    /// ```
    /// let buffer = TextBuffer::from_string("Hello\nWorld");
    /// assert_eq!(buffer.char_to_line(0), 0);   // 'H' is on line 0
    /// assert_eq!(buffer.char_to_line(5), 0);   // '\n' is on line 0
    /// assert_eq!(buffer.char_to_line(6), 1);   // 'W' is on line 1
    /// ```
    #[must_use]
    pub fn char_to_line(&self, char_idx: usize) -> usize {
        self.rope.char_to_line(char_idx)
    }

    /// Returns the total number of characters in the buffer.
    ///
    /// # Performance
    /// Runs in O(1) time.
    ///
    /// # Example
    /// ```
    /// let buffer = TextBuffer::from_string("Hello\nWorld");
    /// assert_eq!(buffer.len(), 11); // "Hello\nWorld" has 11 characters
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.rope.len_chars()
    }

    /// Returns `true` if the buffer contains no characters.
    ///
    /// # Performance
    /// Runs in O(1) time.
    ///
    /// # Example
    /// ```
    /// let empty = TextBuffer::new();
    /// assert!(empty.is_empty());
    ///
    /// let non_empty = TextBuffer::from_string("Hello");
    /// assert!(!non_empty.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rope.len_chars() == 0
    }

    /// Returns a reference to the underlying `Rope`.
    ///
    /// This is useful for advanced operations that aren't exposed through
    /// the `TextBuffer` API.
    #[must_use]
    pub fn rope(&self) -> &Rope {
        &self.rope
    }

    /// Returns a mutable reference to the underlying `Rope`.
    ///
    /// Use with caution - direct modifications bypass any future
    /// tracking mechanisms (e.g., undo/redo history).
    pub fn rope_mut(&mut self) -> &mut Rope {
        &mut self.rope
    }

    /// Returns the number of bytes in the buffer.
    ///
    /// # Performance
    /// Runs in O(1) time.
    #[must_use]
    pub fn len_bytes(&self) -> usize {
        self.rope.len_bytes()
    }

    /// Gets the line at the specified index, returning `None` if out of bounds.
    ///
    /// This is a non-panicking version of `line()`.
    ///
    /// # Arguments
    /// * `line_idx` - The zero-indexed line number
    ///
    /// # Returns
    /// `Some(Cow<str>)` if the line exists, `None` otherwise.
    ///
    /// # Performance
    /// Runs in O(log N) time.
    #[must_use]
    pub fn get_line(&self, line_idx: usize) -> Option<Cow<'_, str>> {
        self.rope.get_line(line_idx).map(|slice| slice.into())
    }

    /// Attempts to convert a line index to a character offset without panicking.
    ///
    /// # Arguments
    /// * `line_idx` - The zero-indexed line number
    ///
    /// # Returns
    /// `Some(usize)` with the character offset if the line index is valid,
    /// `None` otherwise.
    ///
    /// # Performance
    /// Runs in O(log N) time.
    #[must_use]
    pub fn try_line_to_char(&self, line_idx: usize) -> Option<usize> {
        self.rope.try_line_to_char(line_idx).ok()
    }

    /// Attempts to convert a character offset to a line index without panicking.
    ///
    /// # Arguments
    /// * `char_idx` - The character offset
    ///
    /// # Returns
    /// `Some(usize)` with the line index if the character index is valid,
    /// `None` otherwise.
    ///
    /// # Performance
    /// Runs in O(log N) time.
    #[must_use]
    pub fn try_char_to_line(&self, char_idx: usize) -> Option<usize> {
        self.rope.try_char_to_line(char_idx).ok()
    }

    /// Converts a byte offset to a line index.
    ///
    /// # Arguments
    /// * `byte_idx` - The byte offset
    ///
    /// # Returns
    /// The zero-indexed line number containing the specified byte.
    ///
    /// # Panics
    /// Panics if `byte_idx` is out of bounds (i.e., `byte_idx > len_bytes()`).
    /// Note: `byte_idx` can be one-past-the-end, which returns the last line index.
    ///
    /// # Performance
    /// Runs in O(log N) time - uses the rope's native byte_to_line method.
    ///
    /// # Example
    /// ```
    /// let buffer = TextBuffer::from_string("Hello\nWorld");
    /// assert_eq!(buffer.byte_to_line(0), 0);   // 'H' is on line 0
    /// assert_eq!(buffer.byte_to_line(5), 0);   // '\n' is on line 0
    /// assert_eq!(buffer.byte_to_line(6), 1);   // 'W' is on line 1
    /// ```
    #[must_use]
    pub fn byte_to_line(&self, byte_idx: usize) -> usize {
        self.rope.byte_to_line(byte_idx)
    }

    /// Attempts to convert a byte offset to a line index without panicking.
    ///
    /// # Arguments
    /// * `byte_idx` - The byte offset
    ///
    /// # Returns
    /// `Some(usize)` with the line index if the byte index is valid,
    /// `None` otherwise.
    ///
    /// # Performance
    /// Runs in O(log N) time - uses the rope's native byte_to_line method.
    #[must_use]
    pub fn try_byte_to_line(&self, byte_idx: usize) -> Option<usize> {
        self.rope.try_byte_to_line(byte_idx).ok()
    }

    /// Converts a byte offset to a character offset.
    ///
    /// # Arguments
    /// * `byte_idx` - The byte offset
    ///
    /// # Returns
    /// The character index corresponding to the given byte position.
    /// If the byte is in the middle of a multi-byte char, returns the index
    /// of the char that the byte belongs to.
    ///
    /// # Panics
    /// Panics if `byte_idx` is out of bounds (i.e., `byte_idx > len_bytes()`).
    ///
    /// # Performance
    /// Runs in O(log N) time - uses the rope's native byte_to_char method.
    #[must_use]
    pub fn byte_to_char(&self, byte_idx: usize) -> usize {
        self.rope.byte_to_char(byte_idx)
    }

    /// Attempts to convert a byte offset to a character offset without panicking.
    ///
    /// # Arguments
    /// * `byte_idx` - The byte offset
    ///
    /// # Returns
    /// `Some(usize)` with the character index if the byte index is valid,
    /// `None` otherwise.
    ///
    /// # Performance
    /// Runs in O(log N) time.
    #[must_use]
    pub fn try_byte_to_char(&self, byte_idx: usize) -> Option<usize> {
        self.rope.try_byte_to_char(byte_idx).ok()
    }

    /// Converts a character offset to a byte offset.
    ///
    /// # Arguments
    /// * `char_idx` - The character offset
    ///
    /// # Returns
    /// The byte index corresponding to the given character position.
    ///
    /// # Panics
    /// Panics if `char_idx` is out of bounds (i.e., `char_idx > len()`).
    ///
    /// # Performance
    /// Runs in O(log N) time - uses the rope's native char_to_byte method.
    #[must_use]
    pub fn char_to_byte(&self, char_idx: usize) -> usize {
        self.rope.char_to_byte(char_idx)
    }

    /// Attempts to convert a character offset to a byte offset without panicking.
    ///
    /// # Arguments
    /// * `char_idx` - The character offset
    ///
    /// # Returns
    /// `Some(usize)` with the byte index if the character index is valid,
    /// `None` otherwise.
    ///
    /// # Performance
    /// Runs in O(log N) time.
    #[must_use]
    pub fn try_char_to_byte(&self, char_idx: usize) -> Option<usize> {
        self.rope.try_char_to_byte(char_idx).ok()
    }

    /// Extracts a range of lines as a String.
    ///
    /// This method is useful for windowed operations that need to process
    /// only a subset of lines (e.g., bracket matching around cursor).
    ///
    /// # Arguments
    /// * `start_line` - The starting line index (inclusive, 0-indexed)
    /// * `end_line` - The ending line index (exclusive, 0-indexed)
    ///
    /// # Returns
    /// A tuple containing:
    /// - The extracted text as a String
    /// - The character offset where the slice starts in the full document
    ///
    /// # Performance
    /// Runs in O(log N + M) time, where N is the total buffer size and M is
    /// the size of the extracted slice. This is much better than to_string()
    /// which is always O(N).
    ///
    /// # Example
    /// ```
    /// let buffer = TextBuffer::from_string("Line 0\nLine 1\nLine 2\nLine 3");
    /// let (text, start_char) = buffer.slice_lines_to_string(1, 3);
    /// // text contains "Line 1\nLine 2\n", start_char is 7
    /// ```
    #[must_use]
    pub fn slice_lines_to_string(&self, start_line: usize, end_line: usize) -> (String, usize) {
        let total_lines = self.line_count();
        let start_line = start_line.min(total_lines);
        let end_line = end_line.min(total_lines);

        if start_line >= end_line {
            let start_char = self.try_line_to_char(start_line).unwrap_or(0);
            return (String::new(), start_char);
        }

        let start_char = self.rope.line_to_char(start_line);
        let end_char = if end_line >= total_lines {
            self.rope.len_chars()
        } else {
            self.rope.line_to_char(end_line)
        };

        let slice = self.rope.slice(start_char..end_char);
        (slice.to_string(), start_char)
    }
}

impl From<&str> for TextBuffer {
    fn from(s: &str) -> Self {
        Self::from_string(s)
    }
}

impl From<String> for TextBuffer {
    fn from(s: String) -> Self {
        Self::from_string(&s)
    }
}

impl std::fmt::Display for TextBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.rope)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty_buffer() {
        let buffer = TextBuffer::new();
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
        assert_eq!(buffer.line_count(), 1); // Empty buffer has one empty line
    }

    #[test]
    fn test_from_string() {
        let buffer = TextBuffer::from_string("Hello\nWorld");
        assert_eq!(buffer.len(), 11);
        assert_eq!(buffer.line_count(), 2);
        assert!(!buffer.is_empty());
    }

    #[test]
    fn test_insert() {
        let mut buffer = TextBuffer::from_string("HelloWorld");
        buffer.insert(5, " ");
        assert_eq!(buffer.to_string(), "Hello World");
        assert_eq!(buffer.len(), 11);
    }

    #[test]
    fn test_insert_at_start() {
        let mut buffer = TextBuffer::from_string("World");
        buffer.insert(0, "Hello ");
        assert_eq!(buffer.to_string(), "Hello World");
    }

    #[test]
    fn test_insert_at_end() {
        let mut buffer = TextBuffer::from_string("Hello");
        buffer.insert(5, " World");
        assert_eq!(buffer.to_string(), "Hello World");
    }

    #[test]
    fn test_remove() {
        let mut buffer = TextBuffer::from_string("Hello World");
        buffer.remove(5, 6); // Remove " World"
        assert_eq!(buffer.to_string(), "Hello");
    }

    #[test]
    fn test_remove_zero_length() {
        let mut buffer = TextBuffer::from_string("Hello");
        buffer.remove(2, 0); // Should do nothing
        assert_eq!(buffer.to_string(), "Hello");
    }

    #[test]
    fn test_line() {
        let buffer = TextBuffer::from_string("Line 1\nLine 2\nLine 3");
        assert_eq!(buffer.line(0).as_ref(), "Line 1\n");
        assert_eq!(buffer.line(1).as_ref(), "Line 2\n");
        assert_eq!(buffer.line(2).as_ref(), "Line 3");
    }

    #[test]
    fn test_get_line() {
        let buffer = TextBuffer::from_string("Line 1\nLine 2");
        assert!(buffer.get_line(0).is_some());
        assert!(buffer.get_line(1).is_some());
        assert!(buffer.get_line(2).is_none());
        assert!(buffer.get_line(100).is_none());
    }

    #[test]
    fn test_line_count() {
        let buffer = TextBuffer::from_string("Line 1\nLine 2\nLine 3");
        assert_eq!(buffer.line_count(), 3);

        let buffer_with_trailing = TextBuffer::from_string("Line 1\n");
        assert_eq!(buffer_with_trailing.line_count(), 2);
    }

    #[test]
    fn test_line_to_char() {
        let buffer = TextBuffer::from_string("Hello\nWorld\nTest");
        assert_eq!(buffer.line_to_char(0), 0);
        assert_eq!(buffer.line_to_char(1), 6); // After "Hello\n"
        assert_eq!(buffer.line_to_char(2), 12); // After "Hello\nWorld\n"
    }

    #[test]
    fn test_char_to_line() {
        let buffer = TextBuffer::from_string("Hello\nWorld\nTest");
        assert_eq!(buffer.char_to_line(0), 0); // 'H' on line 0
        assert_eq!(buffer.char_to_line(5), 0); // '\n' on line 0
        assert_eq!(buffer.char_to_line(6), 1); // 'W' on line 1
        assert_eq!(buffer.char_to_line(11), 1); // '\n' on line 1
        assert_eq!(buffer.char_to_line(12), 2); // 'T' on line 2
    }

    #[test]
    fn test_try_methods() {
        let buffer = TextBuffer::from_string("Hello\nWorld");

        // Valid indices
        assert_eq!(buffer.try_line_to_char(0), Some(0));
        assert_eq!(buffer.try_line_to_char(1), Some(6));
        assert_eq!(buffer.try_char_to_line(0), Some(0));
        assert_eq!(buffer.try_char_to_line(6), Some(1));

        // Invalid indices
        assert_eq!(buffer.try_line_to_char(100), None);
        assert_eq!(buffer.try_char_to_line(100), None);
    }

    #[test]
    fn test_len_bytes() {
        let buffer = TextBuffer::from_string("Hello");
        assert_eq!(buffer.len_bytes(), 5);

        // Test with multi-byte characters
        let buffer_unicode = TextBuffer::from_string("こんにちは"); // "Hello" in Japanese
        assert_eq!(buffer_unicode.len(), 5); // 5 characters
        assert_eq!(buffer_unicode.len_bytes(), 15); // 15 bytes (3 bytes per character)
    }

    #[test]
    fn test_from_traits() {
        let buffer1: TextBuffer = "Hello".into();
        assert_eq!(buffer1.to_string(), "Hello");

        let buffer2: TextBuffer = String::from("World").into();
        assert_eq!(buffer2.to_string(), "World");
    }

    #[test]
    fn test_display() {
        let buffer = TextBuffer::from_string("Hello World");
        assert_eq!(format!("{buffer}"), "Hello World");
    }

    #[test]
    fn test_default() {
        let buffer = TextBuffer::default();
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_multiline_operations() {
        let mut buffer = TextBuffer::from_string("Line 1\nLine 2\nLine 3");

        // Insert a new line
        let pos = buffer.line_to_char(1);
        buffer.insert(pos, "Inserted\n");

        assert_eq!(buffer.line_count(), 4);
        assert_eq!(buffer.line(1).trim_end(), "Inserted");
        assert_eq!(buffer.line(2).trim_end(), "Line 2");
    }

    #[test]
    fn test_unicode_handling() {
        // "Hello " (6) + "世界" (2) + " " (1) + "🌍" (1) = 10 characters
        let buffer = TextBuffer::from_string("Hello 世界 🌍");
        assert_eq!(buffer.len(), 10);
        assert_eq!(buffer.line_count(), 1);

        // Test line with unicode
        // "こんにちは" (5) + "\n" (1) + "世界" (2) = 8 characters, 2 lines
        let buffer_multiline = TextBuffer::from_string("こんにちは\n世界");
        assert_eq!(buffer_multiline.line_count(), 2);
        assert_eq!(buffer_multiline.line(0).trim_end(), "こんにちは");
        assert_eq!(buffer_multiline.line(1).as_ref(), "世界");
    }

    #[test]
    fn test_rope_access() {
        let mut buffer = TextBuffer::from_string("Test");

        // Test read access
        assert_eq!(buffer.rope().len_chars(), 4);

        // Test write access
        buffer.rope_mut().insert(0, "Pre ");
        assert_eq!(buffer.to_string(), "Pre Test");
    }

    #[test]
    fn test_byte_to_line() {
        let buffer = TextBuffer::from_string("Hello\nWorld\nTest");
        assert_eq!(buffer.byte_to_line(0), 0); // 'H' on line 0
        assert_eq!(buffer.byte_to_line(5), 0); // '\n' on line 0
        assert_eq!(buffer.byte_to_line(6), 1); // 'W' on line 1
        assert_eq!(buffer.byte_to_line(11), 1); // '\n' on line 1
        assert_eq!(buffer.byte_to_line(12), 2); // 'T' on line 2
    }

    #[test]
    fn test_byte_to_line_unicode() {
        // Japanese text: "こんにちは" is 5 characters but 15 bytes (3 bytes each)
        let buffer = TextBuffer::from_string("こんにちは\n世界");
        // First line: 15 bytes + 1 newline = 16 bytes
        assert_eq!(buffer.byte_to_line(0), 0); // First byte of first char
        assert_eq!(buffer.byte_to_line(14), 0); // Last byte of 5th char
        assert_eq!(buffer.byte_to_line(15), 0); // '\n' is still on line 0
        assert_eq!(buffer.byte_to_line(16), 1); // First byte of 世 is on line 1
    }

    #[test]
    fn test_try_byte_to_line() {
        let buffer = TextBuffer::from_string("Hello\nWorld");

        // Valid indices
        assert_eq!(buffer.try_byte_to_line(0), Some(0));
        assert_eq!(buffer.try_byte_to_line(6), Some(1));
        assert_eq!(buffer.try_byte_to_line(11), Some(1)); // One-past-end

        // Invalid indices
        assert_eq!(buffer.try_byte_to_line(100), None);
    }

    #[test]
    fn test_byte_to_char() {
        let buffer = TextBuffer::from_string("Hello");
        assert_eq!(buffer.byte_to_char(0), 0);
        assert_eq!(buffer.byte_to_char(3), 3);
        assert_eq!(buffer.byte_to_char(5), 5); // One-past-end
    }

    #[test]
    fn test_byte_to_char_unicode() {
        // Each Japanese char is 3 bytes
        let buffer = TextBuffer::from_string("こんにちは"); // 5 chars, 15 bytes
        assert_eq!(buffer.byte_to_char(0), 0); // First char
        assert_eq!(buffer.byte_to_char(3), 1); // Second char starts at byte 3
        assert_eq!(buffer.byte_to_char(6), 2); // Third char starts at byte 6
        assert_eq!(buffer.byte_to_char(1), 0); // Middle of first char returns that char
    }

    #[test]
    fn test_try_byte_to_char() {
        let buffer = TextBuffer::from_string("Hello");

        // Valid indices
        assert_eq!(buffer.try_byte_to_char(0), Some(0));
        assert_eq!(buffer.try_byte_to_char(5), Some(5)); // One-past-end

        // Invalid indices
        assert_eq!(buffer.try_byte_to_char(100), None);
    }

    // Performance test - commented out by default as it takes time
    // Uncomment to run: cargo test test_large_file -- --ignored
    #[test]
    #[ignore]
    fn test_large_file() {
        // Create a 4MB string
        let line = "This is a test line with some content for testing.\n";
        let line_count = (4 * 1024 * 1024) / line.len();
        let large_content: String = (0..line_count).map(|_| line).collect();

        let buffer = TextBuffer::from_string(&large_content);

        // Verify basic properties
        assert!(buffer.len() > 4_000_000);
        assert_eq!(buffer.line_count(), line_count + 1);

        // Test insert operation
        let mut buffer_mut = buffer.clone();
        let middle = buffer_mut.len() / 2;
        buffer_mut.insert(middle, "INSERTED TEXT");
        assert!(buffer_mut.len() > buffer.len());

        // Test remove operation
        buffer_mut.remove(middle, 13);
        assert_eq!(buffer_mut.len(), buffer.len());
    }

    #[test]
    fn test_char_to_byte() {
        let buffer = TextBuffer::from_string("Hello\nWorld");
        assert_eq!(buffer.char_to_byte(0), 0);
        assert_eq!(buffer.char_to_byte(5), 5); // '\n'
        assert_eq!(buffer.char_to_byte(6), 6); // 'W'
        assert_eq!(buffer.char_to_byte(11), 11); // End of buffer
    }

    #[test]
    fn test_char_to_byte_unicode() {
        // Japanese: "こんにちは" = 5 chars, 15 bytes
        let buffer = TextBuffer::from_string("こんにちは");
        assert_eq!(buffer.char_to_byte(0), 0);
        assert_eq!(buffer.char_to_byte(1), 3);
        assert_eq!(buffer.char_to_byte(2), 6);
        assert_eq!(buffer.char_to_byte(5), 15); // End of buffer
    }

    #[test]
    fn test_try_char_to_byte() {
        let buffer = TextBuffer::from_string("Hello");
        assert_eq!(buffer.try_char_to_byte(0), Some(0));
        assert_eq!(buffer.try_char_to_byte(5), Some(5)); // End of buffer
        assert_eq!(buffer.try_char_to_byte(100), None); // Out of bounds
    }

    #[test]
    fn test_slice_lines_to_string() {
        let buffer = TextBuffer::from_string("Line 0\nLine 1\nLine 2\nLine 3");

        // Extract middle lines
        let (text, start_char) = buffer.slice_lines_to_string(1, 3);
        assert_eq!(text, "Line 1\nLine 2\n");
        assert_eq!(start_char, 7); // "Line 0\n" = 7 chars

        // Extract from start
        let (text, start_char) = buffer.slice_lines_to_string(0, 2);
        assert_eq!(text, "Line 0\nLine 1\n");
        assert_eq!(start_char, 0);

        // Extract to end
        let (text, start_char) = buffer.slice_lines_to_string(2, 4);
        assert_eq!(text, "Line 2\nLine 3");
        assert_eq!(start_char, 14); // "Line 0\nLine 1\n" = 14 chars

        // Out of bounds clamped
        let (text, start_char) = buffer.slice_lines_to_string(3, 100);
        assert_eq!(text, "Line 3");
        assert_eq!(start_char, 21);

        // Empty range
        let (text, _) = buffer.slice_lines_to_string(2, 2);
        assert_eq!(text, "");
    }

    #[test]
    fn test_slice_lines_to_string_single_line() {
        let buffer = TextBuffer::from_string("Just one line");
        let (text, start_char) = buffer.slice_lines_to_string(0, 1);
        assert_eq!(text, "Just one line");
        assert_eq!(start_char, 0);
    }

    #[test]
    fn test_slice_lines_to_string_empty_buffer() {
        let buffer = TextBuffer::new();
        let (text, start_char) = buffer.slice_lines_to_string(0, 10);
        assert_eq!(text, "");
        assert_eq!(start_char, 0);
    }
}
