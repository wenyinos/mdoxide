# Check for Updates

Manual update checker accessed via **Settings → About → Check for Updates**.

## Overview

Ferrite is an offline-first application. The update checker is the **only** feature that contacts the internet, and it runs **only** when the user explicitly clicks the button. There is no automatic, background, or periodic checking.

## Architecture

```
User clicks button
        │
        ▼
┌──────────────────┐     mpsc channel     ┌──────────────────┐
│  Settings Panel  │ ◄─────────────────── │ Background Thread │
│  (polls rx each  │                      │  (ureq GET to    │
│   frame while    │                      │   GitHub API)    │
│   Checking)      │                      └──────────────────┘
└──────────────────┘
        │
        ▼
  UpdateState displayed inline:
  - Idle → show button
  - Checking → spinner
  - UpToDate → green checkmark
  - UpdateAvailable → card with version + link
  - Error → red message + retry
```

### Key Files

| File | Purpose |
|------|---------|
| `src/update.rs` | Update module: GitHub API, version comparison, URL validation |
| `src/ui/settings.rs` | About section UI, `UpdateState` polling, result display |
| `locales/en.yaml` | Translation keys under `settings.about.*` |

## Security Model

This is documented explicitly because it's the only internet-touching code in the app.

### What we send
- **One HTTPS GET** to `https://api.github.com/repos/OlaProeis/Ferrite/releases/latest`
- **Headers**: `User-Agent: Ferrite/<version>`, `Accept: application/vnd.github+json`
- **No cookies, tokens, telemetry, or user data**

### What we receive and process
- JSON with two fields: `tag_name` (string) and `html_url` (string)
- Deserialized via `serde` into a flat struct — no dynamic evaluation

### URL validation
The `html_url` from the API response is validated before being opened:

```rust
const GITHUB_RELEASES_PREFIX: &str = "https://github.com/OlaProeis/Ferrite/releases/";

// Only open URLs that match our repo's release prefix
if release.html_url.starts_with(GITHUB_RELEASES_PREFIX) {
    release.html_url  // pass through
} else {
    // Construct safe URL from tag name instead
    format!("{}tag/{}", GITHUB_RELEASES_PREFIX, release.tag_name)
}
```

This prevents a compromised API response from redirecting users to a malicious site.

### TLS
- Uses `rustls` (pure-Rust TLS, no OpenSSL C bindings)
- Certificate validation via `webpki-roots` (Mozilla root certificates)
- No certificate pinning beyond standard CA validation

### Dependencies added

| Crate | Purpose | Notes |
|-------|---------|-------|
| `ureq` 2.x | HTTP client | Lightweight, blocking, no tokio required |
| `rustls` | TLS | Pure Rust, widely audited |
| `ring` | Crypto primitives | Used by rustls |
| `webpki-roots` | CA certificates | Mozilla root store |

## Version Comparison

Versions are compared as `(major, minor, patch)` tuples. Pre-release suffixes (e.g., `-hotfix.1`) are stripped before comparison:

```rust
parse_version("v0.2.6-hotfix.1") → Some((0, 2, 6))
parse_version("0.3.0")           → Some((0, 3, 0))

is_newer("v0.2.7", "0.2.6")           → true
is_newer("v0.2.6-hotfix.1", "0.2.6")  → false (same base version)
```

## UI States

The `UpdateState` enum drives what the About section displays:

| State | Display |
|-------|---------|
| `Idle` | "Check for Updates" button |
| `Checking` | Spinner + "Checking for updates..." |
| `UpToDate` | Green "✓ You're on the latest version!" + "Check Again" |
| `UpdateAvailable` | Card with version comparison + "View Release & Download" button |
| `Error` | Red warning + error message + "Try Again" button |

The `UpdateAvailable` state opens the GitHub release page in the user's default browser when they click the link — no in-app downloading.

## Threading Model

The HTTP request runs on a dedicated background thread (`update-checker`) to avoid blocking the UI. Communication uses a standard `mpsc::channel`:

1. Button click → `spawn_update_check()` creates thread + returns `Receiver`
2. Settings panel stores the receiver and sets state to `Checking`
3. Each frame, `try_recv()` polls for the result (non-blocking)
4. While checking, `request_repaint_after(100ms)` ensures we poll regularly
5. On result, state transitions and receiver is dropped

## Tests

Located in `src/update.rs`:

- `test_parse_version_basic` — standard version strings
- `test_parse_version_with_prerelease` — strips `-hotfix.1`, `-beta.2`
- `test_parse_version_invalid` — graceful `None` for bad input
- `test_is_newer` — full comparison matrix (newer, same, older)
- `test_current_version` — `env!("CARGO_PKG_VERSION")` is parseable
- `test_github_url_validation` — URL prefix validation catches spoofs
