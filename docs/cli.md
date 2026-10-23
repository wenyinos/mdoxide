# MDOxide Command Line Interface

MDOxide supports command-line arguments for opening files and directories directly from the terminal.

## Usage

```bash
mdoxide [OPTIONS] [PATH]...
```

## Arguments

### `[PATH]...` - Files or directories to open

Pass one or more file paths to open them as tabs, or a directory to open it as a workspace.

**Examples:**

```bash
# Open a single file
mdoxide README.md

# Open multiple files as tabs
mdoxide file1.md file2.md notes.md

# Open current directory as workspace
mdoxide .

# Open a specific directory as workspace
mdoxide ~/projects/my-project

# Mix of files (first file gets focus)
mdoxide README.md CHANGELOG.md docs/getting-started.md
```

## Options

### `-h, --help`

Print help information and exit.

```bash
mdoxide --help
mdoxide -h
```

### `-V, --version`

Print version information and exit.

```bash
mdoxide --version
mdoxide -V
```

### `--log-level <LEVEL>`

Set the log level for debugging. This overrides the `log_level` setting in `config.json`.

Valid values: `debug`, `info`, `warn`, `error`, `off`

```bash
# Enable debug logging
mdoxide --log-level debug

# Show only errors
mdoxide --log-level error

# Disable all logging
mdoxide --log-level off

# Combine with file arguments
mdoxide --log-level debug README.md
```

**Log Level Precedence:**
1. CLI flag (`--log-level`)
2. Config file (`config.json`)
3. Built-in default (`warn`)

## Behavior

### Opening Files

- Each file path is opened as a separate tab
- The first successfully opened file gets focus
- Non-existent files are logged as warnings and skipped
- Relative paths are resolved to absolute paths

### Opening Directories

- A single directory can be passed to open it as a workspace
- The workspace mode enables:
  - File tree navigation
  - Quick switcher (Ctrl+P)
  - Search in files (Ctrl+Shift+F)
  - Git status indicators
- If multiple directories are provided, only the first one is used

### Mixed Arguments

When both files and a directory are provided:

- The directory becomes the workspace root
- Files are opened as tabs within that workspace context

### Error Handling

- Non-existent paths produce a warning in the log but don't crash the app
- Permission errors are logged and the file is skipped
- Invalid paths (neither file nor directory) are skipped with a warning

## Examples

### Basic Usage

```bash
# Start with empty editor
mdoxide

# Edit a markdown file
mdoxide ~/Documents/notes.md

# Open project as workspace
mdoxide ~/projects/my-rust-project
```

### Shell Integration

```bash
# Open file from pipe (not directly supported, use temp file)
# Instead, use shell redirection:
cat file.md | mdoxide /dev/stdin  # Linux/macOS

# Edit git commit message
GIT_EDITOR="mdoxide" git commit

# Open all markdown files in current directory
mdoxide *.md
```

### Scripting

```bash
# Check version in scripts
VERSION=$(mdoxide --version)
echo "Using $VERSION"

# Conditional launch
if [ -f "README.md" ]; then
    mdoxide README.md
fi
```

## Environment Variables

MDOxide reads log configuration from `config.json` and supports CLI overrides. The `RUST_LOG` environment variable is no longer used; use `--log-level` instead.

| Method | Example | Priority |
|--------|---------|----------|
| CLI flag | `mdoxide --log-level debug` | Highest |
| Config file | `"log_level": "debug"` in config.json | Medium |
| Built-in default | `warn` | Lowest |

## Platform Notes

### Windows

```powershell
# PowerShell
.\mdoxide.exe README.md

# CMD
mdoxide.exe README.md

# Open current directory
mdoxide.exe .
```

### Linux/macOS

```bash
# Standard usage
./mdoxide README.md

# If installed to PATH
mdoxide README.md
```

## Related

- [Building from Source](building.md)
- [Configuration](../README.md#configuration)
