# Live Pipeline Panel

## Overview

The Live Pipeline panel allows users to pipe the content of JSON and YAML files through shell commands and view the results in real-time. This is particularly useful for:

- Formatting and pretty-printing with tools like `jq` and `yq`
- Filtering and transforming data
- Validating schemas
- Quick data exploration

## Features

### Core Functionality

- **Command Input**: Single-line input field for shell commands with placeholder hint text
- **Run Button**: Manual execution trigger (command also runs on Enter)
- **Recent Commands**: Dropdown history of previously used commands (persisted across sessions)
- **Status Strip**: Shows execution status, exit code, duration, and truncation indicator
- **Output Display**: Monospace text areas for stdout and stderr (split view when stderr has content)

### File Type Support

The pipeline panel is only available for JSON and YAML files. TOML files are excluded because the common command-line tools for TOML processing are less prevalent than `jq`/`yq`.

### Integration

- **Zen Mode**: Pipeline panel is automatically hidden in Zen Mode for distraction-free writing
- **Split View**: Works alongside the split view feature
- **Session Persistence**: Recent commands and panel height are saved with settings

## Usage

### Opening the Panel

1. Open a JSON or YAML file
2. Use one of:
   - Keyboard shortcut: `Ctrl+Shift+L`
   - Ribbon button: Click the ⚡ (lightning) icon in the Format group
   - The panel appears at the bottom of the editor

### Running Commands

1. Type a command in the input field (e.g., `jq '.'`)
2. Press Enter or click the "Run" button
3. View results in the output area

### Example Commands

```bash
# Format/pretty-print JSON
jq '.'

# Extract specific field
jq '.items[]'

# Filter array items
jq '.items[] | select(.status == "active")'

# YAML processing (requires yq installed)
yq '.data'

# Text search
grep 'pattern'
```

## Configuration

### Settings

The following settings can be configured in the settings file:

| Setting | Default | Description |
|---------|---------|-------------|
| `pipeline_enabled` | `true` | Enable/disable the pipeline feature globally |
| `pipeline_debounce_ms` | `500` | Debounce delay before auto-execution (reserved for future) |
| `pipeline_max_output_bytes` | `1048576` (1MB) | Maximum output size before truncation |
| `pipeline_max_runtime_ms` | `30000` (30s) | Maximum runtime before killing process |
| `pipeline_panel_height` | `200.0` | Default panel height in pixels |
| `pipeline_recent_commands` | `[]` | Recent command history |

### Limits

- **Output Size**: Limited to 1MB by default (configurable up to 10MB)
- **Runtime**: Limited to 30 seconds by default (configurable up to 5 minutes)
- **Command History**: Maximum 20 recent commands

## Technical Details

### Process Execution

Commands are executed using:
- **Windows**: `cmd /C <command>`
- **Unix/Linux/macOS**: `sh -c '<command>'`

The current document content is piped to the command's stdin.

### Working Directory

The working directory for command execution is:
1. The directory containing the current file (if saved)
2. The workspace root (if in workspace mode)
3. Current working directory (fallback)

### Error Handling

- Non-zero exit codes are displayed in the status strip
- stderr output is shown in a separate panel with red styling
- Timeout and cancellation are handled gracefully
- Process spawning failures show descriptive error messages

### State Management

Each tab maintains its own pipeline state:
- Current command
- Last stdout/stderr output
- Execution status
- Panel visibility

This allows different files to have different active commands.

### Panel Behavior

- **Instant Animations**: The panel opens/closes instantly with no transition animations
- **Resizable**: Drag the top edge to resize (100-500px range)
- **Height Persistence**: Panel height is saved and restored across sessions
- **Per-Tab Visibility**: Each tab remembers whether the panel is shown

## Security Considerations

⚠️ **Warning**: This feature executes arbitrary shell commands on your machine. Only run commands you trust.

The pipeline feature:
- Runs commands with your user permissions
- Has access to your PATH and environment variables
- Can execute any program available on your system

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+Shift+L` | Toggle pipeline panel |
| `Enter` (in command input) | Execute command |

## Future Enhancements

Potential future improvements (not yet implemented):
- Auto-execution with debounce on document changes
- Syntax highlighting for JSON/YAML output
- Command templates/presets
- Output diff view for comparing transformations
- Shell selection (bash, zsh, PowerShell)
