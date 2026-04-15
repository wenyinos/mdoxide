# Task 50 — Table inline formatting (manual test)

Use this file to verify **strikethrough**, **bold**, *italic*, and nested combinations stay intact when editing tables in rendered/WYSIWYG mode and when switching **Raw ↔ Rendered**.

## Core formatting matrix

| Style | Example cell | Notes |
|-------|--------------|-------|
| Strikethrough | ~~removed text~~ | Should stay crossed out in rendered view |
| Bold | **strong emphasis** | Heavier weight |
| Italic | *slanted text* | Slanted |
| Bold + italic | ***bold italic*** | Nested emphasis |
| Strikethrough + bold | **~~bold struck~~** | Both styles visible |
| Strikethrough + italic | *~~italic struck~~* | Both styles visible |
| Triple nest | ***~~bold italic struck~~*** | Stress test |
| Plain mix | plain then **bold** then plain | Markers only around bold span |

## Edge cases

| Case | Cell A | Cell B |
|------|--------|--------|
| Empty | | (empty) |
| Only markers | ** | ~~ |
| Spaces | ` ` (single space) | x |
| Mixed with code | has `inline code` and **bold** | ~~struck~~ near `code` |
| Link + format | [link](https://example.com) | [**bold link**](https://example.com) |

## Round-trip checklist

1. Open this file in **Rendered** (or split) view.
2. Click into a formatted cell; edit a character inside the formatted span.
3. Switch to **Raw** view: confirm `~~`, `*`, `**`, `***` markers are still present and not duplicated or stripped.
4. Switch back to **Rendered**: styling should match before the edit.
5. Save, close tab, reopen: content unchanged.

## Regression (plain tables)

| Name | Value |
|------|-------|
| Alice | 42 |
| Bob | 17 |

Plain cells should behave as before (no spurious markers).

## Optional: many cells (light perf smoke)

| C1 | C2 | C3 | C4 | C5 | C6 | C7 | C8 | C9 | C10 |
|----|----|----|----|----|----|----|----|----|-----|
| ~~a1~~ | **b1** | *c1* | ***d1*** | plain | ~~**e1**~~ | *~~f1~~* | `g1` | [h1](https://example.com) | **i1** |
| ~~a2~~ | **b2** | *c2* | ***d2*** | plain | ~~**e2**~~ | *~~f2~~* | `g2` | [h2](https://example.com) | **i2** |
| ~~a3~~ | **b3** | *c3* | ***d3*** | plain | ~~**e3**~~ | *~~f3~~* | `g3` | [h3](https://example.com) | **i3** |
| ~~a4~~ | **b4** | *c4* | ***d4*** | plain | ~~**e4**~~ | *~~f4~~* | `g4` | [h4](https://example.com) | **i4** |
| ~~a5~~ | **b5** | *c5* | ***d5*** | plain | ~~**e5**~~ | *~~f5~~* | `g5` | [h5](https://example.com) | **i5** |
| ~~a6~~ | **b6** | *c6* | ***d6*** | plain | ~~**e6**~~ | *~~f6~~* | `g6` | [h6](https://example.com) | **i6** |
| ~~a7~~ | **b7** | *c7* | ***d7*** | plain | ~~**e7**~~ | *~~f7~~* | `g7` | [h7](https://example.com) | **i7** |
| ~~a8~~ | **b8** | *c8* | ***d8*** | plain | ~~**e8**~~ | *~~f8~~* | `g8` | [h8](https://example.com) | **i8** |
| ~~a9~~ | **b9** | *c9* | ***d9*** | plain | ~~**e9**~~ | *~~f9~~* | `g9` | [h9](https://example.com) | **i9** |
| ~~a10~~ | **b10** | *c10* | ***d10*** | plain | ~~**e10**~~ | *~~f10~~* | `g10` | [h10](https://example.com) | **i10** |

Scroll and edit a few cells; UI should stay responsive (no obvious lag).
