# Space Bug Investigation

Copy each section from **Rendered mode** and paste into a plain text editor.
Compare with the raw markdown. Mark which sections have extra spaces.

--- 

## Test 1: Inline code adjacent to text

The command `echo hello` prints hello.
Run `npm install` then `npm start` to begin.
Use `git commit -m "fix"` to save.

Expected: no extra space around backtick-delimited code.

---

## Test 2: Mixed inline formatting

This is **bold** and this is *italic* and this is `code` in one line.
Here is **bold text** followed by *italic text* then normal.
A **bold** word next to an *italic* word next to a `code` word.

Expected: single spaces between words, no doubles.

---

## Test 3: Code blocks (syntax highlighted)

### Bash script

```bash
#!/bin/bash
echo "Hello World"
for i in $(seq 1 10); do
  echo "Number: $i"
done
curl -X POST https://api.example.com/data \
  -H "Content-Type: application/json" \
  -d '{"key": "value"}'
```

### Python script

```python
def process_data(input_file, output_file):
    with open(input_file, 'r') as f:
        data = json.loads(f.read())
    result = {k: v.strip() for k, v in data.items()}
    return result
```

### Rust code

```rust
fn main() {
    let values: Vec<i32> = (0..100).filter(|x| x % 2 == 0).collect();
    println!("Found {} even numbers", values.len());
}
```

### JavaScript

```javascript
const fetchData = async (url) => {
    const response = await fetch(url, { headers: { 'Authorization': `Bearer ${token}` } });
    return response.json();
};
```

### PowerShell

```powershell
Get-ChildItem -Path C:\Users -Recurse -Filter *.log | Where-Object { $_.Length -gt 1MB } | Remove-Item -Force
```

Expected: code blocks should copy exactly as written.

---

## Test 4: One-liner commands (inline code)

Install with `pip install requests==2.31.0` and then run `python -m pytest tests/`.
Docker: `docker run -d --name myapp -p 8080:80 -v /data:/app/data nginx:latest`
Regex: `grep -E '^[0-9]{3}-[0-9]{2}-[0-9]{4}$' input.txt`

Expected: commands should be copyable and runnable.

---

## Test 5: Paragraphs with soft breaks

This is a paragraph that
continues on the next line
and wraps across multiple
source lines in markdown.

This is another paragraph with **bold that
spans two lines** in the source.

Expected: soft breaks become single spaces, no doubles.

---

## Test 6: Nested formatting

This has ***bold and italic*** text.
Here is **bold with `code` inside** it.
And *italic with `code` inside* too.
What about **bold *and italic* together**?
Or ~~strikethrough with **bold** inside~~?

Expected: no extra spaces at formatting boundaries.

---

## Test 7: Links adjacent to text

Visit [Google](https://google.com) for search.
Check [the docs](https://docs.rs) and [crates.io](https://crates.io) for packages.
A link [here](http://example.com) then **bold** then `code`.

Expected: no extra spaces around links.

---

## Test 8: Lists with formatting

- Item with `code` in it
- Item with **bold** text
- Item with *italic* and `code` and **bold**
- Plain item
  - Nested with `inline code`
  - Nested with **bold** and *italic*

1. First `command here`
2. Second **important** step
3. Third step with `multiple` code `segments`

Expected: list items should copy cleanly.

---

## Test 9: Tables

| Command | Description | Example |
|---------|-------------|---------|
| `git status` | Show status | `git status -s` |
| `git log` | Show history | `git log --oneline -10` |
| `docker ps` | List containers | `docker ps -a` |

Expected: table content should not have extra padding when copied.

---

## Test 10: Task lists

- [x] Completed task with `code`
- [ ] Pending task with **bold**
- [ ] Another with *italic* and `code`

---

## Test 11: Blockquotes with inline elements

> This is a quote with `code` in it.
> And **bold** and *italic* too.
> Run `echo "hello world"` to test.

---

## Test 12: Exact copy test strings

Copy these lines from rendered mode and compare character-by-character:

```
EXACT:echo "hello"
EXACT:git commit -m "message"
EXACT:curl -s https://api.example.com
EXACT:for i in {1..10}; do echo $i; done
EXACT:pip install flask==2.3.0
```

Plain text versions (not in code block):

The command is `echo "hello"` exactly.
The command is `git commit -m "message"` exactly.
The command is `curl -s https://api.example.com` exactly.

---

## Test 13: Adjacent inline elements (no space between)

**bold***italic* ← should these touch or have a gap?
`code1``code2` ← two code spans touching
**A**B**C** ← bold-plain-bold

---

## Test 14: Long code block line

```bash
ffmpeg -i input.mp4 -c:v libx264 -preset slow -crf 22 -c:a aac -b:a 128k -movflags +faststart -vf "scale=1920:1080:force_original_aspect_ratio=decrease,pad=1920:1080:(ow-iw)/2:(oh-ih)/2" output.mp4
```

Expected: long line should copy as one line without breaks or extra spaces.
