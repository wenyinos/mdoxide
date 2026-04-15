# Flowchart Test Cases

## Coffee Machine Troubleshooting (TD)

```mermaid
graph TD
    A[Coffee machine not working] --> B{Machine has power?}
    B -->|No| H[Plug in and turn on]
    B -->|Yes| C[Out of beans or water?] -->|Yes| G[Refill beans and  water]
    C -->|No| D{Filter warning?} -->|Yes| I[Replace or clean filter]
    D -->|No| F[Send for repair]
``` 

Expected:
- All 8 nodes should render (A, B, C, D, F, G, H, I)  
- "No" branch from B should be on LEFT, "Yes" branch on RIGHT
- Hierarchy: A -> B -> (H, C) -> (G, D) -> (I, F)
 
## Chapter Flow (LR)

```mermaid
flowchart LR
    a[Chapter 1] --> b[Chapter 2] --> c[Chapter 3]
    c-->d[Using Ledger]
    c-->e[Using Trezor]
    d-->f[Chapter 4]
    e-->f
```

Expected:
- All 6 nodes should render (a, b, c, d, e, f)
- Flow goes left to right
- "Using Ledger" (d) should be ABOVE "Using Trezor" (e)

## Simple Decision (TD)

```mermaid
flowchart TD
    A[Start] --> B{Decision}
    B -->|Yes| C[Process 1]
    B -->|No| D[Process 2]
    C --> E[End]
    D --> E
```

Expected:
- "Yes" (C) should be on LEFT
- "No" (D) should be on RIGHT

## Simple Decision (LR)

```mermaid
flowchart LR
    A[Start] --> B{Decision}
    B -->|Yes| C[Process 1]
    B -->|No| D[Process 2]
    C --> E[End]
    D --> E
```

Expected:
- "Yes" (C) should be on TOP
- "No" (D) should be on BOTTOM

## Subgraph Test (TD)

```mermaid
flowchart TD
    subgraph A[Outer Group]
        a1[Node 1] --> a2[Node 2]
    end
    
    subgraph B[Inner Group]
        b1[Node 3] --> b2[Node 4]
    end
    
    a2 --> b1
```

Expected:
- Two subgraph containers with visible cream/yellow backgrounds
- Titles "Outer Group" and "Inner Group" should be prominent
- Stroke should be clearly visible
- Different depth = alternating fill colors

## Nested Subgraph Test (Sibling Subgraphs)

```mermaid
flowchart TB
    subgraph one[First]
        direction LR
        x1[A] --> x2[B]
    end
    
    subgraph two[Second]  
        direction LR
        y1[C] --> y2[D]
    end
    
    one --> two
```

Expected:
- Two distinct subgraph containers
- Inside "First": A and B should be side-by-side (LR direction)
- Inside "Second": C and D should be side-by-side (LR direction)
- First should be above Second (TB main direction)
- Clear separation and visibility
- Warm cream/yellow background tones

## True Nested Subgraphs (Parent-Child)

```mermaid
flowchart TD
    subgraph outer[Outer Container]
        subgraph inner[Inner Container]
            A[Node A] --> B[Node B]
        end
        C[Node C]
    end
    B --> C
```

Expected:
- "Inner Container" should be fully inside "Outer Container"
- Node C should be below the Inner Container
- Edge from B to C should route properly
- Both titles should be visible and not overlap
- Inner should have different fill color (alternating depth)

## Deeply Nested Subgraphs (3 Levels)

```mermaid
flowchart TD
    subgraph level1[Level 1]
        subgraph level2[Level 2]
            subgraph level3[Level 3]
                deep[Deep Node]
            end
            mid[Mid Node]
        end
        top[Top Node]
    end
    deep --> mid --> top
```

Expected:
- Three nested boxes (level1 > level2 > level3)
- Each level should have visible margins/padding
- All three titles should be visible
- Alternating fill colors for depth
- Edge routing through all levels

## Nested with Direction Override

```mermaid
flowchart TD
    subgraph process[Main Process]
        direction LR
        subgraph input[Input Stage]
            i1[Input 1]
            i2[Input 2]
        end
        subgraph output[Output Stage]
            o1[Output 1]
            o2[Output 2]
        end
        input --> output
    end
```

Expected:
- Main direction is TD (top to bottom)
- "Main Process" has direction LR override
- Input Stage should be LEFT of Output Stage (LR)
- Nodes within Input/Output should be arranged vertically (inherits LR)
- Both nested subgraphs visible with proper titles

## Edge Routing Across Subgraph Boundaries (TD)

```mermaid
flowchart TD
    Start[Start] --> A1
    
    subgraph A[Process A]
        A1[Step 1] --> A2[Step 2]
    end
    
    subgraph B[Process B]
        B1[Step 3] --> B2[Step 4]
    end
    
    A2 --> B1
    B2 --> End[End]
```

Expected:
- Edge from Start to A1 should enter subgraph A cleanly
- Edge from A2 to B1 should exit subgraph A and enter subgraph B cleanly
- Edge from B2 to End should exit subgraph B cleanly
- Edges should route through subgraph borders, not arbitrarily

## Edge Routing Across Subgraph Boundaries (LR)

```mermaid
flowchart LR
    Start[Start] --> A1
    
    subgraph A[Process A]
        A1[Step 1] --> A2[Step 2]
    end
    
    subgraph B[Process B]
        B1[Step 3] --> B2[Step 4]
    end
    
    A2 --> B1
    B2 --> End[End]
```

Expected:
- Same edge routing behavior but in left-to-right direction
- Edges should enter/exit subgraphs at left/right borders

## Complex Cross-Subgraph Routing

```mermaid
flowchart TD
    External1[External Start] --> Inner1
    
    subgraph Group1[Group 1]
        Inner1[Node A] --> Inner2[Node B]
        Inner2 --> Inner3[Node C]
    end
    
    subgraph Group2[Group 2]
        Other1[Node X] --> Other2[Node Y]
    end
    
    Inner3 --> Other1
    Other2 --> External2[External End]
    External1 --> Other1
```

Expected:
- Multiple edges crossing subgraph boundaries
- Edge from External1 to Inner1 enters Group1
- Edge from Inner3 to Other1 exits Group1 and enters Group2
- Edge from External1 to Other1 enters Group2
- All edges should route cleanly through borders

## Asymmetric Shape Test

```mermaid
flowchart TD
    A[Start] --> B>Odd shape]
    B --> C>Really long text in asymmetric]
    C --> D[End]
```

Expected:
- `B` node should render as left-pointing flag/banner shape
- `C` node should size appropriately for longer text
- Shape should have pointed left side, flat right side
- Text should be readable and visually centered

## Asymmetric Shape with Dash-Style Edge Labels

```mermaid
flowchart LR
    od>Odd shape]-- Two line<br/>edge comment --> ro[Result One]
    od -- Simple label --> ro2[Result Two]
    A>Flag]-. Dotted label .-> B[Target]
    X>Banner]== Thick label ==> Y[End]
```

Expected:
- `od` node should render as left-pointing flag/banner (asymmetric), NOT as rectangle
- Edge from `od` to `ro` should have label "Two line\nedge comment" (with line break)
- Edge from `od` to `ro2` should have label "Simple label"
- `A` node should render as asymmetric shape with dotted edge to `B`
- `X` node should render as asymmetric shape with thick edge to `Y`
- All asymmetric nodes should have pointed left side, flat right side

## Standalone Asymmetric Nodes

```mermaid
flowchart TD
    a>Simple flag]
    b>Another asymmetric shape]
    c>Third one]
    a --> b --> c
```

Expected:
- All three nodes (a, b, c) should render as flag/banner shapes
- Vertical layout with edges connecting them

## All Node Shapes

```mermaid
flowchart LR
    A[Rectangle] --> B(Rounded Rect)
    B --> C([Stadium])
    C --> D{Diamond}
    D --> E{{Hexagon}}
    E --> F>Asymmetric]
    F --> G((Circle))
    G --> H[(Cylinder)]
    H --> I[[Subroutine]]
```

Expected:
- All 9 node shapes should render distinctly
- Each shape should match its Mermaid specification

## linkStyle Edge Styling

```mermaid
flowchart TD
    A[Start] --> B[Middle] --> C[End]
    linkStyle 0 stroke:#f00,stroke-width:4px
    linkStyle 1 stroke:#00f,stroke-width:2px
```

Expected:
- First edge (A→B) should be RED with 4px width
- Second edge (B→C) should be BLUE with 2px width

## linkStyle Default

```mermaid
flowchart LR
    A --> B --> C --> D
    linkStyle default stroke:#090
```

Expected:
- All three edges should render in GREEN (#090)

## linkStyle Mixed Index and Default

```mermaid
flowchart TD
    A[Node 1] --> B[Node 2]
    B --> C[Node 3]
    C --> D[Node 4]
    linkStyle default stroke:#333,stroke-width:1px
    linkStyle 1 stroke:#f0f,stroke-width:3px
```

Expected:
- First edge (A→B): GRAY default (#333), 1px width
- Second edge (B→C): MAGENTA (#f0f), 3px width (override)
- Third edge (C→D): GRAY default (#333), 1px width

## linkStyle with Invalid Index

```mermaid
flowchart TD
    A --> B --> C
    linkStyle 99 stroke:#f00
```

Expected:
- All edges render with default style (invalid index 99 ignored)
- No errors or crashes

## YAML Frontmatter with Title

```mermaid
---
title: My Flowchart Title
---
flowchart TD
    A[Start] --> B[Process] --> C[End]
```

Expected:
- Title "My Flowchart Title" displays above the diagram
- Title is bold and slightly larger than diagram text
- Diagram renders normally below the title

## YAML Frontmatter with Config

```mermaid
---
title: Dark Theme Chart
config:
  theme: dark
---
flowchart LR
    A[Input] --> B[Transform] --> C[Output]
```

Expected:
- Title "Dark Theme Chart" displays above the diagram
- Diagram renders normally (config.theme is parsed but not yet applied)

## Frontmatter with Unknown Keys (Graceful Handling)

```mermaid
---
title: Test Chart
unknownKey: someValue
anotherUnknown:
  nested: value
---
flowchart TD
    A --> B --> C
```

Expected:
- Title "Test Chart" displays correctly
- Unknown keys are silently ignored
- No errors or warnings visible

## Empty Frontmatter

```mermaid
---
---
flowchart TD
    A[Empty FM] --> B[Still Works]
```

Expected:
- No title displayed (frontmatter is empty)
- Diagram renders normally
