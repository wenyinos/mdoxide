# Architecture Research: Ferrite v0.5.0 Integration

**Project:** Ferrite (Rust + egui markdown editor)
**Research Date:** 2026-01-24
**Confidence:** MEDIUM-HIGH

## Executive Summary

This research examines how to integrate AI assistance, database tools, and enhanced terminal features into Ferrite's existing egui immediate-mode architecture. Ferrite already has:

- **Immediate-mode UI:** egui/eframe with per-frame rendering
- **Panel system:** Existing pattern with `*Panel` structs and `*PanelOutput` return types
- **State management:** Settings struct with serde JSON persistence
- **Terminal foundation:** portable-pty + vte with custom layout tree
- **No async runtime:** Currently synchronous-only codebase

Key architectural challenge: **Integrating async operations (AI streaming, database queries, SSH connections) into synchronous egui immediate-mode UI**.

---

## 1. Modular Panels System

### Current Pattern Analysis

Ferrite uses a **consistent panel architecture**:

```rust
// Pattern observed in outline_panel.rs, file_tree.rs, terminal_panel.rs
pub struct SomePanel {
    width: f32,
    // panel-specific state
}

pub struct SomePanelOutput {
    pub action_requested: Option<Action>,
    pub close_requested: bool,
    pub new_width: Option<f32>,
}

impl SomePanel {
    pub fn show(&mut self, ctx: &egui::Context, ...) -> SomePanelOutput {
        // Render using egui::SidePanel or TopBottomPanel
    }
}
```

**Key characteristics:**
- State stored in panel struct (persistent across frames)
- Output struct for user actions (navigation, commands, etc.)
- `show()` method renders and returns output
- Panels use `egui::SidePanel` or `egui::TopBottomPanel`
- Resizable with min/max width constraints

### Implementing Toggleable Features

**Approach: Settings-driven visibility with AnimatedPanel**

```rust
// In Settings struct (already has serde)
pub struct Settings {
    // ... existing fields ...
    pub ai_panel_visible: bool,
    pub database_panel_visible: bool,
    pub ssh_panel_visible: bool,
}

// In App struct
pub struct App {
    settings: Settings,
    ai_panel: Option<AiPanel>,      // Lazy init
    db_panel: Option<DatabasePanel>,
    ssh_panel: Option<SshPanel>,
    // ...
}

// In App::update()
if self.settings.ai_panel_visible {
    if self.ai_panel.is_none() {
        self.ai_panel = Some(AiPanel::new());
    }
    egui::SidePanel::right("ai_panel")
        .show_animated(ctx, self.settings.ai_panel_visible, |ui| {
            let output = self.ai_panel.as_mut().unwrap().show(ui);
            // Handle output
        });
}
```

**egui provides:**
- `.show_animated()` - Smooth collapse/expand animations
- `.show_animated_inside()` - For conditional rendering
- Built-in resize handles and width persistence

**Integration points:**
1. Add visibility toggles to `Settings` struct
2. Add menu items in `View` menu (like existing "Terminal Panel")
3. Add keyboard shortcuts (like Ctrl+`)
4. Lazy-initialize panels when first shown
5. Persist visibility state in settings.json

---

## 2. AI Integration Architecture

### Challenge: Async Streaming in Synchronous egui

**egui is synchronous** - the update() function must not block. Async operations require background execution.

### Recommended Pattern: Channel-Based Updates

**Library:** Use `egui-async` or custom `mpsc::channel` with `ctx.request_repaint()`

```rust
use std::sync::mpsc;

pub struct AiPanel {
    // UI state
    prompt: String,
    response: String,

    // Async communication
    tx: mpsc::Sender<AiCommand>,
    rx: mpsc::Receiver<AiResponse>,
}

enum AiCommand {
    SendPrompt { prompt: String },
    CancelRequest,
}

enum AiResponse {
    StreamingChunk(String),
    Complete,
    Error(String),
}

impl AiPanel {
    pub fn new() -> Self {
        let (cmd_tx, cmd_rx) = mpsc::channel();
        let (resp_tx, resp_rx) = mpsc::channel();

        // Spawn background thread
        std::thread::spawn(move || {
            ai_worker_thread(cmd_rx, resp_tx);
        });

        Self {
            prompt: String::new(),
            response: String::new(),
            tx: cmd_tx,
            rx: resp_rx,
        }
    }

    pub fn show(&mut self, ui: &mut Ui) -> AiPanelOutput {
        // Poll for responses
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                AiResponse::StreamingChunk(chunk) => {
                    self.response.push_str(&chunk);
                    ui.ctx().request_repaint(); // Trigger immediate redraw
                }
                AiResponse::Complete => { /* Update state */ }
                AiResponse::Error(e) => { /* Show error */ }
            }
        }

        // Render UI
        ui.text_edit_multiline(&mut self.prompt);
        if ui.button("Send").clicked() {
            let _ = self.tx.send(AiCommand::SendPrompt {
                prompt: self.prompt.clone()
            });
        }
        ui.label(&self.response);

        AiPanelOutput::default()
    }
}

fn ai_worker_thread(cmd_rx: Receiver<AiCommand>, resp_tx: Sender<AiResponse>) {
    // Create tokio runtime in background thread
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async {
        while let Ok(cmd) = cmd_rx.recv() {
            match cmd {
                AiCommand::SendPrompt { prompt } => {
                    // Use Rig or OpenAI SDK for streaming
                    let mut stream = llm_client.stream_completion(&prompt).await;

                    while let Some(chunk) = stream.next().await {
                        let _ = resp_tx.send(AiResponse::StreamingChunk(chunk));
                    }
                    let _ = resp_tx.send(AiResponse::Complete);
                }
                _ => {}
            }
        }
    });
}
```

**Key architectural decisions:**

1. **Tokio runtime in background thread** (not main thread)
   - Main thread handles egui event loop
   - Background thread runs async runtime

2. **Channel-based communication**
   - `mpsc::channel` for commands (UI → worker)
   - `mpsc::channel` for responses (worker → UI)
   - `ctx.request_repaint()` triggers immediate UI update

3. **LLM SDK options:**
   - **Rig** (rig-core crate) - Rust-native, tokio-based, supports streaming
   - **async-openai** - OpenAI API client with streaming support
   - Both integrate well with tokio streams

4. **State management:**
   - Response text accumulated in panel state
   - Scrollable `TextEdit` or `ScrollArea` for display
   - Cancel button sends command to stop stream

**egui-specific considerations:**
- Use `TextEdit::multiline()` for prompt input
- Use `ScrollArea::vertical()` for response display
- Use `ui.ctx().request_repaint()` after each chunk
- Consider rate limiting (only repaint every N chunks for performance)

**Sources:**
- [egui-async on crates.io](https://crates.io/crates/egui-async)
- [Combining tokio and egui](https://actix.vdop.org/view_post?post_num=14)
- [Rig LLM Framework](https://rig.rs/)
- [Streaming LLM responses with Rust](https://www.trieve.ai/blog/open_ai_streaming)

---

## 3. Database Integration Architecture

### Connection Management

**Problem:** Database connections are async, egui is sync.

**Solution:** Connection pool in background thread with command/response channels.

```rust
use sqlx::PgPool;

pub struct DatabasePanel {
    query_text: String,
    results: Vec<Vec<String>>,  // Simple table representation
    columns: Vec<String>,

    // Async communication
    cmd_tx: mpsc::Sender<DbCommand>,
    resp_rx: mpsc::Receiver<DbResponse>,

    // Connection state
    connection_status: ConnectionStatus,
}

enum DbCommand {
    Connect { url: String },
    ExecuteQuery { sql: String },
    Disconnect,
}

enum DbResponse {
    Connected,
    QueryResult { columns: Vec<String>, rows: Vec<Vec<String>> },
    Error(String),
}

fn db_worker_thread(cmd_rx: Receiver<DbCommand>, resp_tx: Sender<DbResponse>) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async {
        let mut pool: Option<PgPool> = None;

        while let Ok(cmd) = cmd_rx.recv() {
            match cmd {
                DbCommand::Connect { url } => {
                    match PgPool::connect(&url).await {
                        Ok(p) => {
                            pool = Some(p);
                            let _ = resp_tx.send(DbResponse::Connected);
                        }
                        Err(e) => {
                            let _ = resp_tx.send(DbResponse::Error(e.to_string()));
                        }
                    }
                }
                DbCommand::ExecuteQuery { sql } => {
                    if let Some(ref p) = pool {
                        match sqlx::query(&sql).fetch_all(p).await {
                            Ok(rows) => {
                                // Convert sqlx::Row to Vec<Vec<String>>
                                let result = serialize_rows(rows);
                                let _ = resp_tx.send(DbResponse::QueryResult {
                                    columns: result.columns,
                                    rows: result.rows,
                                });
                            }
                            Err(e) => {
                                let _ = resp_tx.send(DbResponse::Error(e.to_string()));
                            }
                        }
                    }
                }
                DbCommand::Disconnect => {
                    pool = None;
                }
            }
        }
    });
}
```

**Connection pooling with SQLx:**

```rust
use sqlx::postgres::PgPoolOptions;

let pool = PgPoolOptions::new()
    .max_connections(5)
    .min_connections(1)
    .acquire_timeout(std::time::Duration::from_secs(10))
    .connect(&database_url)
    .await?;
```

**SQLx features:**
- Async connection pool (built-in)
- Automatic connection recycling
- Works with tokio runtime
- Supports PostgreSQL, MySQL, SQLite

### Query Result Display

**Use egui_table or custom table widget:**

```rust
// Option 1: Use egui_table crate
use egui_table::Table;

pub fn show_results(&self, ui: &mut Ui) {
    Table::new()
        .columns(self.columns.len())
        .show(ui, |ui| {
            // Header row
            for col in &self.columns {
                ui.heading(col);
            }
            ui.end_row();

            // Data rows
            for row in &self.results {
                for cell in row {
                    ui.label(cell);
                }
                ui.end_row();
            }
        });
}

// Option 2: Manual table with egui::Grid
egui::Grid::new("query_results")
    .striped(true)
    .show(ui, |ui| {
        // Same rendering logic
    });
```

**Table display libraries:**
- `egui-data-table` - Generic data table with sorting
- `egui_table` - Simple table widget
- `egui::Grid` (built-in) - Manual grid layout

**Integration points:**
1. Connection string stored in Settings (encrypted?)
2. Connection profiles (dev, staging, prod)
3. Query history (stored in Settings)
4. Export results to CSV (use `csv` crate already in Cargo.toml)

**Sources:**
- [SQLx Connection Pooling Guide (Jan 2026)](https://oneuptime.com/blog/post/2026-01-07-rust-database-connection-pooling/view)
- [egui-data-table on crates.io](https://crates.io/crates/egui-data-table)
- [SurrealDB + egui integration](https://surrealdb.com/docs/sdk/rust/frameworks/egui)

---

## 4. SSH/Remote Architecture

### Connection Management

**Library:** `russh` (formerly thrussh) - async SSH client

```rust
use russh::{client, Channel};

pub struct SshPanel {
    connections: HashMap<String, SshConnection>,
    active_connection: Option<String>,

    cmd_tx: mpsc::Sender<SshCommand>,
    resp_rx: mpsc::Receiver<SshResponse>,
}

struct SshConnection {
    host: String,
    status: ConnectionStatus,
    terminal_output: String,
}

enum SshCommand {
    Connect { host: String, user: String, key_path: PathBuf },
    ExecuteCommand { connection_id: String, command: String },
    Disconnect { connection_id: String },
}

enum SshResponse {
    Connected { connection_id: String },
    CommandOutput { connection_id: String, output: String },
    Error { connection_id: String, error: String },
}
```

**SSH client pattern with russh:**

```rust
async fn ssh_worker_thread(cmd_rx: Receiver<SshCommand>, resp_tx: Sender<SshResponse>) {
    use russh::*;
    use russh_keys::*;

    let config = client::Config::default();
    let mut clients: HashMap<String, client::Handle<MyHandler>> = HashMap::new();

    while let Ok(cmd) = cmd_rx.recv() {
        match cmd {
            SshCommand::Connect { host, user, key_path } => {
                let key = load_secret_key(key_path, None)?;
                let session = client::connect(config.clone(), (host.as_str(), 22), MyHandler).await?;

                let auth_res = session.authenticate_publickey(user, Arc::new(key)).await?;
                if auth_res {
                    clients.insert(host.clone(), session);
                    resp_tx.send(SshResponse::Connected { connection_id: host })?;
                }
            }
            SshCommand::ExecuteCommand { connection_id, command } => {
                if let Some(session) = clients.get(&connection_id) {
                    let mut channel = session.channel_open_session().await?;
                    channel.exec(true, command).await?;

                    let mut output = String::new();
                    while let Some(msg) = channel.wait().await {
                        match msg {
                            ChannelMsg::Data { ref data } => {
                                output.push_str(&String::from_utf8_lossy(data));
                            }
                            ChannelMsg::Eof => break,
                            _ => {}
                        }
                    }

                    resp_tx.send(SshResponse::CommandOutput {
                        connection_id,
                        output,
                    })?;
                }
            }
            _ => {}
        }
    }
}
```

**Channel multiplexing:**
- russh supports multiple channels per session
- Each command runs in a separate channel
- Connection pooling per host

**Integration with existing terminal:**
- Option 1: SSH connections open in new terminal tabs
- Option 2: Dedicated SSH panel with connection list
- Reuse existing `TerminalWidget` for display

**Sources:**
- [russh on GitHub](https://github.com/Eugeny/russh)
- [async-ssh2-tokio](https://docs.rs/async-ssh2-tokio/latest/async_ssh2_tokio/)
- [makiko SSH client](https://github.com/honzasp/makiko)

---

## 5. State Management & Persistence

### Current State Architecture

Ferrite uses:

```rust
// src/config/settings.rs
#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub theme: Theme,
    pub view_mode: ViewMode,
    pub font_size: f32,
    // ... ~50+ fields
    pub last_open_tabs: Vec<TabInfo>,
    pub recent_files: Vec<PathBuf>,
}

// Persistence: JSON file at ~/.config/ferrite/settings.json
pub fn save_settings(settings: &Settings) -> Result<()> {
    let config_path = config_dir().join("settings.json");
    let json = serde_json::to_string_pretty(settings)?;
    std::fs::write(config_path, json)?;
    Ok(())
}
```

**egui's persistence system (not currently used):**
- `egui::Memory` - Window positions, scroll offsets, collapse states
- Requires `persistence` feature in eframe
- Auto-saves on shutdown

### New State Requirements

**Panel visibility and layout:**

```rust
// Extend Settings struct
pub struct Settings {
    // ... existing fields ...

    // Panel visibility
    pub ai_panel_visible: bool,
    pub database_panel_visible: bool,
    pub ssh_panel_visible: bool,
    pub file_tree_visible: bool,
    pub outline_panel_visible: bool,

    // Panel widths (for resize persistence)
    pub ai_panel_width: f32,
    pub database_panel_width: f32,
    pub ssh_panel_width: f32,

    // Connection profiles
    pub database_connections: Vec<DatabaseProfile>,
    pub ssh_connections: Vec<SshProfile>,

    // AI settings
    pub ai_api_key: Option<String>,  // Consider encryption
    pub ai_model: String,
    pub ai_max_tokens: usize,
}

#[derive(Serialize, Deserialize)]
pub struct DatabaseProfile {
    pub name: String,
    pub connection_string: String,  // Encrypt sensitive data
    pub database_type: DatabaseType,
}

#[derive(Serialize, Deserialize)]
pub struct SshProfile {
    pub name: String,
    pub host: String,
    pub user: String,
    pub key_path: PathBuf,
}
```

**Workspace-specific state:**

```rust
// Per-workspace settings (optional)
#[derive(Serialize, Deserialize)]
pub struct WorkspaceState {
    pub workspace_path: PathBuf,
    pub open_files: Vec<PathBuf>,
    pub layout: TerminalLayout,  // Reuse existing layout enum
    pub database_profile: Option<String>,
    pub ssh_connections: Vec<String>,
}

// Saved to .ferrite/workspace.json in workspace root
```

**Sensitive data handling:**

```rust
use keyring::Entry;

// Store API keys in system keyring (not JSON)
pub fn store_api_key(service: &str, key: &str) -> Result<()> {
    let entry = Entry::new("ferrite", service)?;
    entry.set_password(key)?;
    Ok(())
}

pub fn load_api_key(service: &str) -> Result<String> {
    let entry = Entry::new("ferrite", service)?;
    Ok(entry.get_password()?)
}
```

**Session restoration:**

1. On startup: Load Settings from JSON
2. If workspace mode: Load WorkspaceState from .ferrite/workspace.json
3. Restore panel visibility and widths
4. Reconnect to databases/SSH (if configured)
5. Restore terminal layout and tabs

**Sources:**
- [egui Memory and Persistence](https://docs.rs/egui/latest/egui/struct.Memory.html)
- [egui persistence discussion](https://github.com/emilk/egui/issues/733)

---

## 6. Component Integration Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                          Main Window (eframe)                       │
├─────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  ┌─────────────┐  ┌──────────────────────────────┐  ┌────────────┐ │
│  │ FileTree    │  │    CentralPanel              │  │  Outline   │ │
│  │ Panel       │  │  ┌────────────────────────┐  │  │  Panel     │ │
│  │             │  │  │ Editor / Preview       │  │  │            │ │
│  │ - Files     │  │  │ (Existing)             │  │  │ - Headings │ │
│  │ - Git       │  │  └────────────────────────┘  │  │ - Stats    │ │
│  │   status    │  │                              │  │            │ │
│  └─────────────┘  └──────────────────────────────┘  └────────────┘ │
│                                                                       │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │           Terminal Panel (Existing)                             ││
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐                         ││
│  │  │ Term 1  │  │ Term 2  │  │ Term 3  │  [Split layouts]        ││
│  │  └─────────┘  └─────────┘  └─────────┘                         ││
│  └─────────────────────────────────────────────────────────────────┘│
│                                                                       │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐                     │
│  │ AI Panel   │  │ Database   │  │ SSH Panel  │  (New - toggleable) │
│  │            │  │ Panel      │  │            │                     │
│  │ - Prompt   │  │ - Connect  │  │ - Conns    │                     │
│  │ - Stream   │  │ - Query    │  │ - Execute  │                     │
│  │ - Response │  │ - Results  │  │ - Output   │                     │
│  └────────────┘  └────────────┘  └────────────┘                     │
│         ▲               ▲               ▲                            │
│         │               │               │                            │
│    ┌────┴───────────────┴───────────────┴─────┐                     │
│    │      Background Worker Threads            │                     │
│    │  ┌──────────┐ ┌──────────┐ ┌──────────┐  │                     │
│    │  │ AI       │ │ Database │ │ SSH      │  │                     │
│    │  │ Worker   │ │ Worker   │ │ Worker   │  │                     │
│    │  │ (tokio)  │ │ (tokio)  │ │ (tokio)  │  │                     │
│    │  └──────────┘ └──────────┘ └──────────┘  │                     │
│    └────────────────────────────────────────────┘                     │
│                         ▲                                             │
│                         │ mpsc::channel                               │
│                         ▼                                             │
│    ┌────────────────────────────────────────────┐                     │
│    │         Settings (JSON persistence)        │                     │
│    │  - Panel visibility                        │                     │
│    │  - Connection profiles                     │                     │
│    │  - Layout state                            │                     │
│    └────────────────────────────────────────────┘                     │
└───────────────────────────────────────────────────────────────────────┘
```

**Component boundaries:**

1. **UI Layer (Main thread)**
   - egui panels and widgets
   - Event handling
   - State polling via `try_recv()`
   - No blocking operations

2. **Worker Layer (Background threads)**
   - Tokio runtime per worker type
   - Async operations (AI, DB, SSH)
   - Send responses via `mpsc::channel`

3. **Persistence Layer**
   - Settings struct (JSON)
   - Workspace state (optional JSON)
   - Keyring for secrets

---

## 7. Build Order & Dependencies

### Phase 1: Foundation (Low risk, core infrastructure)

**Goal:** Establish async infrastructure without breaking existing code

1. **Add tokio to Cargo.toml**
   ```toml
   tokio = { version = "1", features = ["rt-multi-thread", "macros", "sync"] }
   ```
   - No code changes yet, just dependency

2. **Create worker thread infrastructure**
   - New module: `src/async_worker.rs`
   - Generic worker pattern with channels
   - Test with simple echo worker
   - **Rationale:** Foundation for all async features

3. **Extend Settings struct**
   - Add panel visibility flags
   - Add panel width fields
   - Add connection profile structs
   - **Rationale:** State management before UI

### Phase 2: First Feature - Database Panel (Medium risk)

**Goal:** Prove async pattern works with real feature

4. **Implement DatabasePanel UI**
   - New module: `src/ui/database_panel.rs`
   - Following existing panel pattern
   - Basic connection form + query input
   - **Rationale:** Simpler than AI streaming, tests channel pattern

5. **Add SQLx integration**
   ```toml
   sqlx = { version = "0.7", features = ["runtime-tokio", "postgres"] }
   ```
   - Database worker thread
   - Connection pooling
   - Query execution
   - **Rationale:** Tests tokio runtime integration

6. **Add result table display**
   ```toml
   egui-data-table = "0.1"
   ```
   - Result grid rendering
   - Export to CSV
   - **Rationale:** Tests data display patterns

### Phase 3: AI Panel (High complexity)

**Goal:** Add AI assistance with streaming

7. **Implement AI worker**
   ```toml
   rig-core = "0.1"  # or async-openai
   ```
   - LLM client setup
   - Streaming response handler
   - **Rationale:** Most complex async operation

8. **Implement AiPanel UI**
   - Prompt input
   - Streaming response display
   - Cancel/retry controls
   - **Rationale:** User-facing AI interface

9. **Add API key management**
   ```toml
   keyring = "2"
   ```
   - Secure storage
   - Settings UI for keys
   - **Rationale:** Security requirement

### Phase 4: SSH Integration (Medium risk)

**Goal:** Remote terminal connections

10. **Add russh integration**
    ```toml
    russh = "0.40"
    russh-keys = "0.40"
    ```
    - SSH worker thread
    - Connection management
    - **Rationale:** Async I/O pattern

11. **Implement SshPanel UI**
    - Connection profiles
    - Command execution
    - Output display
    - **Rationale:** User interface

12. **Integrate with terminal**
    - SSH sessions in terminal tabs
    - Or dedicated SSH output widget
    - **Rationale:** Reuse existing terminal infrastructure

### Phase 5: Polish & Integration (Low risk)

13. **Add menu items**
    - View > AI Panel
    - View > Database Panel
    - View > SSH Panel
    - **Rationale:** Discoverability

14. **Add keyboard shortcuts**
    - Register in shortcuts system
    - **Rationale:** Power user efficiency

15. **Session persistence**
    - Save/restore panel state
    - Workspace-specific settings
    - **Rationale:** User experience

---

## 8. Integration Points with Existing Components

### Terminal Panel Integration

**Current:** TerminalPanel with layout tree, tabs, floating windows

**New integration:**
1. SSH connections open in new terminal tabs
2. Database query results can spawn terminal for CSV export
3. AI can suggest terminal commands (copy to clipboard)

**Shared code:**
- `TerminalLayout` for split SSH sessions
- `TerminalWidget` for remote shell display
- Keyboard focus management

### Settings Panel Integration

**Current:** SettingsPanel with sections (Appearance, Editor, etc.)

**New sections:**
1. "AI Assistant" section
   - Model selection
   - API key (masked input)
   - Max tokens slider

2. "Database" section
   - Connection profiles list
   - Add/edit/delete profiles
   - Default connection

3. "SSH" section
   - SSH key paths
   - Known hosts
   - Connection profiles

**Shared code:**
- Settings struct serialization
- Reset to defaults logic

### File Tree Integration

**Current:** FileTreePanel with Git status

**New integration:**
1. AI context: "Explain this file"
2. Database: "Query this SQLite file"
3. Right-click menu extensions

---

## 9. Risk Assessment & Mitigation

### High Risk: Async Runtime on Main Thread

**Risk:** Blocking egui event loop

**Mitigation:**
- ✅ Run tokio in background threads only
- ✅ Use `try_recv()` (non-blocking) in UI
- ✅ Call `ctx.request_repaint()` after updates
- ⚠️ Test with high-frequency updates

**Validation:**
- Benchmark: 100 chunks/sec streaming
- Monitor frame time (should stay <16ms)

### Medium Risk: State Synchronization

**Risk:** Race conditions between UI and worker threads

**Mitigation:**
- ✅ Use `mpsc::channel` (thread-safe)
- ✅ UI owns display state, worker owns async state
- ✅ One-way communication (command/response)
- ⚠️ Avoid shared mutable state

**Validation:**
- Stress test with rapid connect/disconnect
- Test concurrent queries

### Medium Risk: Connection Lifecycle

**Risk:** Leaked connections, resource exhaustion

**Mitigation:**
- ✅ SQLx connection pooling (auto-cleanup)
- ✅ russh session management
- ✅ Disconnect on panel close
- ⚠️ Timeout handling

**Validation:**
- Test max connections limit
- Test reconnection after network failure

### Low Risk: Settings Migration

**Risk:** Breaking existing settings.json

**Mitigation:**
- ✅ Add `#[serde(default)]` to new fields
- ✅ Settings version field
- ✅ Migration function
- ✅ Backup old settings

**Validation:**
- Test with v0.2.5 settings.json
- Test with missing fields
- Test with corrupted JSON

---

## 10. Recommended Crates

### Core Async Infrastructure
- **tokio** `1.x` - Async runtime (multi-threaded)
- **mpsc** (std::sync) - Channel communication (already in std)

### AI Integration
- **rig-core** `0.1.x` - LLM framework (HIGH confidence)
  - Streaming support
  - Multiple providers
  - Tokio native
- **Alternative:** async-openai `0.19.x` - OpenAI-specific client

### Database
- **sqlx** `0.7.x` - Async SQL toolkit (HIGH confidence)
  - Connection pooling
  - PostgreSQL, MySQL, SQLite
  - Tokio runtime
- **Alternative:** tokio-postgres `0.7.x` - PostgreSQL only

### SSH
- **russh** `0.40.x` - Pure Rust SSH (MEDIUM confidence)
  - Async/tokio
  - Client and server
  - Active development
- **Alternative:** async-ssh2-tokio `0.8.x` - Higher-level API

### UI Tables
- **egui-data-table** `0.1.x` - Generic table widget (MEDIUM confidence)
- **egui_table** `0.1.x` - Simple table (LOW confidence - less mature)
- **Built-in:** egui::Grid - Manual but flexible

### Security
- **keyring** `2.x` - System keyring integration (HIGH confidence)
  - Windows Credential Manager
  - macOS Keychain
  - Linux Secret Service

---

## 11. Architecture Decision Records

### ADR-1: Background Threads for Async Operations

**Decision:** Run tokio runtime in background threads, not main thread

**Rationale:**
- egui requires responsive main thread
- OS window events must be handled quickly
- Blocking causes UI freeze

**Alternatives considered:**
- Async egui (rejected: not production-ready)
- Poll futures manually (rejected: complex, error-prone)

**Consequences:**
- ✅ UI stays responsive
- ✅ Standard Rust async patterns
- ⚠️ Thread communication overhead (minimal)

### ADR-2: Settings-Based Panel Visibility

**Decision:** Store panel visibility in Settings struct, not egui::Memory

**Rationale:**
- Consistent with existing Ferrite architecture
- Works across sessions
- User control via settings UI

**Alternatives considered:**
- egui persistence feature (rejected: different pattern than Ferrite uses)
- Compile-time features (rejected: not runtime toggleable)

**Consequences:**
- ✅ Consistent with existing code
- ✅ Easy to reset defaults
- ⚠️ Settings.json grows larger

### ADR-3: Lazy Panel Initialization

**Decision:** Only create panel instances when first shown

**Rationale:**
- Reduce startup time
- Avoid spawning unused worker threads
- Lower memory footprint

**Alternatives considered:**
- Always create all panels (rejected: wasteful)
- Singleton panels (rejected: harder to clean up)

**Consequences:**
- ✅ Faster startup
- ✅ Lower baseline resource usage
- ⚠️ First-show latency (acceptable)

### ADR-4: Channel-Based Communication

**Decision:** Use std::sync::mpsc for UI ↔ worker communication

**Rationale:**
- No external dependencies (std library)
- Simple, well-understood pattern
- Works with both sync and async code

**Alternatives considered:**
- crossbeam channels (rejected: extra dependency)
- tokio channels (rejected: requires tokio on UI side)
- Shared state with mutexes (rejected: more complex)

**Consequences:**
- ✅ Simple mental model
- ✅ Type-safe messages
- ⚠️ Bounded vs unbounded decision per panel

---

## 12. Open Questions & Future Research

### Question 1: AI Context Management

**Issue:** How much editor context to send to AI?

**Options:**
1. Current file only
2. All open tabs
3. Workspace-aware (git diff, related files)

**Needs research:**
- Token limits vs context size
- Privacy considerations (don't send secrets)
- Performance impact

**Recommended:** Phase-specific research in AI implementation phase

### Question 2: Database Query Caching

**Issue:** Should we cache query results?

**Options:**
1. No caching (always fresh)
2. LRU cache with TTL
3. User-controlled refresh

**Needs research:**
- Memory limits
- Cache invalidation strategy
- User expectations

**Recommended:** Start without caching, add if needed

### Question 3: Multi-Workspace Support

**Issue:** Can user have multiple workspaces open?

**Options:**
1. Single workspace per window
2. Multiple workspaces in tabs
3. Multiple windows (egui viewports)

**Needs research:**
- Worker thread management per workspace
- Settings isolation
- Resource limits

**Recommended:** Defer to post-v0.5.0

---

## 13. Success Metrics

### Performance Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| UI frame time | <16ms | During AI streaming at 50 chunks/sec |
| Panel toggle animation | 250ms | Smooth 60fps animation |
| Database query latency | <500ms overhead | Excluding actual query time |
| SSH connection time | <2s | From click to connected |
| Settings save time | <100ms | On shutdown |

### Integration Quality

| Aspect | Requirement |
|--------|-------------|
| No regressions | Existing features work unchanged |
| Memory usage | <50MB increase with all panels open |
| Startup time | <200ms increase with lazy init |
| Binary size | <5MB increase |

### User Experience

| Feature | Quality Bar |
|---------|-------------|
| Panel visibility | Persists across restarts |
| AI streaming | Real-time updates (no lag) |
| Database results | Scrollable, exportable |
| SSH sessions | Reconnect on disconnect |
| Error handling | User-friendly messages |

---

## Sources

**egui Architecture:**
- [egui SidePanel documentation](https://docs.rs/egui/latest/egui/containers/panel/struct.SidePanel.html)
- [egui Memory and Persistence](https://docs.rs/egui/latest/egui/struct.Memory.html)
- [Rust egui Step-by-Step Tutorial](https://hackmd.io/@Hamze/Sys9nvF6Jl)

**Async Integration:**
- [egui-async on crates.io](https://crates.io/crates/egui-async)
- [Combining tokio and egui](https://actix.vdop.org/view_post?post_num=14)
- [egui with tokio/async discussion](https://users.rust-lang.org/t/how-to-combine-egui-with-tokio-async-code/82500)

**AI/LLM Integration:**
- [Rig LLM Framework](https://rig.rs/)
- [Streaming LLM responses with Rust](https://www.trieve.ai/blog/open_ai_streaming)
- [Building AI Agents in Rust](https://refreshagent.com/engineering/building-ai-agents-in-rust)

**Database Integration:**
- [SQLx Connection Pooling (Jan 2026)](https://oneuptime.com/blog/post/2026-01-07-rust-database-connection-pooling/view)
- [SQLx documentation](https://docs.rs/sqlx/latest/sqlx/)
- [SurrealDB + egui integration](https://surrealdb.com/docs/sdk/rust/frameworks/egui)
- [egui-data-table on crates.io](https://crates.io/crates/egui-data-table)

**SSH Integration:**
- [russh on GitHub](https://github.com/Eugeny/russh)
- [async-ssh2-tokio documentation](https://docs.rs/async-ssh2-tokio/latest/async_ssh2_tokio/)
- [makiko SSH client](https://github.com/honzasp/makiko)

**UI Patterns:**
- [egui collapsible panel tutorial](https://whoisryosuke.com/blog/2023/getting-started-with-egui-in-rust)
- [Sidebar menu design patterns (2025)](https://www.navbar.gallery/blog/best-side-bar-navigation-menu-design-examples)

**State Management:**
- [egui persistence discussion](https://github.com/emilk/egui/issues/733)
- [egui UI State management discussion](https://github.com/emilk/egui/discussions/7553)
