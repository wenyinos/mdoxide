# Vim Mode Test File

Use this file to test Vim mode editing (v0.2.7 feature).

**Setup:** Enable Vim mode in Settings -> Editor -> Vim Mode.

## Normal Mode Commands

Test each command below. Place cursor on the indicated line and execute.

### Movement (hjkl)
```
h - move left
j - move down
k - move up
l - move right
```

Practice line: The quick brown fox jumps over the lazy dog.

### Word Movement
```
w - next word start
b - previous word start
e - end of word
```

Practice: one two three four five six seven eight nine ten

### Line Movement
```
0 - start of line
$ - end of line
^ - first non-whitespace
```

    Indented line for testing ^ vs 0 movement.

### Delete Commands
```
dd  - delete entire line
x   - delete character under cursor
dw  - delete word
d$  - delete to end of line
```

DELETE THIS LINE with dd
Delete FROM x HERE to end with d$
Delete the FIRST word on this line with dw

### Yank and Paste
```
yy - yank (copy) line
p  - paste below
P  - paste above
```

Copy this line with yy, then paste with p below.

### Visual Mode
```
v  - character-wise visual
V  - line-wise visual
```

Select THIS WORD in visual mode and delete with d.
Select these three lines
with V (line visual mode)
and yank with y.

### Search
```
/pattern - search forward
n        - next match
N        - previous match
```

Find the word NEEDLE in this HAYSTACK. There is another NEEDLE here.
And one more NEEDLE at the end.

## Mode Indicator

- [ ] Status bar shows [NORMAL] when in Normal mode
- [ ] Status bar shows [INSERT] when in Insert mode (press i)
- [ ] Status bar shows [VISUAL] when in Visual mode (press v or V)
- [ ] Pressing Escape returns to Normal mode from any mode

## Insert Mode Entry

```
i - insert before cursor
a - append after cursor
o - open line below
O - open line above
A - append at end of line
I - insert at beginning of line
```

## Ctrl+ Shortcuts in Vim Mode

- [ ] Ctrl+S saves the file (works in all modes)
- [ ] Ctrl+Z undoes (works in all modes)
- [ ] Ctrl+C copies selected text
- [ ] Ctrl+V pastes text
