# Requirements: Ferrite v0.5.0 "Swiss Army Knife"

**Created:** 2026-01-24
**Milestone:** v0.5.0
**Status:** Draft

---

## v1 Requirements (This Milestone)

### Async Foundation (INFRA)
- [x] **INFRA-01**: Tokio runtime runs in background threads (not main thread)
- [x] **INFRA-02**: Channel-based communication (mpsc) between UI and workers
- [x] **INFRA-03**: Settings struct extended with panel visibility flags
- [x] **INFRA-04**: Lazy panel initialization (only create when first shown)

### Developer Productivity Hub (PROD)
- [x] **PROD-01**: User can create tasks with markdown checkbox syntax (`- [ ]`)
- [x] **PROD-02**: User can mark tasks complete (strikethrough, checkbox checked)
- [x] **PROD-03**: Tasks persist across app restarts (JSON storage in .ferrite/)
- [x] **PROD-04**: User can start/stop Pomodoro timer (25/5 work/break)
- [x] **PROD-05**: User receives sound notification when Pomodoro ends
- [x] **PROD-06**: User can write quick notes in a dedicated panel
- [x] **PROD-07**: Notes persist per workspace (.ferrite/notes/)

### AI Assistant (AI)
- [ ] **AI-01**: User can open AI chat panel via View menu or keyboard shortcut
- [ ] **AI-02**: User can type prompts and receive streaming responses
- [ ] **AI-03**: AI panel shows current file content as context
- [ ] **AI-04**: User can select AI provider (Claude, OpenAI, Ollama)
- [ ] **AI-05**: User can configure API keys in settings (securely stored)
- [ ] **AI-06**: User can cancel in-progress AI requests
- [ ] **AI-07**: AI responses render as markdown

### Power Terminal (TERM)
- [ ] **TERM-01**: User can search command history with Ctrl+R
- [ ] **TERM-02**: Command history persists across sessions
- [ ] **TERM-03**: User can create SSH connection profiles (host, user, key)
- [ ] **TERM-04**: User can connect to SSH servers from dedicated panel
- [ ] **TERM-05**: SSH sessions display output in terminal-style widget
- [ ] **TERM-06**: User can search terminal scrollback with Ctrl+F

### Database Tools (DB)
- [ ] **DB-01**: User can open database panel via View menu
- [ ] **DB-02**: User can connect to SQLite databases (file picker)
- [ ] **DB-03**: User can browse database schema (tables, columns)
- [ ] **DB-04**: User can write and execute SQL queries
- [ ] **DB-05**: Query results display in scrollable table grid
- [ ] **DB-06**: User can export query results to CSV
- [ ] **DB-07**: User can export query results as markdown table
- [ ] **DB-08**: Database connections persist in settings

### Integration (INT)
- [x] **INT-01**: User can toggle all new panels via View menu
- [x] **INT-02**: Panel visibility persists across app restarts
- [x] **INT-03**: Keyboard shortcuts work context-aware (no conflicts with terminal)
- [x] **INT-04**: All panels follow existing Ferrite UI patterns

---

## Future Requirements (v0.6.0+)

### AI Enhancements
- [ ] AI inline completions (ghost text as you type)
- [ ] Terminal error → AI context (auto-detect errors, suggest fix)
- [ ] Codebase-wide AI context (multiple files)

### Productivity Enhancements
- [ ] Git branch → task creation
- [ ] Pomodoro time tracking per task
- [ ] Session summary generation

### Database Enhancements
- [ ] PostgreSQL support
- [ ] MySQL support
- [ ] Query autocomplete

### Terminal Enhancements
- [ ] AI command generation (natural language → shell)
- [ ] Block-based terminal output

---

## Out of Scope

| Feature | Reason |
|---------|--------|
| Cloud sync | Adds complexity, privacy concerns |
| Collaboration | Not a team tool, focus on individual |
| Calendar integration | Scope creep |
| Visual query builder | Complex UI, SQL editor is sufficient |
| ER diagram designer | Feature bloat |
| NoSQL databases | Different paradigm, defer |
| Multiple AI extensions | Conflicts, single unified panel |
| Built-in SFTP/FTP | Use terminal commands |
| Session recording playback | Storage-heavy |

---

## Traceability

| Requirement | Phase | Success Criteria |
|-------------|-------|------------------|
| INFRA-01..04 | 6 | Workers run without blocking UI |
| PROD-01..07 | 7 | Tasks and notes persist, timer works |
| AI-01..07 | 8 | Streaming chat works with 3 providers |
| TERM-01..06 | 9 | History search and SSH connect work |
| DB-01..08 | 10 | SQLite browse, query, export works |
| INT-01..04 | 11 | All panels accessible, no regressions |

---

## Quality Gates

| Metric | Minimum | Target |
|--------|---------|--------|
| UI frame time | <32ms | <16ms |
| Binary size increase | <10MB | <5MB |
| Memory increase (all panels) | <100MB | <50MB |
| Startup time increase | <500ms | <200ms |
| Existing feature regressions | 0 | 0 |
| Test coverage (new code) | 60% | 80% |
