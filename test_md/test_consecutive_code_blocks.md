# Consecutive Fenced Code Blocks (issue #129)

This file reproduces the issue where only the first fenced code block renders
when several fenced blocks appear back-to-back in rendered / split view.

```text
First plain block
line 2
line 3
```

```csharp
// Second block - C#
public class Foo {
    public int Bar { get; set; }
}
```

```python
# Third block - Python (Run should print Hello)
def hello():
    print("Hello")

hello()
```

```powershell
# PowerShell — Run should show stdout (allow shell blocks in Settings)
Write-Output "Hello from PowerShell"
```

```cmd
REM CMD — Run should show stdout (allow shell blocks in Settings)
echo Hello from CMD
```

```rust
// Sixth block - Rust (no Run — not a supported runner yet)
fn main() {
    println!("Hello");
}
```

```text
Seventh block - plain again
last line
```

End of document.
