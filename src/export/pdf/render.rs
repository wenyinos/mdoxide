//! Comrak AST → krilla draw‑call renderer.
//!
//! ## Design
//!
//! Two passes via a single walk:
//!
//! 1. **Layout / buffering.** We walk the comrak AST, lay blocks out
//!    vertically with approximate per‑character widths, paginate at page
//!    boundaries, and emit a `Vec<DrawOp>` per page. This keeps the renderer
//!    free of krilla's `Page<'doc>` / `Surface<'page>` borrow lifetimes.
//! 2. **Replay.** When a page is "closed" (because of a page break or the end
//!    of the document) we open a real krilla `Page`, replay the buffered
//!    `DrawOp`s onto its `Surface`, attach link annotations, and move on.
//!
//! plan. Known v1 limitations are documented inline below.

#![allow(clippy::too_many_arguments)]

use comrak::nodes::{AstNode, NodeValue};
use comrak::{parse_document, Arena, Options};
use krilla::action::{Action, LinkAction};
use krilla::annotation::{Annotation, LinkAnnotation, Target};
use krilla::color::rgb;
use krilla::geom::{PathBuilder, Point, Rect, Size};
use krilla::num::NormalizedF32;
use krilla::page::PageSettings;
use krilla::paint::{Fill, FillRule, Stroke};
use krilla::surface::Surface;
use krilla::text::{Font, TextDirection};
use krilla::Document;
use std::path::Path;
use std::sync::Arc;

use super::fonts::{load_bundled_fonts, FontLoadError, FontStyle, PdfFonts};
use super::options::PdfExportOptions;
use super::theme::PdfTheme;

// ─────────────────────────────────────────────────────────────────────────────
// Errors
// ─────────────────────────────────────────────────────────────────────────────

/// Errors that can occur during PDF export.
#[derive(Debug)]
pub enum PdfExportError {
    /// Failed to load a bundled font.
    Font(FontLoadError),
    /// krilla failed to finalize the document.
    Encode(String),
    /// IO failure while reading source or writing output.
    Io(std::io::Error),
}

impl std::fmt::Display for PdfExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PdfExportError::Font(e) => write!(f, "{}", e),
            PdfExportError::Encode(msg) => write!(f, "PDF encoding failed: {}", msg),
            PdfExportError::Io(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for PdfExportError {}

impl From<FontLoadError> for PdfExportError {
    fn from(e: FontLoadError) -> Self {
        PdfExportError::Font(e)
    }
}

impl From<std::io::Error> for PdfExportError {
    fn from(e: std::io::Error) -> Self {
        PdfExportError::Io(e)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Public API
// ─────────────────────────────────────────────────────────────────────────────

/// Render the given Markdown to a PDF byte buffer.
///
/// `_base_dir` will be used to resolve relative image paths once raster
/// embedding lands; ignored in v1.
pub fn render_markdown_to_pdf(
    markdown: &str,
    options: &PdfExportOptions,
    theme: &PdfTheme,
    _base_dir: Option<&Path>,
) -> Result<Vec<u8>, PdfExportError> {
    let fonts = load_bundled_fonts()?;
    let mut renderer = Renderer::new(options, theme, fonts);
    renderer.render(markdown);
    renderer.finish()
}

// ─────────────────────────────────────────────────────────────────────────────
// Buffered draw operations
// ─────────────────────────────────────────────────────────────────────────────

/// One buffered drawing primitive. Replayed onto a krilla `Surface` when a
/// page is flushed.
#[derive(Clone)]
enum DrawOp {
    Text {
        x: f32,
        y: f32,
        size: f32,
        style: FontStyle,
        mono: bool,
        text: String,
        color: rgb::Color,
        strike: bool,
    },
    FilledRect {
        rect: Rect,
        color: rgb::Color,
    },
    RectOutline {
        rect: Rect,
        width: f32,
        color: rgb::Color,
    },
    HorizontalRule {
        x1: f32,
        x2: f32,
        y: f32,
        width: f32,
        color: rgb::Color,
    },
    Image {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        png: Arc<Vec<u8>>,
    },
}

/// Everything we know about a single page before flushing it to krilla.
#[derive(Default, Clone)]
struct PageBuf {
    ops: Vec<DrawOp>,
    links: Vec<(Rect, String)>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Inline runs
// ─────────────────────────────────────────────────────────────────────────────

/// One styled chunk of inline text. Lines are built up as `Vec<InlineRun>` and
/// then wrapped into visual lines.
#[derive(Clone)]
struct InlineRun {
    text: String,
    style: FontStyle,
    /// True if this run is monospace (inline `code`).
    mono: bool,
    /// Color override; falls back to the body color.
    color: Option<rgb::Color>,
    /// True if a strike‑through line should be drawn over the run.
    strike: bool,
    /// External URL for this run, if it sits inside a link.
    link: Option<String>,
    /// Rendered formula image; when set, `text` is empty and this is drawn instead.
    formula: Option<FormulaImage>,
}

/// A formula rasterized by RaTeX, with its display size in PDF points.
#[derive(Clone)]
struct FormulaImage {
    png: Arc<Vec<u8>>,
    width: f32,
    height: f32,
}

// ─────────────────────────────────────────────────────────────────────────────
// Renderer
// ─────────────────────────────────────────────────────────────────────────────

struct Renderer<'a> {
    options: &'a PdfExportOptions,
    theme: &'a PdfTheme,
    fonts: PdfFonts,

    // Page geometry (PDF points)
    page_w: f32,
    page_h: f32,
    margin_l: f32,
    margin_r: f32,
    margin_t: f32,
    margin_b: f32,

    // Layout state
    cursor_y: f32,
    /// Buffered pages. The last entry is the in‑progress page.
    pages: Vec<PageBuf>,
    /// True if the current page has any content yet (used so an empty page
    /// at the start of the document doesn't trigger a spurious page break).
    page_has_content: bool,
}

// Sizes/spacings in PDF points. Tuned for a 595 × 842 (A4) layout at the
// default 11 pt body size.
const BODY_FONT_SIZE: f32 = 11.0;
const CODE_FONT_SIZE: f32 = 10.0;
const LINE_HEIGHT_FACTOR: f32 = 1.4;
const PARAGRAPH_GAP: f32 = 6.0;
const HEADING_GAP_BEFORE: f32 = 12.0;
const HEADING_GAP_AFTER: f32 = 6.0;
const LIST_INDENT: f32 = 18.0;
const BLOCKQUOTE_INDENT: f32 = 16.0;
const CODE_BLOCK_PADDING: f32 = 8.0;
const TABLE_CELL_PADDING: f32 = 4.0;

/// Heading sizes (1‑indexed: HEADING_SIZES[0] is H1).
const HEADING_SIZES: [f32; 6] = [22.0, 18.0, 15.0, 13.0, 12.0, 11.0];

impl<'a> Renderer<'a> {
    fn new(options: &'a PdfExportOptions, theme: &'a PdfTheme, fonts: PdfFonts) -> Self {
        let margins = options.effective_margins();
        let page_w = options.page_size.width();
        let page_h = options.page_size.height();
        Self {
            options,
            theme,
            fonts,
            page_w,
            page_h,
            margin_l: margins.left,
            margin_r: margins.right,
            margin_t: margins.top,
            margin_b: margins.bottom,
            cursor_y: margins.top,
            pages: vec![PageBuf::default()],
            page_has_content: false,
        }
    }

    fn content_width(&self) -> f32 {
        self.page_w - self.margin_l - self.margin_r
    }

    fn content_bottom(&self) -> f32 {
        self.page_h - self.margin_b
    }

    fn current_page(&mut self) -> &mut PageBuf {
        self.pages.last_mut().expect("at least one page exists")
    }

    /// Open a fresh page and reset the layout cursor.
    fn page_break(&mut self) {
        self.pages.push(PageBuf::default());
        self.cursor_y = self.margin_t;
        self.page_has_content = false;
    }

    /// Reserve `height` PDF points for the next block, paginating if needed.
    /// Returns the y at which to begin drawing.
    fn reserve(&mut self, height: f32) -> f32 {
        let available = self.content_bottom() - self.cursor_y;
        if height > available && self.page_has_content {
            self.page_break();
        }
        let y = self.cursor_y;
        self.cursor_y += height;
        self.page_has_content = true;
        y
    }

    fn push_op(&mut self, op: DrawOp) {
        self.current_page().ops.push(op);
        self.page_has_content = true;
    }

    fn push_link(&mut self, rect: Rect, url: String) {
        self.current_page().links.push((rect, url));
    }

    // ─── Top-level rendering ────────────────────────────────────────────────

    fn render(&mut self, markdown: &str) {
        let mut comrak_opts = Options::default();
        comrak_opts.extension.strikethrough = true;
        comrak_opts.extension.table = true;
        comrak_opts.extension.autolink = true;
        comrak_opts.extension.tasklist = true;
        comrak_opts.extension.footnotes = true;
        comrak_opts.extension.math_dollars = true;
        comrak_opts.extension.front_matter_delimiter = Some("---".to_string());
        comrak_opts.render.unsafe_ = true;

        let arena = Arena::new();
        let root = parse_document(&arena, markdown, &comrak_opts);

        for child in root.children() {
            self.render_block(child);
        }
    }

    fn finish(self) -> Result<Vec<u8>, PdfExportError> {
        let Renderer {
            options,
            theme,
            fonts,
            page_w,
            page_h,
            pages,
            ..
        } = self;
        let total_pages = pages.len();

        let mut document = Document::new();

        for (i, page_buf) in pages.into_iter().enumerate() {
            let page_idx_one = i + 1;
            let settings =
                PageSettings::new(Size::from_wh(page_w, page_h).expect("valid page size"));
            let mut page = document.start_page_with(settings);

            // Paint background if theme colors are enabled and the bg isn't ~white.
            if options.use_theme_colors {
                if let Some(bg) = theme.background {
                    if !is_near_white(bg) {
                        let mut surface = page.surface();
                        let rect =
                            Rect::from_xywh(0.0, 0.0, page_w, page_h).expect("valid bg rect");
                        replay_op(
                            &mut surface,
                            &fonts,
                            &DrawOp::FilledRect { rect, color: bg },
                        );
                        surface.finish();
                    }
                }
            }

            // Page number footer.
            if options.include_page_numbers {
                let label = format!("Page {} of {}", page_idx_one, total_pages);
                let label_size = BODY_FONT_SIZE * 0.8;
                let w = approx_text_width(&label, label_size, false);
                let x = (page_w - w) * 0.5;
                let y = page_h - options.effective_margins().bottom * 0.5;
                let mut surface = page.surface();
                replay_op(
                    &mut surface,
                    &fonts,
                    &DrawOp::Text {
                        x,
                        y,
                        size: label_size,
                        style: FontStyle::Regular,
                        mono: false,
                        text: label,
                        color: theme.muted,
                        strike: false,
                    },
                );
                surface.finish();
            }

            // Main draw ops.
            {
                let mut surface = page.surface();
                for op in &page_buf.ops {
                    replay_op(&mut surface, &fonts, op);
                }
                surface.finish();
            }

            // Link annotations.
            for (rect, url) in page_buf.links {
                let link = LinkAnnotation::new(
                    rect,
                    Target::Action(Action::Link(LinkAction::new(url.clone()))),
                );
                page.add_annotation(Annotation::new_link(link, Some(url)));
            }

            page.finish();
        }

        let pdf = document
            .finish()
            .map_err(|e| PdfExportError::Encode(format!("{:?}", e)))?;
        Ok(pdf)
    }

    // ─── Block dispatch ─────────────────────────────────────────────────────

    fn render_block<'b>(&mut self, node: &'b AstNode<'b>) {
        let nv = node.data.borrow().value.clone();
        match nv {
            NodeValue::Document => {
                for child in node.children() {
                    self.render_block(child);
                }
            }
            NodeValue::FrontMatter(_) => {
                // Skip — we intentionally do not export YAML/TOML frontmatter.
            }
            NodeValue::Heading(h) => {
                self.render_heading(node, h.level);
            }
            NodeValue::Paragraph => {
                // A paragraph that contains only a display-math node is the
                // block form (`$$...$$`): lay it out centred on its own line.
                if let Some(latex) = single_display_math(node) {
                    self.render_math_block(&latex, true);
                } else {
                    let runs = self.collect_inline_runs(
                        node,
                        FontStyle::Regular,
                        false,
                        None,
                        false,
                        None,
                    );
                    self.render_text_block(&runs, BODY_FONT_SIZE);
                    self.cursor_y += PARAGRAPH_GAP;
                }
            }
            NodeValue::ThematicBreak => {
                self.render_hr();
            }
            NodeValue::BlockQuote => {
                self.render_blockquote(node);
            }
            NodeValue::List(_) => {
                self.render_list(node, 0);
            }
            NodeValue::CodeBlock(cb) => {
                self.render_code_block(&cb.literal, &cb.info);
            }
            NodeValue::Math(math) => {
                self.render_math_block(&math.literal, math.display_math);
            }
            NodeValue::Table(_) => {
                self.render_table(node);
            }
            NodeValue::HtmlBlock(_) | NodeValue::HtmlInline(_) => {
                let runs = vec![InlineRun {
                    text: "(raw HTML omitted from PDF)".into(),
                    style: FontStyle::Italic,
                    mono: false,
                    color: Some(self.theme.muted),
                    strike: false,
                    link: None,
                    formula: None,
                }];
                self.render_text_block(&runs, BODY_FONT_SIZE * 0.9);
                self.cursor_y += PARAGRAPH_GAP;
            }
            _ => {
                for child in node.children() {
                    self.render_block(child);
                }
            }
        }
    }

    // ─── Block: Heading ─────────────────────────────────────────────────────

    fn render_heading<'b>(&mut self, node: &'b AstNode<'b>, level: u8) {
        let level = level.clamp(1, 6) as usize;
        let size = HEADING_SIZES[level - 1];

        // H1 page break: skip on the very first heading of the document.
        if level == 1 && self.options.page_break_before_h1 && self.page_has_content {
            self.page_break();
        }

        // Top breathing room (skipped on a fresh page).
        if self.page_has_content {
            self.cursor_y += HEADING_GAP_BEFORE;
        }

        let runs = self.collect_inline_runs(
            node,
            FontStyle::Bold,
            false,
            Some(self.theme.heading),
            false,
            None,
        );
        self.render_text_block(&runs, size);
        self.cursor_y += HEADING_GAP_AFTER;

        // H1 / H2 underline mimics the HTML export look.
        if level <= 2 {
            let y = self.cursor_y - 2.0;
            let left = self.margin_l;
            let right = self.page_w - self.margin_r;
            self.push_op(DrawOp::HorizontalRule {
                x1: left,
                x2: right,
                y,
                width: 0.5,
                color: self.theme.muted,
            });
            self.cursor_y += 2.0;
        }
    }

    // ─── Block: Horizontal rule ─────────────────────────────────────────────

    fn render_hr(&mut self) {
        self.cursor_y += HEADING_GAP_BEFORE;
        let _ = self.reserve(2.0); // small slot for the rule
        let y = self.cursor_y - 1.0;
        self.push_op(DrawOp::HorizontalRule {
            x1: self.margin_l,
            x2: self.page_w - self.margin_r,
            y,
            width: 1.0,
            color: self.theme.muted,
        });
        self.cursor_y += HEADING_GAP_AFTER;
    }

    // ─── Block: Blockquote ──────────────────────────────────────────────────

    fn render_blockquote<'b>(&mut self, node: &'b AstNode<'b>) {
        let saved_left = self.margin_l;
        self.margin_l += BLOCKQUOTE_INDENT;

        let start_y = self.cursor_y;
        let start_page = self.pages.len() - 1;

        for child in node.children() {
            self.render_block(child);
        }
        let end_y = self.cursor_y;
        let end_page = self.pages.len() - 1;

        self.margin_l = saved_left;

        // Draw the left border bar across the part of the quote on each page.
        let bar_x = saved_left + 2.0;
        let bar_color = self.theme.muted;

        if start_page == end_page {
            let height = (end_y - start_y).max(2.0);
            if let Some(rect) = Rect::from_xywh(bar_x, start_y, 3.0, height) {
                self.pages[start_page].ops.push(DrawOp::FilledRect {
                    rect,
                    color: bar_color,
                });
            }
        } else {
            // First page: from start_y to the bottom margin.
            let first_h = self.content_bottom() - start_y;
            if first_h > 0.0 {
                if let Some(rect) = Rect::from_xywh(bar_x, start_y, 3.0, first_h) {
                    self.pages[start_page].ops.push(DrawOp::FilledRect {
                        rect,
                        color: bar_color,
                    });
                }
            }
            // Intermediate pages: full height.
            for p in (start_page + 1)..end_page {
                let h = self.content_bottom() - self.margin_t;
                if let Some(rect) = Rect::from_xywh(bar_x, self.margin_t, 3.0, h) {
                    self.pages[p].ops.push(DrawOp::FilledRect {
                        rect,
                        color: bar_color,
                    });
                }
            }
            // Last page: from top margin to end_y.
            let last_h = end_y - self.margin_t;
            if last_h > 0.0 {
                if let Some(rect) = Rect::from_xywh(bar_x, self.margin_t, 3.0, last_h) {
                    self.pages[end_page].ops.push(DrawOp::FilledRect {
                        rect,
                        color: bar_color,
                    });
                }
            }
        }
    }

    // ─── Block: Lists ───────────────────────────────────────────────────────

    fn render_list<'b>(&mut self, node: &'b AstNode<'b>, depth: usize) {
        let nv = node.data.borrow().value.clone();
        let (ordered, mut counter) = if let NodeValue::List(list) = nv {
            (
                matches!(list.list_type, comrak::nodes::ListType::Ordered),
                list.start as i32,
            )
        } else {
            (false, 1)
        };

        for item in node.children() {
            self.render_list_item(item, ordered, counter, depth);
            counter += 1;
        }
    }

    fn render_list_item<'b>(
        &mut self,
        item: &'b AstNode<'b>,
        ordered: bool,
        index: i32,
        depth: usize,
    ) {
        let saved_left = self.margin_l;
        self.margin_l += LIST_INDENT;

        // Marker, drawn at the current cursor on the line of the first paragraph.
        let marker_text = if let NodeValue::TaskItem(check) = item.data.borrow().value.clone() {
            // U+2611/U+2610 are missing from Source Han Sans; use glyphs that
            // CJK fonts reliably carry.
            if check.is_some() {
                "√".to_string()
            } else {
                "□".to_string()
            }
        } else if ordered {
            format!("{}.", index)
        } else if depth % 2 == 0 {
            "•".to_string()
        } else {
            "◦".to_string()
        };

        let marker_x = saved_left + 4.0;
        let marker_baseline = self.cursor_y + BODY_FONT_SIZE * 0.85;
        self.push_op(DrawOp::Text {
            x: marker_x,
            y: marker_baseline,
            size: BODY_FONT_SIZE,
            style: FontStyle::Regular,
            mono: false,
            text: marker_text,
            color: self.theme.body,
            strike: false,
        });

        for child in item.children() {
            match child.data.borrow().value.clone() {
                NodeValue::List(_) => {
                    self.render_list(child, depth + 1);
                }
                _ => {
                    self.render_block(child);
                }
            }
        }

        self.margin_l = saved_left;
    }

    // ─── Block: Code block ──────────────────────────────────────────────────

    fn render_code_block(&mut self, source: &str, _info: &str) {
        let lines: Vec<&str> = source.lines().collect();
        let line_count = lines.len().max(1) as f32;
        let line_height = CODE_FONT_SIZE * LINE_HEIGHT_FACTOR;
        let block_height = line_count * line_height + CODE_BLOCK_PADDING * 2.0;

        let y_top = self.reserve(block_height);

        let bg = self.theme.code_block_bg;
        let border = self.theme.muted;
        let text_color = self.theme.body;
        let left = self.margin_l;
        let right = self.page_w - self.margin_r;

        if let Some(rect) = Rect::from_xywh(left, y_top, right - left, block_height) {
            self.push_op(DrawOp::FilledRect { rect, color: bg });
            self.push_op(DrawOp::RectOutline {
                rect,
                width: 0.5,
                color: border,
            });
        }

        let mut y = y_top + CODE_BLOCK_PADDING + CODE_FONT_SIZE * 0.85;
        for line in lines {
            self.push_op(DrawOp::Text {
                x: left + CODE_BLOCK_PADDING,
                y,
                size: CODE_FONT_SIZE,
                style: FontStyle::Regular,
                mono: true,
                text: line.to_string(),
                color: text_color,
                strike: false,
            });
            y += line_height;
        }

        self.cursor_y += PARAGRAPH_GAP;
    }

    // ─── Block: Table ───────────────────────────────────────────────────────

    fn render_table<'b>(&mut self, node: &'b AstNode<'b>) {
        let mut rows: Vec<(bool, Vec<String>)> = Vec::new();
        for row_node in node.children() {
            if let NodeValue::TableRow(is_header) = row_node.data.borrow().value.clone() {
                let mut cells = Vec::new();
                for cell_node in row_node.children() {
                    let runs = self.collect_inline_runs(
                        cell_node,
                        FontStyle::Regular,
                        false,
                        None,
                        false,
                        None,
                    );
                    let plain = runs
                        .iter()
                        .map(|r| r.text.as_str())
                        .collect::<Vec<_>>()
                        .join("");
                    cells.push(plain);
                }
                rows.push((is_header, cells));
            }
        }

        if rows.is_empty() {
            return;
        }
        let num_cols = rows.iter().map(|(_, c)| c.len()).max().unwrap_or(1);
        let content_w = self.content_width();
        let col_w = content_w / num_cols as f32;
        let line_height = BODY_FONT_SIZE * LINE_HEIGHT_FACTOR;

        let muted = self.theme.muted;
        let body_color = self.theme.body;
        let header_bg = self.theme.code_block_bg;
        let left = self.margin_l;

        for (is_header, cells) in rows {
            let mut wrapped: Vec<Vec<String>> = Vec::with_capacity(num_cols);
            for col in 0..num_cols {
                let cell = cells.get(col).cloned().unwrap_or_default();
                let lines = wrap_plain_text(
                    &cell,
                    col_w - TABLE_CELL_PADDING * 2.0,
                    BODY_FONT_SIZE,
                    false,
                );
                wrapped.push(lines);
            }
            let row_lines = wrapped.iter().map(|l| l.len().max(1)).max().unwrap_or(1);
            let row_h = row_lines as f32 * line_height + TABLE_CELL_PADDING * 2.0;

            let y_top = self.reserve(row_h);

            let style_for_cell = if is_header {
                FontStyle::Bold
            } else {
                FontStyle::Regular
            };

            if is_header {
                if let Some(rect) = Rect::from_xywh(left, y_top, col_w * num_cols as f32, row_h) {
                    self.push_op(DrawOp::FilledRect {
                        rect,
                        color: header_bg,
                    });
                }
            }

            for (col, lines) in wrapped.into_iter().enumerate() {
                let cell_left = left + col as f32 * col_w;
                if let Some(rect) = Rect::from_xywh(cell_left, y_top, col_w, row_h) {
                    self.push_op(DrawOp::RectOutline {
                        rect,
                        width: 0.5,
                        color: muted,
                    });
                }
                let mut y = y_top + TABLE_CELL_PADDING + BODY_FONT_SIZE * 0.85;
                for line in lines {
                    self.push_op(DrawOp::Text {
                        x: cell_left + TABLE_CELL_PADDING,
                        y,
                        size: BODY_FONT_SIZE,
                        style: style_for_cell,
                        mono: false,
                        text: line,
                        color: body_color,
                        strike: false,
                    });
                    y += line_height;
                }
            }
        }

        self.cursor_y += PARAGRAPH_GAP;
    }

    // ─── Inline collection ──────────────────────────────────────────────────

    fn collect_inline_runs<'b>(
        &self,
        node: &'b AstNode<'b>,
        base_style: FontStyle,
        base_mono: bool,
        base_color: Option<rgb::Color>,
        base_strike: bool,
        base_link: Option<String>,
    ) -> Vec<InlineRun> {
        let mut out: Vec<InlineRun> = Vec::new();
        for child in node.children() {
            self.walk_inline(
                child,
                base_style,
                base_mono,
                base_color,
                base_strike,
                base_link.clone(),
                &mut out,
            );
        }
        out
    }

    fn walk_inline<'b>(
        &self,
        node: &'b AstNode<'b>,
        style: FontStyle,
        mono: bool,
        color: Option<rgb::Color>,
        strike: bool,
        link: Option<String>,
        out: &mut Vec<InlineRun>,
    ) {
        let nv = node.data.borrow().value.clone();
        match nv {
            NodeValue::Text(text) => {
                out.push(InlineRun {
                    text,
                    style,
                    mono,
                    color: link.as_ref().map(|_| self.theme.link).or(color),
                    strike,
                    link: link.clone(),
                    formula: None,
                });
            }
            NodeValue::Code(code) => {
                out.push(InlineRun {
                    text: code.literal,
                    style: FontStyle::Regular,
                    mono: true,
                    color: color.or(Some(self.theme.code_inline)),
                    strike,
                    link: link.clone(),
                    formula: None,
                });
            }
            NodeValue::SoftBreak | NodeValue::LineBreak => {
                out.push(InlineRun {
                    text: " ".to_string(),
                    style,
                    mono,
                    color,
                    strike,
                    link: link.clone(),
                    formula: None,
                });
            }
            NodeValue::Strong => {
                let next_style = match style {
                    FontStyle::Italic | FontStyle::BoldItalic => FontStyle::BoldItalic,
                    _ => FontStyle::Bold,
                };
                for c in node.children() {
                    self.walk_inline(c, next_style, mono, color, strike, link.clone(), out);
                }
            }
            NodeValue::Emph => {
                let next_style = match style {
                    FontStyle::Bold | FontStyle::BoldItalic => FontStyle::BoldItalic,
                    _ => FontStyle::Italic,
                };
                for c in node.children() {
                    self.walk_inline(c, next_style, mono, color, strike, link.clone(), out);
                }
            }
            NodeValue::Strikethrough => {
                for c in node.children() {
                    self.walk_inline(c, style, mono, color, true, link.clone(), out);
                }
            }
            NodeValue::Link(l) => {
                let url = l.url;
                for c in node.children() {
                    self.walk_inline(c, style, mono, color, strike, Some(url.clone()), out);
                }
            }
            NodeValue::Image(img) => {
                out.push(InlineRun {
                    text: format!("[image: {}]", img.url),
                    style: FontStyle::Italic,
                    mono: false,
                    color: color.or(Some(self.theme.muted)),
                    strike,
                    link: link.clone(),
                    formula: None,
                });
            }
            NodeValue::Math(math) => {
                let run_color = color.unwrap_or(self.theme.body);
                match render_formula_image(&math.literal, BODY_FONT_SIZE, run_color) {
                    Some(formula) => out.push(InlineRun {
                        text: String::new(),
                        style,
                        mono: false,
                        color: Some(run_color),
                        strike: false,
                        link: link.clone(),
                        formula: Some(formula),
                    }),
                    None => out.push(InlineRun {
                        text: math.literal,
                        style,
                        mono: false,
                        color: Some(run_color),
                        strike: false,
                        link: link.clone(),
                        formula: None,
                    }),
                }
            }
            NodeValue::HtmlInline(_) => {
                // Skip raw HTML inline.
            }
            _ => {
                for c in node.children() {
                    self.walk_inline(c, style, mono, color, strike, link.clone(), out);
                }
            }
        }
    }

    // ─── Inline layout: wrap and emit DrawOps ───────────────────────────────

    fn render_text_block(&mut self, runs: &[InlineRun], font_size: f32) {
        if runs.is_empty() {
            return;
        }
        let max_width = self.content_width();
        let lines = wrap_runs(runs, max_width, font_size);
        let body_color = self.theme.body;

        for line in lines {
            // Grow the line when it carries formulas so their images start
            // inside the line box instead of bleeding into the line above.
            let formula_height = line
                .iter()
                .filter_map(|run| run.formula.as_ref())
                .map(|formula| formula.height + 2.0)
                .fold(0.0_f32, f32::max);
            let baseline_offset = (font_size * 0.85).max(formula_height);
            let line_height = baseline_offset + font_size * 0.25;
            let y_top = self.reserve(line_height);
            let baseline_y = y_top + baseline_offset;
            let mut x = self.margin_l;
            for run in line {
                let color = run.color.unwrap_or(body_color);
                if let Some(formula) = &run.formula {
                    self.push_op(DrawOp::Image {
                        x,
                        y: baseline_y - formula.height,
                        width: formula.width,
                        height: formula.height,
                        png: formula.png.clone(),
                    });
                    x += formula.width;
                    continue;
                }
                let w = approx_text_width(&run.text, font_size, run.mono);
                self.push_op(DrawOp::Text {
                    x,
                    y: baseline_y,
                    size: font_size,
                    style: run.style,
                    mono: run.mono,
                    text: run.text.clone(),
                    color,
                    strike: run.strike,
                });
                if let Some(url) = &run.link {
                    if let Some(rect) = Rect::from_xywh(
                        x,
                        baseline_y - font_size * 0.85,
                        w,
                        font_size * LINE_HEIGHT_FACTOR,
                    ) {
                        self.push_link(rect, url.clone());
                    }
                    // Underline links so they remain distinguishable when the
                    // surrounding text is also coloured.
                    self.push_op(DrawOp::HorizontalRule {
                        x1: x,
                        x2: x + w,
                        y: baseline_y + 1.5,
                        width: 0.5,
                        color,
                    });
                }
                x += w;
            }
        }
    }

    fn render_math_block(&mut self, latex: &str, display: bool) {
        let size = if display {
            BODY_FONT_SIZE * 1.15
        } else {
            BODY_FONT_SIZE
        };
        let Some(mut formula) = render_formula_image(latex, size, self.theme.body) else {
            let runs = vec![InlineRun {
                text: latex.to_string(),
                style: FontStyle::Regular,
                mono: false,
                color: Some(self.theme.muted),
                strike: false,
                link: None,
                formula: None,
            }];
            self.render_text_block(&runs, BODY_FONT_SIZE);
            self.cursor_y += PARAGRAPH_GAP;
            return;
        };

        let max_width = self.content_width();
        if formula.width > max_width && formula.width > 0.0 {
            let scale = max_width / formula.width;
            formula.width *= scale;
            formula.height *= scale;
        }

        if display {
            self.cursor_y += PARAGRAPH_GAP * 0.5;
        }
        let y_top = self.reserve(formula.height);
        let x = self.margin_l + (max_width - formula.width) * 0.5;
        self.push_op(DrawOp::Image {
            x,
            y: y_top,
            width: formula.width,
            height: formula.height,
            png: formula.png,
        });
        self.cursor_y += PARAGRAPH_GAP;
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Replay: DrawOp → krilla Surface
// ─────────────────────────────────────────────────────────────────────────────

/// Returns the TeX literal when `node` is a paragraph holding only a
/// display-math node (`$$...$$`).
fn single_display_math<'b>(node: &'b AstNode<'b>) -> Option<String> {
    let mut children = node.children();
    let first = children.next()?;
    if children.next().is_some() {
        return None;
    }
    match &first.data.borrow().value {
        NodeValue::Math(math) if math.display_math => Some(math.literal.clone()),
        _ => None,
    }
}

/// Rasterize a LaTeX formula at 2x and convert pixels back to PDF points.
fn render_formula_image(latex: &str, font_size: f32, color: rgb::Color) -> Option<FormulaImage> {
    let png = crate::markdown::math_render::render_latex_png(
        latex,
        font_size,
        [color.red(), color.green(), color.blue()],
        2.0,
    )
    .ok()?;
    let (width, height) = crate::export::png_dimensions(&png)?;
    Some(FormulaImage {
        png: Arc::new(png),
        width: width as f32 / 2.0,
        height: height as f32 / 2.0,
    })
}

fn replay_op(surface: &mut Surface<'_>, fonts: &PdfFonts, op: &DrawOp) {
    match op {
        DrawOp::Text {
            x,
            y,
            size,
            style,
            mono,
            text,
            color,
            strike,
        } => {
            draw_text_run(
                surface, fonts, *style, *mono, *x, *y, *size, text, *color, *strike,
            );
        }
        DrawOp::FilledRect { rect, color } => {
            paint_filled_rect(surface, *rect, *color);
        }
        DrawOp::RectOutline { rect, width, color } => {
            paint_rect_outline(surface, *rect, *width, *color);
        }
        DrawOp::HorizontalRule {
            x1,
            x2,
            y,
            width,
            color,
        } => {
            paint_horizontal_rule(surface, *x1, *x2, *y, *width, *color);
        }
        DrawOp::Image {
            x,
            y,
            width,
            height,
            png,
        } => {
            let data = krilla::Data::from(png.clone());
            if let Ok(image) = krilla::image::Image::from_png(data, true) {
                if let Some(size) = Size::from_wh(*width, *height) {
                    surface.push_transform(&krilla::geom::Transform::from_translate(*x, *y));
                    surface.draw_image(image, size);
                    surface.pop();
                }
            }
        }
    }
}

fn draw_text_run(
    surface: &mut Surface<'_>,
    fonts: &PdfFonts,
    style: FontStyle,
    mono: bool,
    x: f32,
    y: f32,
    size: f32,
    text: &str,
    color: rgb::Color,
    strike: bool,
) {
    if text.is_empty() {
        return;
    }
    // krilla's simple-text path performs no font fallback, so a run containing
    // CJK characters must use the CJK face for the whole run. Source Han Sans
    // also carries Latin glyphs, which keeps mixed runs readable.
    let family = if text.chars().any(is_wide_char) {
        &fonts.cjk
    } else if mono {
        &fonts.mono
    } else {
        &fonts.body
    };
    let font: Font = family.pick(style);

    surface.set_fill(Some(Fill {
        paint: color.into(),
        opacity: NormalizedF32::ONE,
        rule: FillRule::NonZero,
    }));
    surface.draw_text(
        Point::from_xy(x, y),
        font,
        size,
        text,
        false,
        TextDirection::Auto,
    );

    if strike {
        let w = approx_text_width(text, size, mono);
        let mid_y = y - size * 0.3;
        let mut pb = PathBuilder::new();
        pb.move_to(x, mid_y);
        pb.line_to(x + w, mid_y);
        if let Some(path) = pb.finish() {
            surface.set_stroke(Some(Stroke {
                paint: color.into(),
                width: 0.6,
                ..Default::default()
            }));
            surface.draw_path(&path);
        }
    }
}

fn paint_filled_rect(surface: &mut Surface<'_>, rect: Rect, color: rgb::Color) {
    let mut pb = PathBuilder::new();
    pb.push_rect(rect);
    if let Some(path) = pb.finish() {
        surface.set_fill(Some(Fill {
            paint: color.into(),
            opacity: NormalizedF32::ONE,
            rule: FillRule::NonZero,
        }));
        surface.set_stroke(None);
        surface.draw_path(&path);
    }
}

fn paint_rect_outline(surface: &mut Surface<'_>, rect: Rect, width: f32, color: rgb::Color) {
    let mut pb = PathBuilder::new();
    pb.push_rect(rect);
    if let Some(path) = pb.finish() {
        // Paths are filled with the currently active fill, so clear the fill
        // left behind by the previous text run or cell background.
        surface.set_fill(None);
        surface.set_stroke(Some(Stroke {
            paint: color.into(),
            width,
            ..Default::default()
        }));
        surface.draw_path(&path);
    }
}

fn paint_horizontal_rule(
    surface: &mut Surface<'_>,
    x1: f32,
    x2: f32,
    y: f32,
    width: f32,
    color: rgb::Color,
) {
    let mut pb = PathBuilder::new();
    pb.move_to(x1, y);
    pb.line_to(x2, y);
    if let Some(path) = pb.finish() {
        surface.set_fill(None);
        surface.set_stroke(Some(Stroke {
            paint: color.into(),
            width,
            ..Default::default()
        }));
        surface.draw_path(&path);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Word‑wrapping helpers (approximate widths; see roadmap Phase 5)
// ─────────────────────────────────────────────────────────────────────────────

/// Full-width characters: CJK ideographs/kana plus the halfwidth-and-fullwidth
/// forms block, which is where full-width punctuation such as `，` lives.
fn is_wide_char(c: char) -> bool {
    crate::fonts::is_cjk_char(c) || matches!(c, '\u{FF00}'..='\u{FFEF}')
}

fn char_width(c: char, font_size: f32, mono: bool) -> f32 {
    if is_wide_char(c) {
        font_size
    } else {
        font_size * if mono { 0.6 } else { 0.5 }
    }
}

fn approx_text_width(text: &str, font_size: f32, mono: bool) -> f32 {
    // A run containing wide characters is drawn with the proportional CJK
    // face even when it is inline code, so measure it proportionally.
    let mono = mono && !text.chars().any(is_wide_char);
    text.chars().map(|c| char_width(c, font_size, mono)).sum()
}

/// Closing punctuation that must not start a line (CJK line-start prohibition).
fn is_line_start_forbidden(c: char) -> bool {
    matches!(
        c,
        '，' | '。'
            | '、'
            | '；'
            | '：'
            | '！'
            | '？'
            | '）'
            | '》'
            | '」'
            | '』'
            | '】'
            | '〕'
            | '〉'
            | '”'
            | '’'
            | '…'
            | '·'
            | ','
            | '.'
            | ';'
            | ':'
            | '!'
            | '?'
            | ')'
            | ']'
            | '}'
    )
}

/// Split a token that cannot fit on a line into chunks no wider than `max_width`.
///
/// CJK text has no spaces, so without this a whole paragraph becomes one
/// "word" and runs past the right margin.
fn split_long_token(token: &str, max_width: f32, font_size: f32, mono: bool) -> Vec<String> {
    if approx_text_width(token, font_size, mono) <= max_width {
        return vec![token.to_string()];
    }

    let mut chunks: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut width = 0.0;
    for ch in token.chars() {
        let ch_w = char_width(ch, font_size, mono);
        if !current.is_empty() && width + ch_w > max_width {
            if is_line_start_forbidden(ch) {
                // Carry the preceding character down with the punctuation so
                // the new line does not start with a comma/period.
                if let Some(prev) = current.pop() {
                    width -= char_width(prev, font_size, mono);
                    chunks.push(std::mem::take(&mut current));
                    current.push(prev);
                    width += char_width(prev, font_size, mono);
                } else {
                    chunks.push(std::mem::take(&mut current));
                    width = 0.0;
                }
            } else {
                chunks.push(std::mem::take(&mut current));
                width = 0.0;
            }
        }
        current.push(ch);
        width += ch_w;
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    if chunks.is_empty() {
        chunks.push(String::new());
    }
    chunks
}

fn wrap_plain_text(text: &str, max_width: f32, font_size: f32, mono: bool) -> Vec<String> {
    if text.trim().is_empty() {
        return vec![String::new()];
    }
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        for chunk in split_long_token(word, max_width, font_size, mono) {
            let candidate = if current.is_empty() {
                chunk.clone()
            } else {
                format!("{} {}", current, chunk)
            };
            if approx_text_width(&candidate, font_size, mono) <= max_width {
                current = candidate;
            } else {
                if !current.is_empty() {
                    lines.push(std::mem::take(&mut current));
                }
                current = chunk;
            }
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn wrap_runs(runs: &[InlineRun], max_width: f32, font_size: f32) -> Vec<Vec<InlineRun>> {
    let mut lines: Vec<Vec<InlineRun>> = Vec::new();
    let mut current_line: Vec<InlineRun> = Vec::new();
    let mut current_width = 0.0_f32;

    for run in runs {
        // Formula images are atomic: they cannot be split across lines.
        if let Some(formula) = &run.formula {
            if current_width + formula.width > max_width && !current_line.is_empty() {
                lines.push(std::mem::take(&mut current_line));
                current_width = 0.0;
            }
            current_line.push(InlineRun {
                text: String::new(),
                style: run.style,
                mono: false,
                color: run.color,
                strike: false,
                link: run.link.clone(),
                formula: Some(formula.clone()),
            });
            current_width += formula.width;
            continue;
        }
        for token in tokenize_for_wrap(&run.text) {
            for token in split_long_token(&token, max_width, font_size, run.mono) {
                let tok_w = approx_text_width(&token, font_size, run.mono);
                if current_width + tok_w > max_width
                    && !current_line.is_empty()
                    && token.trim().is_empty()
                {
                    lines.push(std::mem::take(&mut current_line));
                    current_width = 0.0;
                    continue;
                }
                if current_width + tok_w > max_width && !current_line.is_empty() {
                    lines.push(std::mem::take(&mut current_line));
                    current_width = 0.0;
                }
                if let Some(last) = current_line.last_mut() {
                    if last.formula.is_none()
                        && last.style == run.style
                        && last.mono == run.mono
                        && last.color == run.color
                        && last.strike == run.strike
                        && last.link == run.link
                    {
                        last.text.push_str(&token);
                        current_width += tok_w;
                        continue;
                    }
                }
                current_line.push(InlineRun {
                    text: token,
                    style: run.style,
                    mono: run.mono,
                    color: run.color,
                    strike: run.strike,
                    link: run.link.clone(),
                    formula: None,
                });
                current_width += tok_w;
            }
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }
    if lines.is_empty() {
        lines.push(Vec::new());
    }
    lines
}

fn tokenize_for_wrap(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut buf = String::new();
    let mut in_ws = false;
    for c in text.chars() {
        let is_ws = c.is_whitespace();
        if buf.is_empty() {
            in_ws = is_ws;
        }
        if is_ws == in_ws {
            buf.push(c);
        } else {
            out.push(std::mem::take(&mut buf));
            buf.push(c);
            in_ws = is_ws;
        }
    }
    if !buf.is_empty() {
        out.push(buf);
    }
    out
}

fn is_near_white(c: rgb::Color) -> bool {
    c.red() > 240 && c.green() > 240 && c.blue() > 240
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_minimal_document() {
        let opts = PdfExportOptions::default();
        let theme = PdfTheme::print_default();
        let pdf = render_markdown_to_pdf(
            "# Hello\n\nThis is a **bold** test.\n\n- item one\n- item two\n",
            &opts,
            &theme,
            None,
        )
        .expect("render must succeed");
        assert!(pdf.starts_with(b"%PDF-"));
        assert!(pdf.len() > 1000);
    }

    #[test]
    fn h1_page_break_creates_extra_page() {
        let mut opts = PdfExportOptions::default();
        opts.page_break_before_h1 = true;
        opts.include_page_numbers = false;
        let theme = PdfTheme::print_default();
        let with_break =
            render_markdown_to_pdf("# A\n\ntext\n\n# B\n\ntext\n", &opts, &theme, None).unwrap();
        opts.page_break_before_h1 = false;
        let without_break =
            render_markdown_to_pdf("# A\n\ntext\n\n# B\n\ntext\n", &opts, &theme, None).unwrap();
        assert!(with_break.len() >= without_break.len());
    }

    #[test]
    fn wrap_plain_text_breaks_long_word() {
        let word = "supercalifragilisticexpialidocious";
        let lines = wrap_plain_text(word, 50.0, 11.0, false);
        assert!(lines.len() > 1);
        for line in &lines {
            assert!(approx_text_width(line, 11.0, false) <= 50.0);
        }
        assert_eq!(lines.concat(), word);
    }

    #[test]
    fn wrap_plain_text_breaks_cjk_paragraph() {
        let text = "天地玄黄，宇宙洪荒。日月盈昃，辰宿列张。";
        let lines = wrap_plain_text(text, 100.0, 11.0, false);
        assert!(lines.len() > 1);
        for line in &lines {
            assert!(approx_text_width(line, 11.0, false) <= 100.0);
        }
        assert_eq!(lines.concat(), text);
    }

    #[test]
    fn cjk_glyphs_are_full_width() {
        assert_eq!(approx_text_width("中", 10.0, false), 10.0);
        assert_eq!(approx_text_width("，", 10.0, false), 10.0);
        assert_eq!(approx_text_width("a", 10.0, false), 5.0);
        assert_eq!(approx_text_width("i", 10.0, true), 6.0);
    }

    #[test]
    fn tokenize_alternates_words_and_spaces() {
        let tokens = tokenize_for_wrap("hello world  foo");
        assert_eq!(tokens, vec!["hello", " ", "world", "  ", "foo"]);
    }

    #[test]
    fn formula_image_has_display_size() {
        let image = render_formula_image(r"\frac{a}{b}", 12.0, PdfTheme::print_default().body)
            .expect("formula must render");
        assert!(image.width > 0.0);
        assert!(image.height >= 12.0);
        assert_eq!(&image.png[..8], b"\x89PNG\r\n\x1a\n");
    }

    #[test]
    fn math_renders_as_embedded_image() {
        let opts = PdfExportOptions::default();
        let theme = PdfTheme::print_default();
        let with_math = render_markdown_to_pdf(
            "Inline $x^2$ math.\n\n$$\n\\frac{a}{b}\n$$\n",
            &opts,
            &theme,
            None,
        )
        .unwrap();
        let plain =
            render_markdown_to_pdf("Inline math.\n", &opts, &theme, None).unwrap();
        assert!(
            with_math.len() > plain.len() + 500,
            "formula images should add bytes: {} vs {}",
            with_math.len(),
            plain.len()
        );
    }

    #[test]
    #[ignore = "manual walkthrough; writes a sample PDF to the temp dir"]
    fn walkthrough_render_sample() {
        let dir = std::env::temp_dir().join("mdoxide-export-walkthrough");
        std::fs::create_dir_all(&dir).unwrap();
        let pdf = render_markdown_to_pdf(
            crate::export::WALKTHROUGH_MARKDOWN,
            &PdfExportOptions::default(),
            &PdfTheme::print_default(),
            None,
        )
        .expect("walkthrough render must succeed");
        let path = dir.join("walkthrough.pdf");
        std::fs::write(&path, &pdf).unwrap();
        println!("wrote {} ({} bytes)", path.display(), pdf.len());
    }
}
