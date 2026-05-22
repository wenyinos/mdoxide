# Executable Code Blocks — Manual Test Suite

Use this file in **Rendered** or **Split** view to exercise fenced-block **Run**.
Open from the repo: `test_md/test_code_execution.md`.

## Before you start

1. **Settings → Editor → Code execution**
   - Enable **Allow code execution** (or click **Run** once and accept the consent dialog).
   - Enable **Allow shell blocks** and **Allow Python blocks**.
   - Enable **Show inline output** (turn off briefly for toast-only tests in §12).
   - Timeout: **30 s** default; use **5 s** for §10 (timeout) only.
2. Save this file to disk so **working-directory** tests (§8) use `test_md/` as cwd.
3. Record: OS, shell availability (Git Bash / WSL / none), Python version.

### Quick pass/fail legend

| Symbol | Meaning |
|--------|---------|
| ✓ | Expected behaviour |
| ⚠ | Platform-dependent — note what you saw |
| ✗ | Bug — describe actual vs expected |

---

## §1 — Run button visibility

These blocks control **which fences show Run**. No need to click Run unless noted.

```rust
// NO Run — unsupported language
fn main() { println!("rust"); }
```

```javascript
// NO Run — unsupported language
console.log("js");
```

```csharp
// NO Run — unsupported language
Console.WriteLine("csharp");
```

```text
plain text fence — NO Run
```

**Expected:** No **Run** on the four blocks above.

---

## §2 — Simple stdout (one line)

### 2a — PowerShell

```powershell
Write-Output "Hello from PowerShell"
```

**Expected:** ✓ Exit 0 · `Hello from PowerShell` · elapsed &lt; 1 s

### 2b — CMD

```cmd
echo Hello from CMD
```

**Expected:** ✓ Exit 0 · `Hello from CMD`

### 2c — Python

```python
print("Hello from Python")
```

**Expected:** ✓ Exit 0 · `Hello from Python` (CRLF on Windows is OK)

### 2d — Bash (Git Bash / WSL / Unix only)

```bash
echo "Hello from Bash"
```

**Expected:** ✓ on Unix or Windows with `bash` in PATH · ⚠ on plain Windows without bash — may fail or run via wrong interpreter (see review notes)

---

## §3 — Multi-line stdout

```powershell
1..5 | ForEach-Object { Write-Output "Line $_" }
Write-Output "Done."
```

**Expected:** ✓ Five numbered lines + `Done.` · panel scrolls if height &gt; ~220 px · **Copy** / **Insert as block** include all lines

---

## §4 — Stderr (separate section)

```powershell
Write-Output "This is stdout"
Write-Error "This is stderr" -ErrorAction Continue
Write-Output "Back to stdout"
```

**Expected:** ✓ Exit non-zero (PowerShell treats Write-Error as error) · stdout and stderr both visible · stderr under `── stderr ──` heading · failure icon (✗)

```python
import sys
print("stdout line")
print("stderr line", file=sys.stderr)
print("stdout again")
```

**Expected:** ✓ Exit 0 · both streams visible · stderr section present

---

## §5 — Exit codes

### 5a — Success (explicit)

```cmd
exit /b 0
```

**Expected:** ✓ Green success · Exit 0 · may show “no output” hint

### 5b — Failure

```cmd
exit /b 42
```

**Expected:** ✓ Red failure · Exit 42 · no output or minimal output

---

## §6 — No output

```python
# Intentionally empty — no print()
x = 1 + 1
```

**Expected:** ✓ Exit 0 · panel shows “no output” message and hint (not a blank scroll area)

```powershell
$null = 1
```

**Expected:** ✓ Same as above — success with no captured bytes

---

## §7 — ANSI colors

```python
print("\033[31mRed text\033[0m normal")
print("\033[1;32mBold green\033[0m")
print("\033[38;5;208mOrange (256-color)\033[0m")
```

**Expected:** ✓ Colored text in inline panel · **Copy** strips ANSI escapes · theme matches terminal palette (dark/light)

---

## §8 — Working directory (save file first)

```powershell
Get-Location | Select-Object -ExpandProperty Path
Split-Path -Leaf (Get-Location)
```

**Expected:** ✓ Path ends with `test_md` (or parent folder where this file lives) · **not** Ferrite install dir

```python
import os
print(os.getcwd())
print(os.path.basename(os.getcwd()))
```

**Expected:** ✓ Same cwd as §8 PowerShell block

---

## §9 — Long output (scroll stress)

```python
for i in range(1, 81):
    print(f"Row {i:03d}: " + "x" * 60)
print("END")
```

**Expected:** ✓ 81 lines · vertical scroll inside output panel (max ~220 px) · horizontal scroll for long lines · **END** visible after scroll · UI stays responsive

---

## §10 — Timeout (set timeout to **5 s** in Settings first)

```python
import time
print("Starting sleep…")
time.sleep(60)
print("Should not appear")
```

**Expected:** ✓ Within ~5 s: **Timed out after 5s** · `Starting sleep…` may appear · final line never shown · **Stop** also works if clicked early (§11)

Restore timeout to **30 s** after this test.

---

## §11 — Stop / cancel

```python
import time
print("Running…")
for i in range(300):
    time.sleep(1)
    print(f"tick {i}")
```

**Expected:** Click **Stop** within a few seconds · **Stopped by user** · partial stdout (e.g. `Running…`, maybe ticks) · **Run** enabled again after finish · no hung spinner

---

## §12 — Toast fallback (disable inline output)

1. Settings → disable **Show inline output**.
2. Run:

```powershell
Write-Output "Toast test message"
```

**Expected:** ✓ No inline panel · toast on completion with output snippet · re-enable inline output before continuing

---

## §13 — Consent dialog (optional reset)

To re-test first-run consent: set `code_execution_consent_acknowledged` to `false` in config **or** use a fresh profile, **disable** master toggle, then click **Run** on:

```powershell
Write-Output "Consent flow test"
```

**Expected:** ✓ Modal appears · **Enable and run** executes and shows output · **Just enable** enables without running · **Cancel** aborts

---

## §14 — Insert as block & Copy

Use §2a or §3, wait for completion, then:

1. **Copy** — paste into Notepad; plain text, no ANSI.
2. **Insert as block** — appends fenced ` ```output ` block **after** the source fence.
3. **Dismiss** — panel clears; **Run** works again.

**Expected:** ✓ Inserted block appears immediately below the code fence in source · rendered view updates

---

## §15 — Carriage-return progress (`\r` overwrite)

```python
import sys, time
for i in range(1, 11):
    sys.stdout.write(f"\rProgress: {i}/10")
    sys.stdout.flush()
    time.sleep(0.15)
print()  # final newline
print("Complete")
```

**Expected:** ✓ Final progress line shows `Progress: 10/10` (not ten separate lines) · `Complete` on its own line · CRLF from Python on Windows must not wipe lines (regression for `\r\n` normalization)

---

## §16 — Mixed blocks in sequence

Run each block top-to-bottom; each should keep its **own** output panel state.

```powershell
Write-Output "Block A"
```

```python
print("Block B")
```

```cmd
echo Block C
```

**Expected:** ✓ Three independent Run buttons · running A does not disable B/C · output attaches to the correct block only

---

## §17 — Error: command not found / spawn failure

```powershell
& 'C:\Definitely\Not\A\Real\Executable.exe'
```

**Expected:** ✓ Failed status · error message in panel (spawn or execution error) · no crash

---

## §18 — Python syntax error

```python
print("missing paren"
```

**Expected:** ✓ Non-zero exit · traceback on stderr · stderr section visible

---

## Notes field (fill in while testing)

| Section | Pass? | Notes |
|---------|-------|-------|
| §1 visibility | | |
| §2 simple | | |
| §3 multiline | | |
| §4 stderr | | |
| §5 exit codes | | |
| §6 no output | | |
| §7 ANSI | | |
| §8 cwd | | |
| §9 long output | | |
| §10 timeout | | |
| §11 stop | | |
| §12 toast | | |
| §13 consent | | |
| §14 insert/copy | | |
| §15 `\r` progress | | |
| §16 sequence | | |
| §17 spawn error | | |
| §18 py syntax | | |

---

## Related docs

- `docs/technical/markdown/code-block-run.md`
- `docs/technical/config/code-execution-settings.md`
- `test_md/test_consecutive_code_blocks.md` — layout regression for multiple fences (issue #129)
