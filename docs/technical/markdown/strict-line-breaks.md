# Strict Line Breaks

Optional setting that treats single newlines in markdown source as hard line breaks (`<br>`) in the rendered view.

## Behavior

| Setting | Input | Rendered Output |
|---------|-------|----------------|
| Off (default) | `line one\nline two` | Single paragraph, lines flow together |
| On | `line one\nline two` | Two visual lines separated by a line break |

Standard CommonMark treats a single newline between text as a "soft break" — essentially a space. Enabling strict line breaks overrides this so that every newline in the source produces a visible line break.

## Implementation

### Settings (`src/config/settings.rs`)

```rust
#[serde(default)]
pub strict_line_breaks: bool,
```

- Defaults to `false`
- Persisted via serde serialization (survives restarts)
- Backward-compatible: missing field in old config files defaults to `false`

### Parser (`src/markdown/parser.rs`)

A `hardbreaks` field on `MarkdownOptions` is wired to `comrak::Options::render::hardbreaks`. This follows comrak's naming convention and propagates the setting into the parser options.

### Renderer (`src/markdown/editor.rs`)

The `MarkdownEditor` widget stores `strict_line_breaks` and writes it to egui temporary memory each frame. The `render_inline_node` function reads this flag when processing `SoftBreak` AST nodes:

- **Off**: `SoftBreak` renders as a space (`ui.label(" ")`)
- **On**: `SoftBreak` renders as a line break (`ui.end_row()`)

The `with_settings()` builder method on `MarkdownEditor` automatically picks up the value from `Settings`.

### UI

Toggle available in two locations:

- **Settings Panel** (`src/ui/settings.rs`): Editor section, alongside other toggles
- **Welcome Page** (`src/ui/welcome.rs`): Editor section, alongside word wrap, line numbers, etc.

## Files Changed

| File | Change |
|------|--------|
| `src/config/settings.rs` | Added `strict_line_breaks: bool` field with `#[serde(default)]` |
| `src/markdown/parser.rs` | Added `hardbreaks` to `MarkdownOptions`, wired to comrak render options |
| `src/markdown/editor.rs` | Added field/builder/egui-memory storage, `SoftBreak` conditional rendering |
| `src/ui/settings.rs` | Added checkbox toggle in Editor section |
| `src/ui/welcome.rs` | Added checkbox toggle in Editor section |
