# Adding Languages to Ferrite

This guide explains how to add new translations to Ferrite and set up community translation workflows.

## Current Status

| Component | Status |
|-----------|--------|
| i18n Infrastructure | ✅ Complete |
| English (en) | ✅ Complete (base language) |
| Language Selector | ✅ In Settings → Appearance |
| Auto-detect System Locale | ✅ First launch detection |
| Translation Portal | 🔜 To be set up |

## Quick Start: Adding a New Language

### Step 1: Create the Locale File

Copy `locales/en.yaml` to a new file with the appropriate locale code:

```bash
# Examples:
cp locales/en.yaml locales/zh-CN.yaml   # Chinese (Simplified)
cp locales/en.yaml locales/ja.yaml      # Japanese
cp locales/en.yaml locales/ko.yaml      # Korean
cp locales/en.yaml locales/de.yaml      # German
cp locales/en.yaml locales/fr.yaml      # French
```

### Step 2: Translate the Strings

Edit the new file and translate each string value. Keep the keys (left side) unchanged:

```yaml
# locales/zh-CN.yaml
app:
  name: "Ferrite"                        # Keep product name
  tagline: "快速、轻量的 Markdown 文本编辑器"

menu:
  file:
    label: "文件"
    new: "新建"
    open: "打开..."
    save: "保存"
    # ... translate all strings
```

**Important:**
- Keep YAML structure intact (indentation matters!)
- Keep placeholder variables like `%{filename}`, `%{count}`, `%{path}` unchanged
- Don't translate keys, only values
- Some strings (product names, technical terms) may stay in English

### Step 3: Register the Language in Code

Edit `src/config/settings.rs` to add the new language variant:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[default]
    #[serde(rename = "en")]
    English,
    
    // Add new language:
    #[serde(rename = "zh-CN")]
    ChineseSimplified,
}

impl Language {
    pub fn locale_code(&self) -> &'static str {
        match self {
            Language::English => "en",
            Language::ChineseSimplified => "zh-CN",
        }
    }

    pub fn native_name(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::ChineseSimplified => "简体中文",
        }
    }

    pub fn all() -> &'static [Language] {
        &[
            Language::English,
            Language::ChineseSimplified,
        ]
    }

    pub fn from_locale_code(locale: &str) -> Option<Language> {
        let normalized = locale.to_lowercase().replace('_', "-");
        let primary_lang = normalized.split('-').next().unwrap_or(&normalized);

        match primary_lang {
            "en" => Some(Language::English),
            "zh" => Some(Language::ChineseSimplified),
            _ => None,
        }
    }
}
```

### Step 4: Build and Test

```bash
cargo build
cargo run

# Or run tests:
cargo test settings::tests::test_language
```

### Step 5: Test in the Application

1. Delete `%APPDATA%\ferrite\config.json` (Windows) or `~/.config/ferrite/config.json` (Linux/macOS)
2. Launch Ferrite - should detect system locale
3. Go to Settings → Appearance → Language
4. Select the new language
5. Verify all UI strings are translated

---

## Translation Portal Options

For community-driven translations, consider these platforms:

### Option 1: Crowdin (Recommended)

**Pros:**
- Free for open-source projects
- Great UI for translators
- Supports YAML format
- Automatic PR creation for completed translations
- Translation memory and glossary
- Quality assurance checks

**Setup:**
1. Sign up at [crowdin.com](https://crowdin.com)
2. Create new project → Open Source
3. Add source file: `locales/en.yaml`
4. Configure target languages
5. Set up GitHub integration for automatic sync

**Crowdin Config (`.crowdin.yml`):**
```yaml
project_id: "ferrite"
api_token_env: "CROWDIN_API_TOKEN"
base_path: "."
base_url: "https://api.crowdin.com"

files:
  - source: /locales/en.yaml
    translation: /locales/%locale%.yaml
```

### Option 2: Weblate

**Pros:**
- Self-hostable (or use hosted version)
- Git-native workflow
- Strong privacy focus
- YAML support

**Setup:**
1. Create project at [hosted.weblate.org](https://hosted.weblate.org) (free for FOSS)
2. Connect to GitHub repository
3. Configure YAML file format
4. Add `locales/en.yaml` as source

**Critical (avoids broken PRs):** In the component settings, set **Monolingual base language file** (or **Template for new translations**) to **`locales/en.yaml`**. That way Weblate mirrors the same key structure and nesting as the English file; translators only fill values. Without this, new keys can end up under wrong sections (e.g. `find.about` instead of `about` or `settings.about`) and the app’s `t!("key.path")` lookups fail. See [Weblate PR 90 review](i18n/weblate-pr90-review.md) for what went wrong and how to fix the component.

### Option 3: POEditor

**Pros:**
- Simple interface
- Good for smaller projects
- Supports YAML

### Option 4: Lokalise

**Pros:**
- AI-assisted translations
- Excellent for professional workflows
- GitHub integration

---

## Translation Guidelines

### For Translators

1. **Keep placeholders intact:**
   ```yaml
   # ✅ Correct
   message: "¿Desea guardar los cambios en \"%{filename}\"?"
   
   # ❌ Wrong - placeholder removed
   message: "¿Desea guardar los cambios en el archivo?"
   ```

2. **Preserve formatting:**
   ```yaml
   # Keep newlines, special characters as in English
   ```

3. **Don't translate:**
   - Product name "Ferrite"
   - Technical terms when no good equivalent exists
   - Keyboard shortcuts (Ctrl, Shift, etc.)

4. **Consider context:**
   - Menu items should be concise
   - Tooltips can be more descriptive
   - Error messages should be helpful

### For Reviewers

- Check placeholder preservation
- Verify consistent terminology
- Test in context (some strings may be truncated in UI)
- Check for grammatical correctness

---

## File Structure

```
locales/
├── en.yaml          # English (base/source)
├── zh-CN.yaml       # Chinese (Simplified)
├── zh-TW.yaml       # Chinese (Traditional)
├── ja.yaml          # Japanese
├── ko.yaml          # Korean
├── de.yaml          # German
├── fr.yaml          # French
├── es.yaml          # Spanish
└── pt-BR.yaml       # Portuguese (Brazil)
```

## Locale Codes Reference

| Language | Code | Native Name |
|----------|------|-------------|
| English | `en` | English |
| Chinese (Simplified) | `zh-CN` | 简体中文 |
| Chinese (Traditional) | `zh-TW` | 繁體中文 |
| Japanese | `ja` | 日本語 |
| Korean | `ko` | 한국어 |
| German | `de` | Deutsch |
| French | `fr` | Français |
| Spanish | `es` | Español |
| Portuguese (Brazil) | `pt-BR` | Português (Brasil) |
| Russian | `ru` | Русский |
| Arabic | `ar` | العربية |

---

## Testing Translations

### Manual Testing Checklist

- [ ] All menu items display correctly
- [ ] Settings panel fully translated
- [ ] Dialogs (save, open, confirm) translated
- [ ] Error messages translated
- [ ] Keyboard shortcuts still work
- [ ] No text overflow/truncation issues
- [ ] Placeholder variables work (`%{filename}`, etc.)

### Automated Tests

```bash
# Run all i18n-related tests
cargo test i18n
cargo test settings::tests::test_language

# Check for missing keys (compare with en.yaml)
# TODO: Add validation script
```

---

## String Categories

The `en.yaml` file is organized into these categories:

| Category | Description | Example Keys |
|----------|-------------|--------------|
| `app` | Application metadata | `app.name`, `app.tagline` |
| `menu` | Menu bar items | `menu.file.open`, `menu.edit.undo` |
| `toolbar` | Toolbar button labels | `toolbar.new_file`, `toolbar.bold` |
| `status` | Status bar | `status.line`, `status.modified` |
| `dialog` | Dialog boxes | `dialog.unsaved_changes.*` |
| `settings` | Settings panel | `settings.editor.font_size` |
| `find` | Find/Replace panel | `find.placeholder`, `find.replace_all` |
| `outline` | Outline panel | `outline.title`, `outline.no_headings` |
| `sidebar` | File tree sidebar | `sidebar.files`, `sidebar.open_folder` |
| `error` | Error messages | `error.file_not_found` |
| `notification` | Toast notifications | `notification.file_saved` |
| `shortcuts` | Keyboard shortcuts dialog | `shortcuts.category.*` |
| `about` | About dialog | `about.version`, `about.description` |
| `git` | Git integration | `git.modified`, `git.branch` |
| `a11y` | Accessibility labels | `a11y.close_button` |

---

## Contributing Translations

### Via GitHub

1. Fork the repository
2. Create locale file: `locales/<code>.yaml`
3. Translate strings
4. Update `src/config/settings.rs` (add Language variant)
5. Submit PR with title: `i18n: Add <Language> translation`

### Via Translation Portal

*(Once set up)*

1. Visit [translation portal URL]
2. Select your language
3. Translate strings in the web interface
4. Translations are automatically synced to GitHub

---

## Technical Details

### rust-i18n Crate

Ferrite uses [rust-i18n](https://crates.io/crates/rust-i18n) for internationalization:

```rust
// In src/main.rs
rust_i18n::i18n!("locales", fallback = "en");

// Usage in code
use rust_i18n::t;

let message = t!("menu.file.open");  // "Open..."
let msg = t!("dialog.unsaved_changes.message", filename = "test.md");
```

### Locale Files Format

YAML format with nested keys:

```yaml
category:
  subcategory:
    key: "Translated string"
    key_with_placeholder: "Hello, %{name}!"
```

### Fallback Behavior

If a translation is missing, rust-i18n falls back to English (`en`).

---

## Next Steps

1. **Set up Crowdin** (or chosen platform)
2. **Add priority languages**: Chinese, Japanese, Korean
3. **Create validation script** to check for missing keys
4. **Add CI checks** for translation completeness
5. **Document contributor workflow** on translation portal

---

*Last updated: 2026-01-14*
