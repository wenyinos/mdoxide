# Rendered paragraph block spacing

Block-level paragraphs in the WYSIWYG rendered/split view append [`PARAGRAPH_TRAILING_SPACE_Y`](../../../src/markdown/editor.rs) (16px) after `render_paragraph` / `render_paragraph_with_structural_keys` so consecutive paragraphs are visibly separated. The same constant is applied after non-mermaid and mermaid code blocks for consistent separation from following content.

[`BLOCK_ITEM_SPACING_Y`](../../../src/markdown/editor.rs) stays at 1px for egui `item_spacing` and viewport overscan math in `show_viewport()`. Per-block heights from the measurement pass (`y_after - y_before`) include the trailing space automatically, so scrollbar range and culling stay aligned.

List item body text does not use `render_node` for paragraphs; spacing between items is unchanged.
