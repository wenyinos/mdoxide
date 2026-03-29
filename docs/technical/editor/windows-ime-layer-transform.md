# Windows IME: layer transform for `IMEOutput`

## Overview

`FerriteEditor` reports IME cursor geometry to the OS via `egui::output::IMEOutput`. Those rectangles must be in **screen space** (after the widget layer’s transform). This matches egui’s built-in `TextEdit` and fixes misaligned IME candidate windows on Windows when the editor lives in scaled or transformed UI (e.g. custom title bar).

## Key Files

- `src/editor/ferrite/editor.rs` — Sets `IMEOutput` after painting cursors and preedit; applies layer `TSTransform` to `rect` and `cursor_rect`.

## Implementation Details

egui **0.28.1** does not provide `layer_transform_to_global()` (that name does not exist in this release). Upstream `TextEdit` reads `ui.memory(|m| m.layer_transforms.get(&ui.layer_id()))`, then uses `transform * rect` and `transform * cursor_rect`. Ferrite uses the same pattern so OS-level IME positioning stays consistent with egui’s coordinate system.

If candidate **z-order** or placement is still wrong after this, further work may require Win32-specific behavior outside egui’s portable output.

## Dependencies Used

- **egui** — `LayerId` transforms in `Memory::layer_transforms`, `TSTransform` × `Rect`, `IMEOutput`.

## Usage

**Test (Windows):** Enable an IME (e.g. Microsoft Pinyin), compose in the raw editor; the candidate list should sit near the composition cursor. Toggle custom window decorations and light/dark themes as a smoke test.

## References

- Upstream: `egui` `text_edit` builder IME block (~lines 729–738 in egui 0.28.1).
- Related: GitHub #103, #15; ROADMAP (Windows IME).
