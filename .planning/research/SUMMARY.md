# Research Summary: Ferrite v0.5.0 "Swiss Army Knife"

**Synthesized:** 2026-01-24
**Milestone:** v0.5.0 - Developer Productivity Hub
**Overall Confidence:** HIGH

---

## Executive Summary

Research across 4 dimensions (Stack, Features, Architecture, Pitfalls) reveals a clear path to expand Ferrite into a modular developer productivity hub. The key insight: **async integration with egui is the critical foundation** - all new features (AI, databases, SSH) require background threads with channel-based communication.

### Key Findings

| Dimension | Critical Finding |
|-----------|-----------------|
| **Stack** | Use tokio + poll-promise + reqwest (not heavy SDKs). Single async runtime shared across features. |
| **Features** | Table stakes are minimal (task checkboxes, pomodoro, SQL editor, SSH). Differentiation comes from cross-feature integration. |
| **Architecture** | Channel-based UI updates: background threads run tokio, send results via mpsc, UI polls with try_recv(). |
| **Pitfalls** | UI thread blocking is the #1 risk. Use Arc<Mutex<>> for large state, feature-gate dependencies, explicit SSH cleanup. |

---

## Recommended Stack Additions

### Core Infrastructure (Required)
```toml
tokio = { version = "1.49", features = ["rt-multi-thread", "macros"] }
poll-promise = "0.3"
reqwest = { version = "0.12", features = ["json", "stream"] }
```

### Per-Feature (Feature-Gated)
| Feature | Crates | Binary Impact |
|---------|--------|---------------|
| AI Integration | eventsource-client, (reqwest already included) | ~1 MB |
| Database | rusqlite (bundled), sqlx (postgres) | ~2 MB |
| Productivity | octocrab, gitlab, chrono | ~600 KB |
| SSH | russh, russh-keys, reedline | ~800 KB |

**Total Impact:** ~4-6 MB additional (from ~15MB to ~20MB)

### NOT Recommended
- async-openai (50+ deps, OpenAI-only)
- Diesel ORM (system deps, sync-only)
- ssh2 (C FFI, session leak issues)
- Specialized pomodoro crates (just use std::time + chrono)

---

## Feature Prioritization

### Phase 1: Foundation (MVP) - 4-6 weeks
| Feature | Category | Complexity |
|---------|----------|------------|
| Task list (markdown checkboxes) | Productivity | Low |
| Pomodoro timer (25/5 cycle) | Productivity | Low |
| Quick notes panel | Productivity | Low |
| AI chat panel (single-file context) | AI | Medium |
| Command history (Ctrl+R search) | Terminal | Low |
| SQLite viewer (read-only) | Database | Medium |

### Phase 2: Differentiation
| Feature | Category | Value |
|---------|----------|-------|
| AI inline completions | AI | High |
| Terminal error → AI fix | Integration | High |
| Markdown table from SQL | Integration | Medium |
| Git task integration | Productivity | Medium |

### Phase 3: Polish (Defer)
- PostgreSQL/MySQL support
- Codebase-wide AI context
- Block-based terminal

---

## Architecture Pattern

```
┌────────────────────────────────────────────────┐
│              Main Thread (egui)                │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐       │
│  │ AI Panel │ │ DB Panel │ │SSH Panel │       │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘       │
│       │ try_recv() │            │              │
├───────┼────────────┼────────────┼──────────────┤
│       ▼            ▼            ▼              │
│  ┌─────────────────────────────────────────┐  │
│  │     mpsc::channel (Command/Response)    │  │
│  └─────────────────────────────────────────┘  │
├────────────────────────────────────────────────┤
│            Background Threads                  │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐       │
│  │AI Worker │ │DB Worker │ │SSH Worker│       │
│  │ (tokio)  │ │ (tokio)  │ │ (tokio)  │       │
│  └──────────┘ └──────────┘ └──────────┘       │
└────────────────────────────────────────────────┘
```

**Key Patterns:**
1. Tokio runtime in background threads (not main thread)
2. mpsc::channel for command/response
3. ctx.request_repaint() after state updates
4. Arc<Mutex<>> for large state (chat history, query results)
5. Lazy panel initialization (only create when first shown)

---

## Critical Pitfalls to Avoid

### 1. UI Thread Blocking (CRITICAL)
**Risk:** Calling `.await` in egui update() freezes UI
**Mitigation:** Always use background threads with channels

### 2. State Cloning Overhead (HIGH)
**Risk:** egui clones widget state 60x/sec
**Mitigation:** Use Arc<Mutex<>> for large data (>1KB)

### 3. SSH Session Leaks (HIGH)
**Risk:** ssh2-rs Drop fails silently in non-blocking mode
**Mitigation:** Use russh instead, or explicit cleanup with set_blocking(true)

### 4. Binary Size Bloat (MEDIUM)
**Risk:** Dependencies balloon from 15MB to 50MB
**Mitigation:** Feature-gate everything, disable default features, monitor with cargo-bloat

### 5. Keyboard Shortcut Conflicts (MEDIUM)
**Risk:** New panels steal shortcuts from terminal
**Mitigation:** Context-aware routing, only active panel receives shortcuts

---

## Build Order (Phases)

| Order | Component | Rationale |
|-------|-----------|-----------|
| 1 | Async infrastructure (tokio + channels) | Foundation for all async features |
| 2 | Settings extensions (panel visibility) | State management before UI |
| 3 | Database panel + SQLite | Simpler than AI streaming, proves pattern |
| 4 | AI panel + streaming | Most complex, builds on proven async |
| 5 | SSH panel | Reuses terminal infrastructure |
| 6 | Productivity panels | Simplest, mostly UI state |

---

## Success Metrics

| Metric | Target |
|--------|--------|
| UI frame time | <16ms during AI streaming |
| Binary size increase | <5MB |
| Memory increase | <50MB with all panels |
| Startup time increase | <200ms |
| Existing feature regressions | Zero |

---

## Open Questions (Deferred)

1. **AI Context Scope:** Single file vs all open tabs vs workspace (start with single file)
2. **Database Query Caching:** None initially, add LRU if needed
3. **Multi-Workspace:** Single workspace per window (defer multi-workspace)

---

## Confidence Assessment

| Area | Confidence | Reason |
|------|------------|--------|
| Async integration pattern | HIGH | egui-async, community discussions verified |
| Stack recommendations | HIGH | All crates verified via crates.io, 2026 sources |
| Feature prioritization | HIGH | Based on competitor analysis, user expectations |
| Pitfall identification | HIGH | Documented issues in GitHub, Stack Overflow |
| Build order | MEDIUM-HIGH | Logical dependencies clear, timing uncertain |

---

## Research Complete

All four dimensions researched with 30+ verified sources. Ready to proceed to requirements definition and roadmap creation.

**Files created:**
- `.planning/research/STACK.md` - Crate recommendations with versions
- `.planning/research/FEATURES.md` - Feature landscape with table stakes/differentiators
- `.planning/research/ARCHITECTURE.md` - Integration patterns and component design
- `.planning/research/PITFALLS.md` - Risk catalog with prevention strategies
- `.planning/research/SUMMARY.md` - This synthesis document
