# Technology Stack Research

**Project:** Ferrite v0.5.0 "Swiss Army Knife"
**Researched:** 2026-01-24
**Confidence:** HIGH

## Executive Summary

This research identifies Rust crates needed for four new feature categories in Ferrite: Developer Productivity Hub, AI-First Editor, Power Terminal, and Data & Database Tools. All recommendations prioritize lightweight integration with the existing egui/eframe architecture while maintaining Ferrite's "powerful fast lightweight customizable" philosophy.

**Key Finding:** The async runtime (tokio) is the critical integration point. All new features (AI streaming, SSH, database queries, GitHub API) require async operations in an egui immediate-mode GUI. The poll-promise pattern is the recommended bridge.

---

## 1. AI/LLM Integration

### Primary Recommendation: Build Custom HTTP Client

**Rationale:** Use `reqwest` directly instead of specialized AI SDKs to minimize dependencies and maintain control.

| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| `reqwest` | 0.12.26 | HTTP client for Claude/Ollama APIs | Industry standard, lightweight, async-first |
| `serde_json` | 1.0.145 | JSON parsing for LLM responses | Zero-cost serialization, ubiquitous |
| `eventsource-client` | Latest | SSE streaming for Claude responses | Async streaming, auto-reconnect |
| `tokio` | 1.49.0 | Async runtime | Required for all async operations |
| `poll-promise` | Latest | Egui async bridge | Recommended by egui for async tasks |

**Integration Pattern:**
```toml
[dependencies]
reqwest = { version = "0.12", features = ["json", "stream"] }
serde_json = "1.0"
eventsource-client = "0.13"
tokio = { version = "1.49", features = ["rt-multi-thread", "macros"] }
poll-promise = "0.3"
```

**Why NOT Heavy SDKs:**
- `async-openai` (v0.28): Adds 50+ dependencies, OpenAI-focused
- `ollama-rs`: Good for dedicated Ollama apps, but overkill for multi-provider support
- Custom HTTP gives flexibility for Claude, Ollama, and future providers with minimal bloat

**Streaming Implementation:**
- Use `eventsource-client` for Claude's SSE streaming
- Use `poll-promise::Promise::spawn_async` to bridge tokio async with egui
- Update UI each frame by polling promise state

---

## 2. Database Tools

### SQLite: rusqlite (Direct Access)
### PostgreSQL: sqlx (Async Queries)

| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| `rusqlite` | 0.38.0 | SQLite browser/query | Lightweight, bundled SQLite, no external deps |
| `sqlx` | 0.8.6 | PostgreSQL async queries | Compile-time checked, no ORM bloat |

**Rationale:**

**rusqlite for SQLite:**
- Bundles SQLite 3.51.1 (no system dependency)
- Low-level API = full control over queries
- Perfect for read-only database browsing
- 169 KiB package size
- No ORM = no magic, no surprises

**sqlx for PostgreSQL:**
- Async-first (works with tokio runtime)
- Compile-time query verification (optional feature)
- Supports connection pooling
- Raw SQL (no DSL to learn)
- Active security updates (0.8.2 fixed RUSTSEC-2024-0363)

**Why NOT Diesel:**
- Diesel requires native PostgreSQL client libraries (adds system deps)
- ORM DSL is heavyweight for a database browser
- Sync-only (harder to integrate with async stack)
- Better for apps that generate queries, not execute user SQL

**Configuration:**
```toml
[dependencies]
rusqlite = { version = "0.38", features = ["bundled"] }
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "sqlite"] }
```

**Note:** SQLx can handle both SQLite AND PostgreSQL, but rusqlite is lighter for SQLite-only operations. Consider starting with just rusqlite + sqlx postgres features.

---

## 3. Developer Productivity Hub

### GitHub/GitLab API + Timers + Storage

| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| `octocrab` | 0.49.3 | GitHub API client | Typed API, webhooks, actively maintained |
| `gitlab` | 0.18+ | GitLab API client | Builder pattern, tracks GitLab 18.7 API v4 |
| `serde` | 1.0 | Serialization for API responses | Already in project (used elsewhere) |
| `chrono` | Latest | Pomodoro timer, timestamps | De facto Rust datetime library |

**GitHub Integration (octocrab):**
- Typed API for issues, PRs, projects
- Version 0.49.3 (actively maintained, v0.44 mentioned in docs)
- Async-first (works with tokio)
- Extensible for custom endpoints

**GitLab Integration (gitlab):**
- Tracks GitLab API v4 (version 18.7 as of 2025)
- Builder pattern for queries
- Updated 2 weeks ago (active maintenance)
- Returns caller-defined types (use serde Deserialize)

**Pomodoro Timer:**
- Don't add a "pomodoro crate" – just use `std::time` + `chrono`
- Surveyed: `pomodoro`, `pomodoro-tui`, `timr-tui`, `pomors`
- All are full applications, not libraries
- For Ferrite: 25min work timer = 20 lines of Rust code

**Task Storage:**
- Use existing `serde` + file I/O (JSON/TOML)
- No need for dedicated task management library
- Keep it simple: serialize tasks to `~/.ferrite/tasks.json`

---

## 4. Power Terminal Features

### SSH + Command History

| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| `russh` | 0.56.0 | SSH client/server | Pure Rust, async-first, actively maintained |
| `russh-keys` | 0.22.0 | SSH key management | Companion crate to russh |
| `reedline` | Latest | Command history/editing | Powers nushell, feature-rich |

**SSH: russh over ssh2**

**russh (RECOMMENDED):**
- Pure Rust (no libssh2 FFI)
- Async-first (tokio integration)
- Supports BOTH client AND server
- Version 0.56.0 (active development)
- Powers VS Code remote SSH
- No OpenSSL dependency

**ssh2 (NOT recommended):**
- FFI bindings to C libssh2
- Client-only (no server support)
- Requires OpenSSL
- Harder to integrate with async

**Command History: reedline over rustyline**

**reedline (RECOMMENDED):**
- Modern line editor (powers nushell v0.60+)
- Features: syntax highlighting, completions, multiline, Unicode
- Persistent history with multi-session support
- Configurable keybindings (emacs/vi)
- FileBackedHistory with size limits

**rustyline (alternative):**
- Lighter weight
- Based on linenoise
- Good for simple history
- Less feature-rich

**For Ferrite's power terminal:** reedline provides IDE-like terminal experience

---

## 5. Data Viewers (CSV/JSON)

### Built-in Rust + serde

| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| `csv` | 1.3+ | CSV parsing | BurntSushi's fast parser, serde support |
| `serde_json` | 1.0.145 | JSON parsing | Already recommended for AI features |

**CSV Viewer:**
- Use `csv` crate (by BurntSushi, same author as ripgrep)
- Fast, flexible, serde integration
- Handles invalid UTF-8 (ByteRecord)

**JSON Viewer:**
- Use `serde_json` (already in stack for AI)
- No additional dependency needed
- Pretty-printing built-in

**No need for:**
- Specialized viewer libraries
- egui has built-in table widgets (use egui::Grid or egui_extras::TableBuilder)

---

## 6. Async Integration with egui

### Critical Bridge: poll-promise

| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| `poll-promise` | 0.3+ | Async task bridge for egui | Recommended by egui maintainer |
| `tokio` | 1.49.0 | Async runtime | Single runtime for all async ops |

**Integration Pattern:**

```rust
// Spawn async task
let promise = poll_promise::Promise::spawn_async(async {
    // Call Claude API, SSH command, database query, etc.
    reqwest::get("...").await?.json().await
});

// In egui update loop
if let Some(result) = promise.ready() {
    ui.label(format!("Result: {:?}", result));
} else {
    ui.spinner(); // Show loading
}
```

**Alternative Considered: egui-async**
- `egui-async` (v0.6+): Higher-level wrapper around poll-promise
- Provides `Bind<T, E>` struct with state management
- Good for beginners, but poll-promise is more flexible

**Recommendation:** Start with `poll-promise` (lower-level, more control)

**Why NOT manual threading:**
- Harder to manage
- poll-promise provides progress tracking
- Cancellation support
- Works on both native and WASM

---

## 7. API Testing Tools (Optional)

### HTTP Client + Mocking

| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| `reqwest` | 0.12.26 | HTTP requests | Already in stack |
| `httpmock` | Latest | Mock HTTP for testing | Async support, JSON/regex matching |

**For API Testing Panel:**
- Use existing `reqwest` client
- No additional library needed for basic GET/POST/PUT/DELETE
- Store request history in JSON

**For Unit Tests:**
- `httpmock`: Full async HTTP mocking
- `wiremock`: Alternative, pairs well with reqwest
- `mockito`: Lightweight option

**Recommendation:** Don't add mocking to production binary. Use `reqwest` directly. Add `httpmock` as dev-dependency for testing Ferrite itself.

---

## Recommended Dependencies Summary

### Core Additions (Required)

```toml
[dependencies]
# Async runtime (foundation for all new features)
tokio = { version = "1.49", features = ["rt-multi-thread", "macros", "net", "io-util"] }
poll-promise = "0.3"

# HTTP/API (AI, GitHub, GitLab, API testing)
reqwest = { version = "0.12", features = ["json", "stream"] }
serde_json = "1.0"

# Database
rusqlite = { version = "0.38", features = ["bundled"] }
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres"] }

# Data viewers
csv = "1.3"

# Productivity
octocrab = "0.49"
gitlab = "0.18"
chrono = "0.4"

# Terminal power features
russh = "0.56"
russh-keys = "0.22"
reedline = "0.14"

# AI streaming
eventsource-client = "0.13"
```

### Optional/Feature-Gated

```toml
[dependencies]
# Only if implementing egui syntax highlighting
tree-sitter-highlight = "0.24"

# Dev dependencies for testing
[dev-dependencies]
httpmock = "0.8"
```

**Estimated Binary Size Impact:**
- Core additions: ~3-5 MB (tokio + reqwest + databases)
- SSH features: ~800 KB (russh)
- GitHub/GitLab: ~600 KB (octocrab + gitlab)
- Total impact: ~4-6 MB additional

**For comparison:** Current Ferrite with egui + portable-pty is likely 8-12 MB. New features add ~40-50% to binary size, but keep it under 20 MB total.

---

## NOT Recommended (Anti-Stack)

### Libraries to Avoid

| Library | Why NOT |
|---------|---------|
| `async-openai` | 50+ dependencies, OpenAI-focused, use reqwest instead |
| `diesel` | ORM bloat, system deps, sync-only, use sqlx/rusqlite |
| `ssh2` | FFI to C library, use pure Rust russh instead |
| `openai` (blocking) | Sync API, incompatible with async stack |
| Specialized AI SDKs | Lock-in, heavy deps, HTTP gives flexibility |
| Pomodoro apps as libs | Use std::time + chrono, don't cargo install apps |

### Integration Anti-Patterns

**DON'T:**
- Use `#[tokio::main]` (blocks main thread)
- Mix async runtimes (only tokio)
- Add ORM when raw SQL works
- Create separate thread pools per feature

**DO:**
- Spawn tokio runtime in background thread
- Use poll-promise for all async UI updates
- Share single tokio runtime across features
- Use raw SQL for transparency

---

## Integration Points with Existing Stack

### Current Ferrite Stack (v0.4)
- **UI:** egui/eframe (immediate-mode GUI)
- **Terminal:** portable-pty, vte (terminal emulation)
- **Serialization:** serde, serde_json (config)
- **Audio:** rodio (sound effects)
- **Git:** git2 (repository operations)
- **Regex:** regex (text processing)

### New Stack Compatibility

| Existing | New Feature | Integration |
|----------|-------------|-------------|
| egui | tokio async | poll-promise bridge |
| serde | API responses | Same serialization framework |
| portable-pty | russh | Separate (local vs remote terminals) |
| git2 | octocrab | git2 = local, octocrab = GitHub API |
| regex | SQL queries | Can use regex in SQLite via rusqlite |

**No conflicts identified.** New stack complements existing.

---

## Migration Path (Phased Adoption)

### Phase 1: Foundation
1. Add `tokio` + `poll-promise`
2. Add `reqwest` + `serde_json`
3. Test async integration with simple HTTP GET

### Phase 2: Database Tools
1. Add `rusqlite` (bundled, no system deps)
2. Add `csv` for data viewers
3. Build SQLite browser panel

### Phase 3: AI Integration
1. Add `eventsource-client` for SSE
2. Implement Claude API client with reqwest
3. Build chat panel with streaming

### Phase 4: Productivity
1. Add `octocrab` for GitHub
2. Add `gitlab` for GitLab
3. Implement Pomodoro with chrono

### Phase 5: Power Terminal
1. Add `russh` + `russh-keys`
2. Add `reedline` for history
3. Build SSH session manager

**Rationale:** This order minimizes risk. Foundation first, then layer features.

---

## Version Lock Recommendations

### Pin These (Breaking Changes Expected)
- `octocrab = "0.49"` (API changes between minor versions)
- `russh = "0.56"` (Still < 1.0, expect breaking changes)
- `sqlx = "0.8"` (Major versions have breaking changes)

### Allow Patch Updates
- `reqwest = "0.12"` (Stable API within 0.12.x)
- `tokio = "1.49"` (Stable 1.x LTS)
- `rusqlite = "0.38"` (Conservative maintenance)

### Always Latest Patch
- `serde_json = "1.0"` (Ultra-stable)
- `csv = "1.3"` (Stable API)

---

## Confidence Assessment

| Category | Confidence | Rationale |
|----------|------------|-----------|
| AI/LLM Stack | HIGH | reqwest + eventsource-client is battle-tested pattern |
| Database Stack | HIGH | rusqlite + sqlx are industry standard |
| Async Integration | HIGH | poll-promise recommended by egui maintainer |
| GitHub API | HIGH | octocrab actively maintained, version 0.49.3 current |
| GitLab API | MEDIUM | gitlab crate less popular than octocrab, but tracks API |
| SSH Stack | HIGH | russh powers VS Code, pure Rust, async-first |
| Command History | HIGH | reedline powers nushell, proven in production |

**Overall Confidence: HIGH**

All recommendations verified with:
- Official crates.io pages
- GitHub release pages (for russh, octocrab)
- Web search for 2026 current usage
- Cross-referenced multiple sources

---

## Sources

### HTTP/Async
- [reqwest on GitHub](https://github.com/seanmonstar/reqwest)
- [reqwest 0.12.26 on docs.rs](https://docs.rs/crate/reqwest/latest)
- [Tokio LTS releases](https://tokio.rs/)
- [tokio 1.49.0 on docs.rs](https://docs.rs/crate/tokio/latest)
- [poll-promise by Embark Studios](https://github.com/EmbarkStudios/poll-promise)
- [egui async integration discussion](https://github.com/emilk/egui/discussions/521)

### AI/LLM
- [async-openai on GitHub](https://github.com/64bit/async-openai)
- [ollama-rs on GitHub](https://github.com/pepperoni21/ollama-rs)
- [eventsource-client on GitHub](https://github.com/launchdarkly/rust-eventsource-client)
- [Server-Sent Events in Rust](https://dev.to/chaudharypraveen98/server-sent-events-in-rust-3lk0)

### Database
- [rusqlite vs diesel comparison](https://github.com/the-electric-computer-company/ele/wiki/rusqlite-vs-diesel)
- [rusqlite 0.38.0 on crates.io](https://crates.io/crates/rusqlite)
- [sqlx 0.8.6 on docs.rs](https://docs.rs/crate/sqlx/latest)
- [Rust database crate comparison 2023](https://rust-trends.com/posts/database-crates-diesel-sqlx-tokio-postgress/)
- [Diesel vs SQLx comparison](https://diesel.rs/compare_diesel.html)

### Productivity
- [octocrab on GitHub](https://github.com/XAMPPRocky/octocrab)
- [octocrab 0.49.3 on docs.rs](https://docs.rs/octocrab)
- [gitlab crate on docs.rs](https://docs.rs/gitlab/latest/gitlab/)
- [GitLab REST API third-party clients](https://docs.gitlab.com/api/rest/third_party_clients/)

### SSH/Terminal
- [russh on GitHub](https://github.com/Eugeny/russh)
- [russh vs ssh2 comparison](https://lib.rs/crates/russh)
- [reedline on GitHub](https://github.com/nushell/reedline)
- [rustyline vs reedline](https://users.rust-lang.org/t/finding-a-readline-library-with-timeout-and-full-duplex-mode/115913)

### Data Parsing
- [csv crate by BurntSushi](https://github.com/BurntSushi/rust-csv)
- [serde_json 1.0.145 on docs.rs](https://docs.rs/crate/serde_json/latest)

### Testing
- [httpmock on crates.io](https://crates.io/crates/httpmock)
- [How to mock reqwest clients](https://webscraping.ai/faq/reqwest/how-can-i-mock-reqwest-clients-for-testing-purposes)
- [Testing reqwest-based clients](https://www.lpalmieri.com/posts/how-to-write-a-rest-client-in-rust-with-reqwest-and-wiremock/)

---

## Next Steps

1. **Validation:** Test async integration pattern with simple reqwest GET in egui
2. **Prototype:** Build minimal SQLite browser with rusqlite
3. **Benchmark:** Measure binary size impact of tokio + reqwest
4. **Architecture:** Design panel system for AI chat, database browser, SSH sessions

**Research complete. Ready for roadmap creation.**
