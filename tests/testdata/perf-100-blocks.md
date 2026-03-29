# Performance Test — 100 Blocks

This document has ~100 top-level blocks for testing rendered view viewport culling and block-height caching.

## Section 1: Paragraphs

Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.

Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.

Sed ut perspiciatis unde omnis iste natus error sit voluptatem accusantium doloremque laudantium, totam rem aperiam, eaque ipsa quae ab illo inventore veritatis et quasi architecto beatae vitae dicta sunt explicabo.

Nemo enim ipsam voluptatem quia voluptas sit aspernatur aut odit aut fugit, sed quia consequuntur magni dolores eos qui ratione voluptatem sequi nesciunt.

Neque porro quisquam est, qui dolorem ipsum quia dolor sit amet, consectetur, adipisci velit, sed quia non numquam eius modi tempora incidunt ut labore et dolore magnam aliquam quaerat voluptatem.

## Section 2: Code Blocks

```rust
fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

fn main() {
    for i in 0..20 {
        println!("fib({}) = {}", i, fibonacci(i));
    }
}
```

```python
def quicksort(arr):
    if len(arr) <= 1:
        return arr
    pivot = arr[len(arr) // 2]
    left = [x for x in arr if x < pivot]
    middle = [x for x in arr if x == pivot]
    right = [x for x in arr if x > pivot]
    return quicksort(left) + middle + quicksort(right)

data = [3, 6, 8, 10, 1, 2, 1]
print(quicksort(data))
```

```javascript
class EventEmitter {
  constructor() {
    this.listeners = {};
  }

  on(event, callback) {
    if (!this.listeners[event]) {
      this.listeners[event] = [];
    }
    this.listeners[event].push(callback);
    return this;
  }

  emit(event, ...args) {
    const callbacks = this.listeners[event] || [];
    callbacks.forEach(cb => cb(...args));
    return this;
  }
}
```

## Section 3: Lists

- Item 1: Configure the build system
- Item 2: Set up CI/CD pipeline
- Item 3: Write unit tests
- Item 4: Implement core features
- Item 5: Add documentation
- Item 6: Performance optimization
- Item 7: Security audit
- Item 8: Accessibility review
- Item 9: Localization
- Item 10: Release preparation

1. First step in the deployment process
2. Second step with detailed instructions for the team
3. Third step involving database migration scripts
4. Fourth step for load balancer configuration
5. Fifth step to verify monitoring dashboards

- [ ] Task: Review pull request #123
- [ ] Task: Update dependency versions
- [x] Task: Fix CI pipeline failure
- [x] Task: Add integration tests
- [ ] Task: Write migration guide

## Section 4: Tables

| Feature | Status | Priority | Assignee |
|---------|--------|----------|----------|
| AST Caching | Done | High | Dev A |
| Viewport Culling | Done | High | Dev A |
| Block Height Cache | Done | High | Dev A |
| LSP Integration | Planned | Medium | Dev B |
| Math Rendering | Planned | Low | Dev C |

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| First frame (100 blocks) | 45ms | 45ms | Same |
| Subsequent frames | 45ms | 3ms | 93% |
| After single edit | 45ms | 5ms | 89% |
| Memory overhead | 0 | ~50KB | Bounded |

## Section 5: Blockquotes and Callouts

> This is a regular blockquote that spans multiple lines. It contains enough text to wrap and create a block of reasonable height for testing measurement caching.

> [!NOTE]
> This is a note callout. The viewport culling system treats each top-level block independently, so callouts are individual blocks that can be culled.

> [!WARNING]
> Performance testing should be done in release mode (`cargo run --release`) to get accurate measurements. Debug builds include extra checks that significantly impact rendering speed.

> [!TIP]
> Use the browser DevTools or egui's built-in frame time display to measure rendering performance. Look at the frame time, not just perceived smoothness.

## Section 6: More Paragraphs (Filler)

At vero eos et accusamus et iusto odio dignissimos ducimus qui blanditiis praesentium voluptatum deleniti atque corrupti quos dolores et quas molestias excepturi sint occaecati cupiditate non provident.

Similique sunt in culpa qui officia deserunt mollitia animi, id est laborum et dolorum fuga. Et harum quidem rerum facilis est et expedita distinctio.

Nam libero tempore, cum soluta nobis est eligendi optio cumque nihil impedit quo minus id quod maxime placeat facere possimus, omnis voluptas assumenda est.

Temporibus autem quibusdam et aut officiis debitis aut rerum necessitatibus saepe eveniet ut et voluptates repudiandae sint et molestiae non recusandae.

Itaque earum rerum hic tenetur a sapiente delectus, ut aut reiciendis voluptatibus maiores alias consequatur aut perferendis doloribus asperiores repellat.

## Section 7: Mixed Content

Here is a paragraph with **bold text**, *italic text*, `inline code`, and ~~strikethrough~~. It also contains a [link](https://example.com) and an inline formula reference.

---

Another paragraph after a thematic break. This tests that horizontal rules are counted as separate blocks and culled correctly.

---

### Subsection with nested content

> A blockquote inside a subsection:
>
> - List item inside blockquote
> - Another list item
> - Third item with **bold**

```toml
[package]
name = "ferrite"
version = "0.2.7"
edition = "2021"

[dependencies]
egui = "0.28"
comrak = "0.22"
blake3 = "1.5"
```

Final paragraph of the mixed content section. This document should have approximately 100 top-level AST blocks when parsed by comrak. Each block gets an independent height measurement that can be cached.

## Section 8: Repeated Paragraphs for Bulk

Paragraph 1: The quick brown fox jumps over the lazy dog. This sentence contains every letter of the English alphabet, making it useful for font testing and typography samples.

Paragraph 2: Pack my box with five dozen liquor jugs. Another pangram useful for testing different character combinations and their rendering in various fonts.

Paragraph 3: How vexingly quick daft zebras jump! Yet another pangram, this time with an exclamation mark to test punctuation rendering.

Paragraph 4: The five boxing wizards jump quickly. A shorter pangram that tests compact block rendering and caching behavior.

Paragraph 5: Sphinx of black quartz, judge my vow. This pangram has a more poetic quality and tests comma handling in rendered paragraphs.

Paragraph 6: Two driven jocks help fax my big quiz. Testing numbers spelled out and their effect on line wrapping at different widths.

Paragraph 7: The jay, pig, fox, zebra, and my wolves quack! Testing multiple commas and an exclamation in a single paragraph block.

Paragraph 8: Crazy Frederick bought many very exquisite opal jewels. A longer pangram that should create a taller block when line-wrapped in narrow viewports.

Paragraph 9: We promptly judged antique ivory buckles for the next prize. Testing line wrapping behavior with different available widths.

Paragraph 10: A mad boxer shot a quick, gloved jab to the jaw of his dizzy opponent. The longest pangram here, testing how multi-line paragraphs are measured and cached.

## Section 9: Nested Lists

- Top level item A
  - Nested item A.1
  - Nested item A.2
    - Deep nested A.2.1
    - Deep nested A.2.2
  - Nested item A.3
- Top level item B
  - Nested item B.1
  - Nested item B.2
- Top level item C
  - Nested item C.1
    - Deep nested C.1.1
      - Very deep C.1.1.1
      - Very deep C.1.1.2
    - Deep nested C.1.2

1. Ordered top level
   1. Ordered nested 1.1
   2. Ordered nested 1.2
2. Ordered top level 2
   1. Ordered nested 2.1
   2. Ordered nested 2.2
   3. Ordered nested 2.3

## Section 10: Final Section

This is the last section of the 100-block test document. If you're viewing this in Rendered mode:

- **First frame**: All blocks are rendered (measurement pass) — block heights are cached
- **Subsequent frames**: Only visible blocks (~10-20) are rendered via viewport culling
- **After editing one paragraph**: Only the edited block is re-measured; all others use cached heights

Performance should be noticeably better than rendering all ~100 blocks every frame.
