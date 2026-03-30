# GitHub Issue Reply Drafts - v0.2.8

---

## Reply to Issue #105 - Very Slow Rendering with Large Files
**URL:** https://github.com/OlaProeis/Ferrite/issues/105
**Status:** 🔍 ACKNOWLEDGED — Rendered View Performance — Scheduled for Future Release

### Suggested Reply:

```markdown
Hi @khimaros! Thank you for the detailed bug report — the specific file size (~6k lines, 50k words) and hardware context (Framework 13) are very helpful.

## Root Cause Analysis

You're experiencing a **fundamental architecture limitation** in the rendered/preview mode, distinct from the raw editor. Here's what's happening:

### The Rendered View Pipeline (Current)

When "rendered view" is enabled, every frame Ferrite must:

1. **Parse the entire markdown** via Comrak (O(N) where N = total document size)
2. **Build egui UI widgets** for every element (paragraphs, lists, headings)
3. **Layout all content** — egui calculates sizes for the entire document, not just visible portion
4. **Render** — even off-screen content contributes to layout calculations

With a 50k word journal file, this creates significant per-frame overhead that scales with document size. On a 60Hz display, that's 60 full document parses/layouts per second.

### What We Fixed in v0.2.7 vs. What Remains

**Fixed in v0.2.7 (Raw Editor):**
The custom FerriteEditor (raw mode) uses **virtual scrolling** — it only parses, highlights, and renders the ~50 visible lines. This was heavily optimized in v0.2.7 and handles large files smoothly now.

**Remaining (Rendered View):**
The WYSIWYG/rendered view (`markdown/editor.rs`) currently builds the entire document structure each frame. It lacks the viewport-based culling that makes the raw editor fast.

## Why This Is Harder Than It Sounds

Unlike the raw editor (uniform line-based content), rendered markdown has:
- **Nested structures** (lists within blockquotes within callouts)
- **Variable-height elements** (tables, mermaid diagrams, images)
- **Bidirectional dependencies** (footnotes, backlinks, wikilinks)
- **egui integration** — we use egui's built-in widgets which expect complete layout information

Simple viewport culling breaks down when:
- A single paragraph spans multiple screens (with word wrap)
- Tables have hundreds of rows
- Mermaid diagrams are taller than the viewport
- Cross-references need global document knowledge

## Roadmap & Timeline

### Immediate Workaround (Available Now)
**Use Raw mode or Split mode with Raw focus** for large journal files:
- Raw mode: `Ctrl+Shift+R` (or View → Raw)
- Split mode: `Ctrl+Shift+S` (edit in raw, preview occasionally)

The raw editor handles your file size smoothly — it was specifically optimized for this in v0.2.7.

### Short-Term (v0.3.x — Late 2026)
**Viewport-Based Rendered View Optimization**
- Implement "windowed" markdown rendering similar to FerriteEditor
- Parse only visible content + small margin
- Cache parsed AST chunks (like our CSV byte-offset row index)
- Lazy widget construction for off-screen content

This is tracked under our **Memory-Mapped I/O & Large Document** initiative (see Roadmap v0.4.0).

### Long-Term (v0.4.0+)
**Full Virtual Document Architecture**
- Incremental markdown parsing (only changed regions)
- Persistent AST with change tracking
- Background thread document processing
- Support for GB-scale files (memory-mapped I/O)

## Why We're Prioritizing Other Features First

You might wonder: why not fix this immediately?

1. **The raw editor workaround exists** — power users editing large files can use raw mode
2. **Most markdown files are <1000 lines** — the current approach works well for typical docs
3. **Architecture risk** — viewport-based markdown requires significant changes to our Comrak integration and egui widget pipeline
4. **Resource allocation** — v0.2.8 is committed to LSP integration and text shaping (HarfRust), which benefit all users; v0.3.0 focuses on RTL/BiDi and Mermaid crate extraction

## Data Gathering

To help us prioritize and test future fixes, could you share:

1. **Does Raw mode perform acceptably?** (`Ctrl+Shift+R`)
2. **File characteristics:**
   - Approximate file size in MB?
   - Heavy use of any specific elements? (tables, code blocks, images, mermaid diagrams)
3. **Performance metrics:**
   - Does scrolling stutter continuously, or only when new content enters view?
   - Is typing laggy in rendered mode, or just scrolling?

## Related Work

- **#19** — Memory-mapped I/O for GB-scale files (planned v0.4.0)
- **v0.2.7 Large File Optimizations** — Raw editor fixes (completed)
- **CSV Lazy Parsing** — Similar viewport-based approach used for large CSV files (1M+ rows)

Thank you for reporting this — large document performance is on our radar, and reports like yours help us prioritize the viewport-based rendered view rewrite.

In the meantime, Raw mode (`Ctrl+Shift+R`) should give you a smooth editing experience for your journal files.
```

---

## Reply to Issue #102 - macOS .md File Association Fix

Hi @sfrankiel,

Thank you for this excellent bug report! Your investigation into the root cause was spot-on — the missing `UTImportedTypeDeclarations` block was indeed preventing macOS from completing the file-type handoff for markdown files.

I've implemented the fix in commit `9704f0c` which adds the required `UTImportedTypeDeclarations` entry to `assets/macos/info_plist_ext.xml`:

```xml
<key>UTImportedTypeDeclarations</key>
<array>
    <dict>
        <key>UTTypeIdentifier</key>
        <string>net.daringfireball.markdown</string>
        <key>UTTypeReferenceURL</key>
        <string>http://daringfireball.net/projects/markdown/</string>
        <key>UTTypeDescription</key>
        <string>Markdown Document</string>
        <key>UTTypeConformsTo</key>
        <array>
            <string>public.plain-text</string>
        </array>
        <key>UTTypeTagSpecification</key>
        <dict>
            <key>public.filename-extension</key>
            <array>
                <string>md</string>
                <string>markdown</string>
                <string>mdown</string>
                <string>mkd</string>
                <string>mkdn</string>
            </array>
        </dict>
    </dict>
</array>
```

This properly declares the `net.daringfireball.markdown` UTI with conformance to `public.plain-text` and all the relevant file extensions.

**To verify the fix:**
1. Rebuild the `.app` bundle: `cargo bundle --release`
2. Replace the existing Ferrite.app in your Applications folder
3. Right-click a `.md` file → "Open With" → Ferrite (or set Ferrite as the default)

This fix is now scheduled for **v0.2.8** (along with the other items on our roadmap like executable code blocks, text shaping improvements, and LSP integration). The CHANGELOG and ROADMAP have been updated accordingly.

Thanks again for the detailed report — it made the fix straightforward! Let me know if you run into any issues with the verification.

---

## Reply to Issue #103 - Windows IME Candidate Box Not Displaying
**URL:** https://github.com/OlaProeis/Ferrite/issues/103
**Status:** 🔍 INVESTIGATED — Scheduled for v0.2.8

### Suggested Reply:

```markdown
Hi @Corditegere! Thank you for this excellent and detailed bug report — the observation about the candidate box appearing during the Win+Shift+S screenshot overlay was particularly insightful and helped us narrow down the root cause.

## Investigation Findings

Your suspicion was correct — this is a **rendering layer/z-order issue**, not an IME input problem. We've identified two contributing factors:

### 1. Missing Coordinate Transform (Confirmed Bug)

Our custom editor (FerriteEditor) sets the IME cursor area using **local widget coordinates**, but egui's built-in `TextEdit` applies a `layer_transform_to_global()` transform to convert to screen coordinates first. Without this transform, the OS receives incorrect coordinates for positioning the candidate window.

**Our code:**
```rust
o.ime = Some(IMEOutput { rect, cursor_rect }); // local coords
```

**What egui's TextEdit does:**
```rust
let to_global = ui.ctx().layer_transform_to_global(ui.layer_id());
o.ime = Some(IMEOutput {
    rect: to_global * rect,           // screen coords
    cursor_rect: to_global * primary_cursor_rect,
});
```

### 2. Custom Window Decorations Z-Order (Suspected)

Ferrite uses `with_decorations(false)` for its custom title bar, which changes the Win32 window styles. This may cause the IME candidate popup to be placed behind the application window in the z-order — explaining why the candidate box becomes visible when the screenshot overlay temporarily changes the compositing order.

## Relationship to #15

This is related to our existing tracked issue [#15](https://github.com/OlaProeis/Ferrite/issues/15) (IME candidate box positioning). Your report provides a more severe manifestation (completely invisible vs. offset) and the screenshot overlay observation was the key clue that confirmed the z-order hypothesis.

## Fix Plan

1. **Apply `to_global` transform** to `rect` and `cursor_rect` when setting `IMEOutput` in our custom editor
2. **Investigate Win32 z-order workarounds** for the candidate popup with custom-decorated windows
3. **Test with multiple IMEs** (Microsoft Pinyin, Sogou, WeChat Input) on Windows 10/11

## Timeline

This fix is scheduled for **v0.2.8**. Since the coordinate transform fix is straightforward, we expect that alone may resolve the issue for most configurations. The z-order investigation may require additional work if the transform fix isn't sufficient.

Thank you again for the detailed reproduction steps and the screenshot overlay observation — that was the breakthrough insight. We'll update this issue when the fix is ready for testing.
```

### Reply to Comment (candidate box visible but mispositioned):

```markdown
Thanks for the follow-up and screenshot @Corditegere — this is very helpful!

The candidate box appearing at the bottom of the screen instead of near your cursor confirms exactly what we suspected: we're passing **local widget coordinates** to the OS instead of **screen coordinates**. Windows is faithfully positioning the candidate box where we tell it to — it's just that the coordinates we're giving it are wrong.

The fix is straightforward — a one-line coordinate transform in our editor code. We'll have this resolved in v0.2.8.
```

---

## Reply to Issue #106 - No Keyboard Input on Ubuntu 24.04 LTS (Wayland)
**URL:** https://github.com/OlaProeis/Ferrite/issues/106
**Status:** 🔍 ACKNOWLEDGED — Wayland Backend Bug in winit 0.29 — Tracked as Task 38

### Suggested Reply:

```markdown
Hi @sfeole! Thank you for the detailed report and for finding the workaround — that's very helpful.

## Root Cause

This is a **known issue with the Wayland backend in winit 0.29.x**, which is the windowing library Ferrite uses (via egui/eframe 0.28). The keyboard input pipeline in winit 0.29's Wayland backend has compatibility issues with certain compositor/libwayland combinations, particularly on GNOME/Mutter (which Ubuntu 24.04 uses by default).

Your workaround of unsetting `WAYLAND_DISPLAY` forces the app to use XWayland instead, which has a more mature input path — confirming the issue is specifically in the Wayland backend.

## Workaround (Confirmed)

As you discovered:

```bash
WAYLAND_DISPLAY= ferrite
```

This forces the XWayland fallback. You can make this permanent by creating a small wrapper script or adding it to your `.desktop` file:

```ini
# ~/.local/share/applications/ferrite-x11.desktop
[Desktop Entry]
Name=Ferrite (X11)
Exec=env WAYLAND_DISPLAY= ferrite %F
Type=Application
```

## Fix Plan

The proper fix is upgrading to **winit 0.31+**, which has a substantially rewritten Wayland backend with improved keyboard and IME handling. This comes with upgrading to **egui 0.31+** (we're currently on 0.28).

As you noted, the dependency tree has some pinned versions, so this is a non-trivial upgrade that needs careful testing across Windows, macOS, and Linux (both X11 and Wayland). We're tracking this as an internal task for an upcoming release.

## In the Meantime

- We'll add documentation about the `WAYLAND_DISPLAY=` workaround to our Linux notes
- If you're building from source and feeling adventurous, bumping `eframe` to `0.31` in `Cargo.toml` might work — though there may be API changes that need fixing

Thank you for the clear reproduction steps and environment details — reports like this are invaluable for tracking platform-specific issues. We'll update this issue when the winit/egui upgrade lands.
```

---

## Reply to Issue #108 - Support Image and PDF Preview
**URL:** https://github.com/OlaProeis/Ferrite/issues/108
**Status:** ✅ IMAGE ACCEPTED (Task 39) | ✅ PDF ACCEPTED (Task 40) — both planned for v0.2.8

### Suggested Reply (initial):

```markdown
Hi @chocolatedesue! Thanks for the feature request.

## Image Preview — Accepted

We already have the image loading infrastructure (PNG, JPEG, GIF, WebP) for rendering images inline in markdown documents. Right now, if you try to open an image file directly, you get a "binary file" error — which isn't great. We're going to turn that into a proper **image viewer tab** instead.

What's planned:
- Open image files (PNG, JPEG, GIF, WebP) in a dedicated viewer tab
- Zoom (Ctrl+scroll wheel) and fit-to-window
- Basic metadata display (dimensions, format, file size)

This is tracked internally and should arrive in an upcoming release.

## Image Support in Markdown (Already Working)

If you're writing markdown, inline image rendering is already available:

```markdown
![description](path/to/image.png)
```

Images display in **Rendered** and **Split** view modes with automatic path resolution relative to the document and workspace root.

## PDF Preview — Long-Term

PDF rendering requires native C/C++ library bindings (PDFium or MuPDF) since no pure-Rust PDF renderer exists with acceptable quality. This adds significant binary size (~20MB per platform) and cross-compilation complexity. It's on our long-term roadmap but won't arrive soon — for now, your OS's built-in PDF viewer will do a better job than anything we could ship in the near term.

Thanks for the suggestion!
```

### Follow-up Reply (re: hayro suggestion):

```markdown
Great find! We've been looking at [hayro](https://github.com/LaurenzV/hayro) and it actually changes the picture significantly for PDF support.

**Why hayro works for us:**
- **Pure Rust** — no native C/C++ dependencies (unlike PDFium/MuPDF), so no cross-compilation headaches
- **MIT/Apache-2.0** — compatible license
- **Uses the `image` crate** — same version we already depend on (0.25), so minimal dependency overhead
- **CPU-only rendering** (via vello_cpu) — no GPU requirements, runs everywhere
- **`#![forbid(unsafe_code)]`** — strong safety guarantees

It renders PDFs to bitmaps, which fits perfectly into our existing texture pipeline (same approach we use for markdown image rendering and the upcoming image viewer tab).

**What we're planning:**
- PDF viewer tab: open `.pdf` files in a read-only viewer
- Page navigation (prev/next, keyboard shortcuts)
- Zoom (Ctrl+scroll, re-render at different DPI)
- Builds on top of the image viewer infrastructure (Task 39)

hayro is still early (v0.5, performance not yet optimized), so complex/large PDFs may be slow — but for the "quick preview a PDF without leaving the editor" use case, it should work well. We'll ship it as a best-effort viewer rather than a full PDF reader.

This is now tracked for **v0.2.8** alongside image viewer support. Thanks for the pointer — this is exactly the kind of community tip that moves features forward!
```

---

## Reply to Issue #109 - No Space Between Paragraphs in Live Preview
**URL:** https://github.com/OlaProeis/Ferrite/issues/109
**Status:** 🐛 CONFIRMED — Tracked as Task 41

### Suggested Reply:

```markdown
Hi @Ragos81! Thanks for the clear bug report and screenshot — confirmed, this is a rendering bug in the live preview.

## Root Cause

The rendered view sets its inter-block spacing (`item_spacing.y`) to just **1 pixel** in `markdown/editor.rs`. This was done intentionally to keep the viewport culling system's height calculations tight and predictable, but it means consecutive paragraphs end up with virtually no visible gap between them — exactly what you're seeing.

In standard markdown rendering, an empty line between paragraphs should produce visible vertical separation (typically about half a line height).

## Fix

We'll add explicit paragraph bottom margins in the rendering functions, independent of the tight inter-block spacing used by the layout system. This gives paragraphs proper visual separation (~0.5em) while keeping the viewport culling math accurate.

We'll also audit spacing for other block types (blockquotes, lists, code blocks) to ensure consistent visual separation across the board.

## Timeline

This is a straightforward fix — tracked internally for **v0.2.8**. We'll update this issue when it ships.

Thanks for reporting!
```

---
