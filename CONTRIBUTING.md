# Contributing to Ferrite

Thank you for your interest in contributing to Ferrite! This document provides guidelines and instructions for contributing.

## Code of Conduct

Please be respectful and constructive in all interactions. We welcome contributors of all experience levels.

## How to Contribute

### Reporting Bugs

1. **Search existing issues** to avoid duplicates
2. **Use the bug report template** when creating a new issue
3. **Include:**
   - Operating system and version
   - Rust version (`rustc --version`)
   - Steps to reproduce
   - Expected vs actual behavior
   - Screenshots if applicable
   - Relevant log output

### Suggesting Features

1. **Search existing issues** for similar requests
2. **Use the feature request template**
3. **Describe the problem** you're trying to solve
4. **Propose a solution** with details on implementation if possible

### Submitting Code

1. **Fork the repository** and create a feature branch
2. **Make focused changes** - one feature/fix per PR
3. **Follow code style guidelines** (see below)
4. **Test your changes** locally
5. **Submit a pull request** using the PR template

## Development Setup

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs/))
- Platform-specific dependencies (see [README.md](README.md#build-from-source))

### Building

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/Ferrite.git
cd Ferrite

# Build debug version (faster compilation)
cargo build

# Build release version (optimized)
cargo build --release

# Run the application
cargo run
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture
```

## Code Style Guidelines

### Formatting

We use `rustfmt` with default settings. Always format before committing:

```bash
cargo fmt
```

### Linting

We use `clippy` for linting. All warnings should be addressed:

```bash
cargo clippy --all-targets -- -D warnings
```

### Code Conventions

- **Module organization:** Follow the existing module structure in `src/`
- **Naming:**
  - Use `snake_case` for functions, variables, and modules
  - Use `PascalCase` for types and traits
  - Use `SCREAMING_SNAKE_CASE` for constants
- **Documentation:**
  - Add doc comments (`///`) for public items
  - Use `//!` for module-level documentation
  - Include examples in doc comments where helpful
- **Error handling:**
  - Use `Result<T, E>` for fallible operations
  - Prefer `?` operator over explicit matching
  - Provide meaningful error messages

### Example Code Style

```rust
/// Represents an editor tab with its content and state.
pub struct Tab {
    /// Unique identifier for the tab
    pub id: usize,
    /// Display name shown in the tab bar
    pub name: String,
    /// File content
    content: String,
}

impl Tab {
    /// Creates a new tab with the given name.
    ///
    /// # Examples
    ///
    /// ```
    /// let tab = Tab::new("Untitled");
    /// assert_eq!(tab.name, "Untitled");
    /// ```
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: generate_id(),
            name: name.into(),
            content: String::new(),
        }
    }
}
```

## Commit Messages

Follow conventional commit format:

```
type(scope): short description

Longer description if needed.

Fixes #123
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `style`: Formatting, no code change
- `refactor`: Code change that neither fixes a bug nor adds a feature
- `perf`: Performance improvement
- `test`: Adding or fixing tests
- `chore`: Maintenance tasks

**Examples:**
```
feat(editor): add word count to status bar
fix(tabs): prevent crash when closing last tab
docs(readme): update installation instructions
refactor(theme): extract color constants to module
```

## Pull Request Process

1. **Create a feature branch:**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes** with clear, focused commits

3. **Verify all checks pass:**
   ```bash
   cargo fmt --check
   cargo clippy --all-targets -- -D warnings
   cargo test
   cargo build --release
   ```

4. **Push and create PR:**
   ```bash
   git push origin feature/your-feature-name
   ```

5. **Fill out the PR template** completely

6. **Address review feedback** promptly

### PR Requirements

- [ ] Code compiles without errors
- [ ] All tests pass
- [ ] `cargo fmt` produces no changes
- [ ] `cargo clippy` produces no warnings
- [ ] PR description explains the changes
- [ ] Screenshots included for UI changes
- [ ] Documentation updated if needed

## Architecture Overview

```
src/
├── main.rs           # Entry point
├── app.rs            # Main application, UI layout
├── state.rs          # Application state management
├── config/           # Settings and persistence
├── editor/           # Text editor components
├── files/            # File operations
├── markdown/         # Parser and WYSIWYG editor
├── preview/          # Preview and sync scrolling
├── export/           # HTML export
├── theme/            # Theming system
├── ui/               # UI components (ribbon, dialogs, panels)
└── workspaces/       # Workspace/folder support
```

See [docs/index.md](docs/index.md) for detailed technical documentation.

## Translating

Ferrite uses [Weblate](https://hosted.weblate.org/projects/ferrite/) for community translations. You can help translate Ferrite into your language!

### How to Contribute Translations

1. **Visit [Weblate](https://hosted.weblate.org/projects/ferrite/ferrite-ui/)** and create an account
2. **Select your language** (or request a new one)
3. **Start translating** - Weblate provides a user-friendly interface
4. **Submit** - Your translations will be automatically synced via Pull Request

### Translation Guidelines

- **Keep translations concise** - UI elements have limited space
- **Preserve placeholders** - Keep `%{variable}` placeholders intact (e.g., `%{count}`, `%{filename}`)
- **Match tone** - Use a friendly, professional tone consistent with other translations
- **Test context** - Consider where the string appears in the UI

### Adding a New Language

To add support for a new language:

1. Request the language on Weblate, or
2. Create `locales/<language-code>.yaml` based on `locales/en.yaml`

Language codes follow ISO 639-1 (e.g., `de` for German, `fr` for French, `nb` for Norwegian Bokmål).

### Translation File Structure

Translations are stored in `locales/` as YAML files:
- `en.yaml` - English (base language)
- `de.yaml` - German
- `fr.yaml` - French
- etc.

The file uses a nested key structure:
```yaml
menu:
  file:
    label: "File"
    new: "New"
    open: "Open..."
```

## Release Process (Maintainers)

Releases are automated via GitHub Actions when a version tag is pushed (`git tag v0.x.x && git push --tags`).

### Code Signing (Windows)

Windows binaries (`ferrite.exe` and `.msi` installer) are signed via [SignPath.io](https://signpath.io) with a certificate from [SignPath Foundation](https://signpath.org).

**Required GitHub repository secrets:**
- `SIGNPATH_API_TOKEN` — API token from SignPath.io dashboard (user must have submitter permissions)
- `SIGNPATH_ORGANIZATION_ID` — Organization ID from SignPath.io dashboard

The signing configuration is defined in `.signpath/artifact-configuration.xml`.

## Getting Help

- **Documentation:** Check [docs/](docs/) for technical details
- **Issues:** Search existing issues or create a new one
- **Discussions:** Use GitHub Discussions for questions

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
