# Log Level Configuration

## Overview

Ferrite v0.2.2 introduces configurable log levels, allowing users to control the verbosity of log output. This addresses [GitHub Issue #11](https://github.com/OlaProeis/Ferrite/issues/11).

## Key Files

| File | Purpose |
|------|---------|
| `src/config/settings.rs` | `LogLevel` enum and `log_level` field in Settings |
| `src/main.rs` | CLI flag parsing and logging initialization |
| `docs/cli.md` | CLI documentation with `--log-level` usage |

## Implementation Details

### LogLevel Enum

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,    // Most verbose
    Info,     // Informational messages
    #[default]
    Warn,     // Warnings and errors (default)
    Error,    // Errors only
    Off,      // Disable logging
}
```

### Helper Methods

- `display_name()` - Human-readable name for UI/logging
- `description()` - Detailed description for tooltips
- `all()` - List of all available levels
- `to_level_filter()` - Convert to `log::LevelFilter` for env_logger

### Configuration Precedence

Log level is determined in this order (highest priority first):

1. **CLI flag** - `ferrite --log-level debug`
2. **Config file** - `"log_level": "debug"` in config.json
3. **Built-in default** - `warn`

### Initialization Flow

```rust
// 1. Parse CLI arguments
let cli = Cli::parse();

// 2. Load config (includes log_level)
let settings = load_config();

// 3. Determine effective log level
let effective_log_level = cli.log_level.unwrap_or(settings.log_level);

// 4. Initialize logging
env_logger::Builder::new()
    .filter_level(effective_log_level.to_level_filter())
    .init();
```

## Usage

### CLI Override

```bash
# Enable debug logging
ferrite --log-level debug

# Show only errors  
ferrite --log-level error

# Disable all logging
ferrite --log-level off

# Combine with file arguments
ferrite --log-level debug README.md
```

### Config File

In `config.json`:

```json
{
  "log_level": "warn"
}
```

Valid values: `debug`, `info`, `warn`, `error`, `off`

## Backward Compatibility

- Uses `#[serde(default)]` so existing config files without `log_level` default to `warn`
- Invalid values in config file cause deserialization to use default
- CLI parser accepts aliases: `warning` → `warn`, `none` → `off`

## Tests

Run log level tests:

```bash
cargo test config::settings::tests::test_log_level
```

Tests cover:
- Default value (`Warn`)
- Serialization/deserialization
- All enum variants
- Backward compatibility with old config files
- Level filter conversion

## Related

- [CLI Reference](../cli.md) - Full CLI documentation
- [Settings & Config](./settings-config.md) - Settings system overview
- [GitHub Issue #11](https://github.com/OlaProeis/Ferrite/issues/11) - Original feature request
