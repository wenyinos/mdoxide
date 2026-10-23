//! Document Export Module for MDOxide
//!
//! This module provides functionality for exporting markdown documents to various formats
//! including standalone themed HTML and copying HTML to clipboard.
//!
//! # Supported Export Formats
//!
//! - **HTML File**: Complete HTML document with inlined theme CSS
//! - **Clipboard HTML**: Copy rendered HTML to clipboard for pasting in other apps
//!
//! # Architecture
//!
//! - `options.rs` - Export configuration and options
//! - `html.rs` - HTML document generation with theme styling
//! - `clipboard.rs` - Platform clipboard operations

pub mod clipboard;
pub mod flowchart_svg;
pub mod html;
pub mod html_options;
pub mod options;
pub mod pdf;

pub use clipboard::copy_html_to_clipboard;
pub use html::{
    generate_html_document_export, resolve_html_theme_for_export, syntax_dark_mode_for_export,
};
pub use html_options::{HtmlExportOptions, HtmlExportThemeChoice};
pub use pdf::{render_markdown_to_pdf, PdfExportOptions, PdfMarginPreset, PdfPageSize, PdfTheme};

/// Read the pixel dimensions from a PNG header (IHDR chunk).
pub(crate) fn png_dimensions(png: &[u8]) -> Option<(u32, u32)> {
    const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
    if png.len() < 24 || &png[..8] != PNG_SIGNATURE {
        return None;
    }
    let width = u32::from_be_bytes(png[16..20].try_into().ok()?);
    let height = u32::from_be_bytes(png[20..24].try_into().ok()?);
    Some((width, height))
}

/// Shared sample used by the manual HTML/PDF export walkthrough tests.
#[cfg(test)]
pub(crate) const WALKTHROUGH_MARKDOWN: &str = r#"# 导出走查 / Export Walkthrough

这是一段中文正文，用于检查 PDF 中文字体、字距与换行。Mixed English words should render with the body font while 中文继续显示。

## 行内格式

**粗体中文**、*斜体中文*、`inline_code 行内代码`、~~删除线~~、[链接](https://example.com)。

> 引用块：天地玄黄，宇宙洪荒。

## 列表

1. 第一项：苹果
2. 第二项：香蕉
   - 嵌套项：葡萄

- [x] 已完成任务
- [ ] 未完成任务

## 表格

| 名称 | 数量 | 说明 |
| --- | ---: | --- |
| 苹果 | 3 | 红色水果 |
| Apple | 5 | English row |

## 代码

```rust
fn main() {
    // 中文注释：打印问候
    println!("你好，世界！");
}
```

## 公式

行内公式 $E = mc^2$ 与块级公式：

$$
\int_0^1 x^2 dx = \frac{1}{3}
$$

## 长中文段落（换行检查）

天地玄黄，宇宙洪荒。日月盈昃，辰宿列张。寒来暑往，秋收冬藏。闰余成岁，律吕调阳。云腾致雨，露结为霜。金生丽水，玉出昆冈。剑号巨阙，珠称夜光。果珍李柰，菜重芥姜。

## 结束

The quick brown fox jumps over the lazy dog. 0123456789.
"#;

