# Pitfalls Research: Ferrite v0.5.0 "Swiss Army Knife"

**Domain:** Rust+egui productivity hub expansion (AI integration, database tools, SSH, productivity panels)
**Researched:** 2026-01-24
**Overall confidence:** HIGH (based on egui ecosystem research, Rust async patterns, verified community issues)

## Executive Summary

Adding AI integration, database browsing, SSH sessions, and productivity features to an existing egui Rust application presents **integration pitfalls** that differ from greenfield development. The core risks are:

1. **UI Thread Blocking** - egui's immediate mode makes async operations treacherous
2. **Binary Size Bloat** - New dependencies can balloon the lightweight binary
3. **State Management Complexity** - Immediate mode + async state is error-prone
4. **Breaking Existing Features** - Terminal emulation already works; new features risk regressions
5. **Performance Degradation** - Memory leaks in egui widgets with large state

---

## AI/LLM Integration Pitfalls

### CRITICAL: UI Thread Blocking During LLM Streaming

**What goes wrong:**
- Calling `.await` in egui's `update()` method freezes the entire UI
- LLM streaming responses take 5-30 seconds - UI becomes unresponsive
- Users perceive the app as "crashed" or "frozen"

**Why it happens:**
egui's immediate mode redraws at 60fps. Any blocking operation in the UI loop stops all rendering.

**Prevention:**
```rust
// WRONG: Blocks UI thread
fn update(&mut self, ctx: &egui::Context) {
    if ui.button("Ask AI").clicked() {
        let response = llm_client.query("...").await; // ❌ FREEZES UI
    }
}

// RIGHT: Use channels + background task
fn update(&mut self, ctx: &egui::Context) {
    if ui.button("Ask AI").clicked() {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let response = block_on(llm_client.query("..."));
            tx.send(response).ok();
        });
        self.ai_response_rx = Some(rx);
    }

    // Poll non-blocking
    if let Some(rx) = &self.ai_response_rx {
        if let Ok(response) = rx.try_recv() {
            self.ai_response = Some(response);
            self.ai_response_rx = None;
        }
    }
}
```

**Detection:**
- App freezes when AI query starts
- Frame rate drops to 0
- UI doesn't respond to clicks/keyboard

**Sources:**
- [Combining tokio and egui - The Iron Code](https://actix.vdop.org/view_post?post_num=14) - Documents core challenge
- [egui_async crate](https://docs.rs/egui-async/latest/egui_async/) - Solution library for Bind<T, E> pattern
- [egui-tokio integration discussion](https://github.com/emilk/egui/discussions/521) - Community patterns

### CRITICAL: LLM API Error Handling Without Timeouts

**What goes wrong:**
- API calls hang indefinitely if provider is down
- No retry logic means temporary failures appear permanent
- App becomes unusable until restart

**Prevention:**
- Enforce 30-60 second timeout on all LLM requests
- Implement exponential backoff retry (3 attempts)
- Show retry count in UI
- Allow user to cancel long-running requests

**Implementation:**
```rust
use std::time::Duration;

// Conditional retry with timeout
async fn query_with_retry(prompt: &str) -> Result<String, LlmError> {
    let mut attempts = 0;
    let mut interval = Duration::from_secs(1);

    loop {
        match tokio::time::timeout(
            Duration::from_secs(30),
            llm_api.query(prompt)
        ).await {
            Ok(Ok(response)) => return Ok(response),
            Ok(Err(e)) if attempts < 3 => {
                attempts += 1;
                tokio::time::sleep(interval).await;
                interval *= 2; // Exponential backoff
            }
            _ => return Err(LlmError::Timeout),
        }
    }
}
```

**Sources:**
- [Rust retry exponential backoff implementation](https://oneuptime.com/blog/post/2026-01-07-rust-retry-exponential-backoff/view) - Recent 2026 guide
- [Request Timeouts - Portkey Docs](https://portkey.ai/docs/product/ai-gateway/request-timeouts) - Recommends 30s minimum
- [Implementing Retry Mechanisms for LLM Calls](https://apxml.com/courses/prompt-engineering-llm-application-development/chapter-7-output-parsing-validation-reliability/implementing-retry-mechanisms)

### HIGH: Streaming Response State Management

**What goes wrong:**
- Partial responses stored in widget state cause memory leaks
- egui widgets clone state every frame - 10KB/frame * 60fps = 600KB/s leak
- Long streaming sessions fill memory

**Prevention:**
- Use `Arc<Mutex<String>>` for streaming buffers, not direct String in widget state
- Clear accumulated response after rendering
- Implement maximum response length (truncate after 100KB)

**Sources:**
- [egui Memory and Persistence](https://deepwiki.com/foxxcn/eguizh/3.4-memory-and-persistence) - Widget state cloning behavior
- [200 MB/s wgpu memory leak](https://github.com/emilk/egui/issues/4674) - Severe memory leak with wgpu integration

---

## Database Integration Pitfalls

### CRITICAL: Blocking Queries Freeze UI

**What goes wrong:**
- Query takes 5 seconds on large table
- UI freezes during `SELECT * FROM large_table`
- User can't cancel the query

**Prevention:**
```rust
// Use async database drivers + background thread
// SQLx or tokio-postgres for async queries

// Spawn query on background thread
let (tx, rx) = mpsc::channel();
thread::spawn(move || {
    let result = block_on(async {
        sqlx::query("SELECT * FROM table LIMIT 1000")
            .fetch_all(&pool)
            .await
    });
    tx.send(result).ok();
});

// Poll in UI update loop
if let Ok(rows) = self.query_rx.try_recv() {
    self.table_data = rows;
}
```

**Sources:**
- [Streaming select query result discussion](https://github.com/sfackler/rust-postgres/issues/155) - Need for streaming large results
- [Database concurrency discussion](https://users.rust-lang.org/t/database-concurrency/67485) - Patterns for async queries

### HIGH: Connection Pool Exhaustion

**What goes wrong:**
- Creating a new connection per query (connection leak)
- Pool runs out of connections after 10 queries
- Database refuses new connections

**Prevention:**
- Use `deadpool-postgres` or `r2d2` for connection pooling
- Initialize pool once at startup, reuse connections
- Set pool size based on expected concurrency (5-10 for single-user app)
- Close connections explicitly on panel close

**Configuration:**
```rust
use deadpool_postgres::{Config, Runtime};

let mut cfg = Config::new();
cfg.dbname = Some("ferrite".to_string());
cfg.pool = Some(PoolConfig::new(10)); // Max 10 connections

let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;
```

**Sources:**
- [How to Handle Database Connection Pooling in Rust](https://oneuptime.com/blog/post/2026-01-07-rust-database-connection-pooling/view) - January 2026 guide
- [r2d2 connection pool](https://github.com/sfackler/r2d2) - Generic connection pooling

### HIGH: Large Result Sets Crash App

**What goes wrong:**
- Fetching 1,000,000 rows into memory
- App memory usage spikes to 2GB
- OOM crash or severe UI lag

**Prevention:**
- **Always use LIMIT** - Default to 1,000 rows, paginate for more
- Stream results instead of loading all at once
- Show row count warning before executing large queries
- Implement virtual scrolling for large tables (don't render all rows)

**Implementation:**
```rust
// Warn before large query
if estimated_rows > 10_000 {
    ui.label("⚠️ This query may return >10K rows. Add LIMIT?");
    if ui.button("Add LIMIT 1000").clicked() {
        query += " LIMIT 1000";
    }
}

// Paginated loading
async fn fetch_page(pool: &Pool, offset: i64, limit: i64) -> Result<Vec<Row>> {
    sqlx::query("SELECT * FROM table LIMIT ? OFFSET ?")
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
}
```

**Sources:**
- [PostgreSQL performance discussion](https://users.rust-lang.org/t/postgresql-performance/51218) - Performance issues with large datasets

---

## SSH Session Management Pitfalls

### CRITICAL: SSH Session Cleanup Leaks

**What goes wrong:**
- `ssh2-rs` Drop implementation can fail silently in non-blocking mode
- Sessions remain open after disconnect
- Resources not released until app restart

**Why it happens:**
In non-blocking mode, `libssh2_session_free` returns `LIBSSH2_ERROR_EAGAIN` and needs repeated calls. If you don't repeatedly call it until it returns 0, resources leak.

**Prevention:**
```rust
impl Drop for SshSession {
    fn drop(&mut self) {
        // Force blocking cleanup
        self.session.set_blocking(true);

        // Explicitly disconnect
        let _ = self.session.disconnect(None, "Closing", None);

        // libssh2_session_free is called by Drop,
        // but setting blocking ensures it completes
    }
}
```

**Sources:**
- [Leaking LIBSSH2_SESSION objects](https://github.com/alexcrichton/ssh2-rs/issues/220) - Documented memory leak issue
- [Session cleanup documentation](https://docs.rs/ssh2/latest/ssh2/struct.Session.html) - Drop behavior

### HIGH: Concurrent SSH Operations Deadlock

**What goes wrong:**
- Trying to read from multiple SSH channels simultaneously
- All operations block because they share the same session
- UI freezes waiting for I/O

**Why it happens:**
In ssh2-rs, blocking reads from a Channel block **all other calls** on objects from the same underlying Session.

**Prevention:**
- Create separate Session instances for concurrent operations
- Use non-blocking mode with careful error handling
- Or spawn one thread per SSH channel

**Sources:**
- [Session concurrency documentation](https://docs.rs/ssh2/latest/ssh2/struct.Session.html) - Warns about blocking behavior
- [Running multiple SSH sessions with russh](https://users.rust-lang.org/t/running-multiple-ssh-client-sessions-using-russh/123513) - Recent discussion

### MEDIUM: SSH Timeout Configuration

**What goes wrong:**
- SSH session stays open indefinitely when network disconnects
- No way to detect dead connections
- Accumulating zombie sessions

**Prevention:**
```rust
// Configure keepalive
session.set_keepalive(true, 60); // Send keepalive every 60 seconds
session.set_timeout(30_000); // 30 second timeout for operations

// Explicitly check if session is still alive
if !session.authenticated() {
    return Err("SSH session disconnected");
}
```

**Sources:**
- [How to prevent SSH session timeouts](https://www.simplified.guide/ssh/disable-timeout) - ServerAliveInterval configuration

---

## Feature Bloat & Binary Size Pitfalls

### CRITICAL: Dependency Bloat

**What goes wrong:**
- Adding `tokio`, `sqlx`, `ssh2`, `rodio`, `reqwest` bloats binary from 15MB to 50MB
- Users complain about download size
- Antivirus flags large binary

**Current situation:**
- Ferrite is 15-20MB (release build with mimalloc/jemalloc)
- Each major dependency adds 5-10MB

**Prevention:**
1. **Feature-gate everything:**
```toml
[features]
default = ["terminal"]
ai = ["reqwest", "tokio"]
database = ["sqlx", "deadpool"]
ssh = ["ssh2"]

[dependencies]
reqwest = { version = "0.11", optional = true }
sqlx = { version = "0.7", optional = true, default-features = false }
```

2. **Disable default features:**
```toml
sqlx = { version = "0.7", default-features = false, features = ["runtime-tokio-native-tls", "sqlite"] }
```

3. **Use cargo-bloat to monitor:**
```bash
cargo install cargo-bloat
cargo bloat --release
```

**Sources:**
- [min-sized-rust guide](https://github.com/johnthagen/min-sized-rust) - Comprehensive binary size optimization
- [Optimize Rust binaries size](https://oknozor.github.io/blog/optimize-rust-binary-size/) - Feature flags approach
- [Binary Size Optimization - Rust Project Primer](https://rustprojectprimer.com/building/size.html)

### HIGH: egui Backend Choice

**What goes wrong:**
- Switching from `glow` to `wgpu` backend adds 10MB+ to binary
- wgpu's shader transpiler (naga) is "a chonky beast"

**Current situation:**
Ferrite likely uses `glow` (lightweight OpenGL backend)

**Prevention:**
- Stay with `glow` backend for v0.5.0
- Only switch to `wgpu` if absolutely necessary for features
- Track WASM size if adding web support

**Sources:**
- [Switch to wgpu as default backend discussion](https://github.com/emilk/egui/issues/5889) - Binary size concerns
- [Track size of wasm builds](https://github.com/emilk/egui/issues/5828) - Size monitoring proposal

### MEDIUM: Over-Bundling Assets

**What goes wrong:**
- Bundling 10 AI models into binary
- Including entire icon sets
- Embedding large fonts

**Prevention:**
- Load AI models from disk (user downloads what they need)
- Use system fonts where possible
- Bundle only essential icons, load rest on demand

---

## egui-Specific Pitfalls

### CRITICAL: Immediate Mode State Confusion

**What goes wrong:**
- Storing AI chat history in widget state
- State gets cloned every frame (60 times/second)
- 1MB chat log * 60fps = 60MB/s memory churn

**Why it happens:**
egui's `Memory::data` field clones values on each read. Large state should be wrapped in `Arc<Mutex<...>>`.

**Prevention:**
```rust
// WRONG: State cloned every frame
struct AiPanel {
    chat_history: Vec<Message>, // ❌ Cloned 60 times/second
}

// RIGHT: Arc prevents cloning
struct AiPanel {
    chat_history: Arc<Mutex<Vec<Message>>>, // ✅ Only pointer cloned
}

impl AiPanel {
    fn ui(&mut self, ui: &mut egui::Ui) {
        let history = self.chat_history.lock().unwrap();
        for msg in history.iter() {
            ui.label(&msg.text);
        }
    }
}
```

**Sources:**
- [egui Memory documentation](https://docs.rs/egui/latest/egui/struct.Memory.html) - State cloning behavior
- [Performance: Reduce heap allocations](https://github.com/emilk/egui/discussions/388) - Optimization discussion
- [Immediate mode design patterns](https://users.rust-lang.org/t/immediate-mode-design-patterns/106833) - Best practices

### HIGH: Async State Updates Without Request Repaint

**What goes wrong:**
- Background thread receives LLM response
- Updates shared state
- UI doesn't re-render until user moves mouse

**Prevention:**
```rust
// In background thread/task
let ctx = ctx.clone();
thread::spawn(move || {
    let response = fetch_ai_response();
    *state.lock().unwrap() = response;
    ctx.request_repaint(); // ✅ Force UI update
});
```

**Sources:**
- [egui Context API](https://docs.rs/egui/latest/egui/) - request_repaint() method

### HIGH: Dynamic Layout Limitations

**What goes wrong:**
- Trying to center two widgets side-by-side
- Responsive layouts that depend on final size
- egui calculates everything in single pass

**Why it happens:**
egui's immediate mode means layout is calculated once per frame. Layouts requiring multiple passes (measure then position) don't work well.

**Prevention:**
- Use `Grid` for structured layouts
- Accept egui's layout constraints
- Don't fight the immediate mode model

**Sources:**
- [egui layout limitations discussion](https://lobste.rs/s/4wnemk/egui_experimental_immediate_mode_gui) - "doesn't support putting two widgets in center"

---

## Integration Pitfalls (Breaking Existing Features)

### CRITICAL: Keyboard Shortcut Conflicts

**What goes wrong:**
- AI panel uses Ctrl+1 for "First suggestion"
- Terminal already uses Ctrl+1 for "Switch to terminal 1"
- Context-aware shortcuts break

**Current situation:**
Ferrite has **context-aware shortcuts** - Ctrl+1-9 switch terminals when terminal focused, switch files when editor focused.

**Prevention:**
- Maintain context-aware routing
- AI panel shortcuts only active when AI panel focused
- Document all shortcuts in settings
- Allow user customization of bindings

**Implementation:**
```rust
// In app.rs
match focused_panel {
    Panel::Terminal => handle_terminal_shortcuts(key),
    Panel::Editor => handle_editor_shortcuts(key),
    Panel::Ai => handle_ai_shortcuts(key),
}
```

### HIGH: Event Propagation Chaos

**What goes wrong:**
- Terminal panel consumes all keyboard events
- AI panel never receives input
- Copy/paste breaks in new panels

**Current situation:**
Terminal already has complex event handling (see `terminal/mod.rs` lines 248-295)

**Prevention:**
- Centralize event routing in `app.rs`
- Use `response.has_focus()` to determine active panel
- Test all keyboard shortcuts in all panels

**Sources:**
- [Terminal keyboard focus fix commit](https://github.com/OlaProeis/Ferrite/commit/a091999) - Recent event propagation fix

### HIGH: State Serialization Breaks

**What goes wrong:**
- Adding new fields to `SavedWorkspace` struct
- Old workspace files fail to load
- Users lose saved layouts

**Prevention:**
```rust
// Use #[serde(default)] for new fields
#[derive(Serialize, Deserialize)]
struct SavedWorkspace {
    pub tabs: Vec<SavedLayout>,

    #[serde(default)] // ✅ Won't break old files
    pub ai_sessions: Vec<AiSession>,
}
```

### MEDIUM: Performance Regression

**What goes wrong:**
- Adding database panel slows terminal rendering
- Polling 5 panels * 60fps = 300 updates/second
- Frame time increases from 2ms to 20ms

**Prevention:**
- Only poll active panels
- Use `ctx.request_repaint_after()` for lazy updates
- Profile with `cargo flamegraph`

```rust
// Only update visible panels
if self.ai_panel_visible {
    self.ai_panel.update(ctx);
}

// Lazy updates for background panels
ctx.request_repaint_after(Duration::from_millis(100));
```

---

## Prevention Checklist

### Before Adding Each Feature

- [ ] Will this block the UI thread? (If yes, use channels/background threads)
- [ ] How much will this add to binary size? (Check with cargo-bloat)
- [ ] Does this need async? (If yes, use tokio on separate thread)
- [ ] Will this break existing shortcuts? (Document conflicts)
- [ ] Does this leak memory with egui state cloning? (Use Arc<Mutex<>>)

### During Implementation

- [ ] Add timeouts to all network/LLM/database calls (30s minimum)
- [ ] Implement retry logic with exponential backoff
- [ ] Use connection pooling for databases
- [ ] Feature-gate large dependencies
- [ ] Test with existing terminal features (ensure no regression)

### Before Commit

- [ ] Run `cargo bloat --release` (check binary size delta)
- [ ] Test all keyboard shortcuts in all panels
- [ ] Profile with `cargo build --release && ./ferrite` (check frame time)
- [ ] Test loading old workspace files (backward compatibility)
- [ ] Verify no memory leaks (run for 5 minutes, check memory usage)

---

## Phase-Specific Warnings

| Phase | Primary Pitfall | Mitigation Strategy |
|-------|-----------------|---------------------|
| **AI Integration** | UI thread blocking | Use egui-async or channel pattern |
| **Database Browser** | Large result sets | Always LIMIT, paginate, virtual scroll |
| **SSH Sessions** | Connection leaks | Explicit cleanup in Drop, keepalive |
| **Productivity Panels** | Binary bloat | Feature flags, cargo-bloat monitoring |
| **All Phases** | Breaking terminals | Integration tests for existing features |

---

## Research Gaps

**LOW CONFIDENCE AREAS:**
- Specific Ollama integration patterns with egui (limited recent examples)
- Best practices for SQLite vs PostgreSQL in desktop apps (preference unclear)
- Remote SSH port forwarding stability (limited Rust examples)

**MEDIUM CONFIDENCE AREAS:**
- AI model loading strategies (disk vs embedded) - need user research
- Optimal connection pool sizes for single-user app (need testing)
- egui performance with 10+ panels open (need profiling)

**HIGH CONFIDENCE AREAS:**
- UI thread blocking patterns (verified with egui-async, community discussions)
- SSH session cleanup issues (documented in ssh2-rs issues)
- Binary size optimization techniques (comprehensive guides available)

---

## Sources Summary

**Primary Sources (HIGH confidence):**
- [egui-async crate](https://docs.rs/egui-async/latest/egui_async/) - Async integration patterns
- [egui discussions on tokio integration](https://github.com/emilk/egui/discussions/521) - Community patterns
- [Combining tokio and egui](https://actix.vdop.org/view_post?post_num=14) - Detailed guide
- [Rust retry exponential backoff (2026)](https://oneuptime.com/blog/post/2026-01-07-rust-retry-exponential-backoff/view)
- [Database connection pooling in Rust (2026)](https://oneuptime.com/blog/post/2026-01-07-rust-database-connection-pooling/view)
- [ssh2-rs session leak issue](https://github.com/alexcrichton/ssh2-rs/issues/220)
- [min-sized-rust guide](https://github.com/johnthagen/min-sized-rust)

**Secondary Sources (MEDIUM confidence):**
- [egui memory leaks discussion](https://github.com/emilk/egui/issues/4674)
- [Request Timeouts - Portkey](https://portkey.ai/docs/product/ai-gateway/request-timeouts)
- [PostgreSQL streaming results](https://github.com/sfackler/rust-postgres/issues/155)

**Ferrite-Specific Context:**
- Existing codebase: `Cargo.toml`, `src/terminal/mod.rs`, `ROADMAP.md`
- Validated architecture: Modular Panels, single binary, feature flags
- Current binary size: ~15-20MB (with terminal emulation)
