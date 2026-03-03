![](assets/20260302-164245-20260302-164117-ferritelogo.jpg)

# Wikilinks Test File

This file tests the `[[wikilinks]]` feature in Ferrite's rendered/split view.

---

## Basic Wikilinks

A simple wikilink: [[wikilink-target-a]]

Another wikilink: [[wikilink-target-b]]

---

## Wikilinks with Display Text

Link with custom text: [[wikilink-target-a|Click here for Target A]]

Another display text link: [[wikilink-target-b|Visit Target B]]

---

## Spaces in Filenames

A file with spaces: [[My Wikilink Note]]

---

## Multiple Wikilinks in One Line

See [[wikilink-target-a]] and [[wikilink-target-b|Target B]] for details.

---

## Broken / Missing Links

This links to a file that does not exist: [[non-existent-file]]

Another missing target: [[this-does-not-exist|Broken Link]]

These should appear dimmed/red with strikethrough in rendered view.

---

## Edge Cases

### Empty wikilink (should render as plain text)
An empty wikilink: [[]]

### Unclosed wikilink (should render as plain text)
An unclosed bracket: [[not closed

### Wikilink in a paragraph
This paragraph has a [[wikilink-target-a|wikilink]] embedded in the middle of a sentence, which should render inline.

### Multiple wikilinks back to back
[[wikilink-target-a]][[wikilink-target-b]]

### Wikilink with .md extension
Explicit extension: [[wikilink-target-a.md]]

---

## Wikilinks Inside Other Elements

### In a list
- First item with [[wikilink-target-a]]
- Second item linking to [[wikilink-target-b|Target B]]
- Third item with a [[non-existent-file|broken link]]

### In a blockquote
> This quote references [[wikilink-target-a]] for more context.

### In bold/italic
**Bold text with [[wikilink-target-b]]**

*Italic with [[wikilink-target-a|a link]]*

---

## Testing Checklist

1. [ ] `[[wikilink-target-a]]` renders as clickable link, clicking opens `wikilink-target-a.md`
2. [ ] `[[wikilink-target-b|Visit Target B]]` shows "Visit Target B", navigates to `wikilink-target-b.md`
3. [ ] `[[My Wikilink Note]]` resolves to `My Wikilink Note.md`
4. [ ] `[[non-existent-file]]` appears dimmed/red (broken link style)
5. [ ] `[[]]` renders as plain text `[[]]`
6. [ ] `[[not closed` renders as plain text
7. [ ] Wikilinks in lists, blockquotes, bold/italic all work
8. [ ] Hovering a wikilink shows tooltip with target path and status
9. [ ] Clicking a broken link shows an error toast (no crash)
