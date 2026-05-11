# State diagram: fork, join, and history pseudostates

Native `stateDiagram` / `stateDiagram-v2` parsing and rendering support **synchronization bars** (fork/join) and **history** nodes (shallow `H`, deep `H*`) aligned with common Mermaid syntax.

## Parsing

| Syntax | `State.pseudostate` | Label used for layout/render |
|--------|---------------------|------------------------------|
| `state id <<fork>>` | `Fork` | Empty unless `state "…" as id <<fork>>` supplies text |
| `state id <<join>>` | `Join` | Same as fork |
| Transition endpoint `[H]` | `HistoryShallow` | `H` |
| Transition endpoint `[H*]` | `HistoryDeep` | `H*` |
| `state [H]` / `state [H*]` | Shallow / deep | `H` / `H*` |

Stereotypes are detected via a trailing `<<fork>>` / `<<join>>` suffix after stripping from composite `state … {` headers and simple `state …` lines.

## Layout and rendering

- **Fork/join:** Fixed width (~88px minimum scaled by config), height 16px; drawn as a filled horizontal bar using the diagram stroke color. Optional alias text is drawn below the bar.
- **History:** Uses normal state height and label-based width; drawn as a **circle** (stroke) with centered **H** or **H**\* text.

Transitions use the same anchor logic as other rectangular nodes (wide shallow bars resolve on left/right edges by direction).

## Tests

See `src/markdown/mermaid/state.rs` (`#[cfg(test)] mod tests`) for parse coverage and `validate_mermaid_source` acceptance of combined fork/join/history diagrams.

## Related

- [State composite nested](./state-composite-nested.md) — composite states and nesting
