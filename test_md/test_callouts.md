# Callout Rendering Test

Test GitHub-style callouts added in v0.2.7.

## Standard Callouts

> [!NOTE]
> This is a note callout. It should render with a blue/info color and an info icon.

> [!TIP]
> This is a tip callout. Helpful advice for the reader.

> [!WARNING]
> This is a warning callout. Pay attention to potential issues.

> [!CAUTION]
> This is a caution callout. Indicates dangerous or risky actions.

> [!IMPORTANT]
> This is an important callout. Critical information the user needs to know.

## Callouts with Custom Titles

> [!NOTE] Custom Title Here
> This callout has a custom title instead of the default "Note".

> [!WARNING] Breaking Change
> The API signature for `process()` changed in v3.0.

## Collapsible Callouts

> [!NOTE]- Click to expand
> This callout should start collapsed. Click the title to expand it.
> It contains multiple lines of hidden content.
> - Item one
> - Item two
> - Item three

## Callout with Rich Content

> [!TIP]
> You can use **bold**, *italic*, and `code` inside callouts.
>
> Even code blocks:
> ```rust
> fn main() {
>     println!("Hello from a callout!");
> }
> ```
>
> And lists:
> - First item
> - Second item

## Nested Blockquotes vs Callouts

> Regular blockquote — this should NOT render as a callout.
> It's just a normal blockquote.

> > Nested blockquote — also not a callout.

## Edge Cases

> [!NOTE]
> Single line callout.

> [!CAUTION]
>
> Callout with empty line after marker — should still render correctly.
> Content continues here.

> [!TIP]
> Callout followed immediately by text.

Regular paragraph after callout — should have proper spacing.

## Adjacent Callouts

> [!NOTE]
> First callout.

> [!WARNING]
> Second callout immediately after the first.

> [!TIP]
> Third callout in a row.
