# Box Drawing Character Test

This file tests Unicode box drawing characters (U+2500-U+257F).

## Simple Box

```
┌─────────────────────┐
│  Box Drawing Test   │
├─────────────────────┤
│  U+2500 - U+257F    │
└─────────────────────┘
```

## Double Line Box

```
╔═════════════════════╗
║  Double Line Box    ║
╠═════════════════════╣
║  Should show lines  ║
╚═════════════════════╝
```

## Table-like Structure

```
┌────────┬────────┬────────┐
│ Col 1  │ Col 2  │ Col 3  │
├────────┼────────┼────────┤
│ Data A │ Data B │ Data C │
│ Data D │ Data E │ Data F │
└────────┴────────┴────────┘
```

## Mixed Characters

```
─│┌┐└┘├┤┬┴┼
━┃┏┓┗┛┣┫┳┻╋
═║╔╗╚╝╠╣╦╩╬
```

## Inline Box Drawing

The characters ─ (horizontal line), │ (vertical line), ┌ (corner) should render as lines, not squares.

## Common Arrows and Symbols

- Arrow right: →
- Arrow left: ←  
- Check mark: ✓
- Cross mark: ✗
- Bullet: •
- Warning: ⚠

## HTML Block Test

<div>
This is an HTML block that should show «HTML» indicator
</div>
