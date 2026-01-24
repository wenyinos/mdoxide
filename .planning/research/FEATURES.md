# Features Research: Ferrite v0.5.0 "Swiss Army Knife"

**Domain:** Developer productivity tools, AI-assisted editors, terminal emulators, database GUIs
**Researched:** 2026-01-24
**Overall Confidence:** HIGH

## Executive Summary

Research analyzed four feature categories for Ferrite v0.5.0, focusing on what users expect (table stakes), what creates competitive advantage (differentiators), and what to avoid (anti-features). Key finding: successful tools in this space balance minimalism with power - avoiding feature bloat while providing deep capabilities in narrow domains.

**Critical insight:** The 2026 landscape shows users are fatigued by slow, bloated tools. Lightweight alternatives (Beekeeper Studio, Ghostty, Cursor) are gaining traction by focusing on speed and essential features over comprehensive coverage.

---

## Developer Productivity Hub

### Table Stakes

Features users expect in any productivity tool. Missing these makes the tool feel incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **Task list with checkboxes** | Core productivity primitive | Low | Markdown-style `- [ ]` checkbox syntax is standard |
| **Basic time tracking** | Developers track time per task | Medium | Start/stop timer, manual time entry |
| **Pomodoro timer** | Standard focus technique | Low | 25/5 work/break cycles, customizable intervals |
| **Quick notes panel** | Capture thoughts without leaving app | Low | Plain text or markdown, auto-save |
| **Task completion tracking** | See progress visually | Low | Simple checkboxes, strikethrough completed items |
| **Persistence** | Tasks survive app restart | Low | JSON or markdown file storage |

**Source confidence:** HIGH - Based on analysis of [Super Productivity](https://super-productivity.com/), [Focus To-Do](https://www.focustodoapp.com), and [Taskade](https://www.taskade.com).

### Differentiators

Features that set tools apart. Not expected, but create competitive advantage.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Git integration** | Auto-create tasks from commits/branches | Medium | Show commit count next to tasks, create task from branch name |
| **Context-aware notes** | Notes per file/workspace | Low | Store notes in `.ferrite/notes/` per project |
| **Pomodoro integration with tasks** | Track which task consumed time | Medium | Link timer to specific task, generate time reports |
| **Session boundary detection** | Auto-summarize work sessions | High | Detect when Claude Code stops, create session summary |
| **Local-first + no cloud** | Privacy for developer workflows | Low | Everything stored locally, no telemetry |
| **Minimal UI overhead** | Sidebar panel, not separate window | Low | Fits Ferrite's focus on speed and lightweight feel |

**Source confidence:** HIGH - Based on [Super Productivity](https://super-productivity.com/) (Jira/GitHub sync), Focus To-Do (task-timer linking).

**Key differentiator for Ferrite:** Integrate with existing terminal/git features. Example: "Create task from current git branch", "Log terminal command to task notes", "Auto-start pomodoro when Claude Code session begins".

### Anti-Features

Features to explicitly NOT build. Common mistakes in this domain.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| **Cloud sync** | Adds complexity, latency, privacy concerns | Local storage only, export/import for sharing |
| **Collaboration features** | Not a team tool, focus on individual workflow | Single-user experience, export session for handoff |
| **Complex task dependencies** | Gantt charts, critical paths - overengineering | Simple parent/child tasks max 2 levels deep |
| **Calendar integration** | Scope creep, not core to developer productivity | Show daily task list, skip calendar view |
| **Project management views** | Kanban boards, roadmaps - too complex | Flat task list with optional grouping (by tag/file) |
| **Analytics dashboards** | Feature bloat, slows app startup | Simple stats panel (tasks completed today/week) |

**Source confidence:** MEDIUM - Based on [overengineering warnings](https://leaddev.com/software-quality/the-6-warning-signs-of-overengineering) and [feature bloat analysis](https://hellopm.co/what-is-feature-bloat/).

**Rationale:** Ferrite targets "powerful fast lightweight customizable". Complex PM features belong in dedicated tools (Jira, Linear). Keep it simple: tasks, timer, notes.

---

## AI-First Editor

### Table Stakes

Features users expect from AI code assistants in 2026.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **Inline completions** | Standard in VSCode, Cursor, etc. | High | Ghost text suggestions as you type |
| **Chat panel** | Ask questions about code | Medium | Side panel with context-aware responses |
| **Context awareness** | AI knows current file content | Medium | Send buffer content with prompts |
| **Accept/reject shortcuts** | Tab to accept, Esc to dismiss | Low | Standard keybindings from GitHub Copilot |
| **Streaming responses** | Real-time text generation | Medium | WebSocket or SSE for live token delivery |
| **Model selection** | Choose between models (GPT-4, Claude, etc.) | Low | Dropdown in settings, API key per provider |

**Source confidence:** HIGH - Based on [GitHub Copilot](https://code.visualstudio.com/docs/copilot/ai-powered-suggestions), [Cursor](https://www.cursor.com), and [Continue](https://continue.dev) documentation.

### Differentiators

Features that create competitive advantage in AI editors.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Codebase-wide context** | Understand entire project, not just file | Very High | Requires indexing, embedding, RAG system |
| **Multi-file edits (Composer)** | Edit multiple files with one prompt | Very High | Plan changes, apply across files, show diff |
| **Local model support (Ollama)** | No API costs, full privacy | Medium | HTTP API to localhost Ollama server |
| **Edit + delete code** | Remove code, not just insert | High | Cursor's differentiator over Copilot |
| **Markdown-aware completions** | Suggest markdown formatting, links | Medium | Ferrite-specific: optimize for markdown editing |
| **Terminal-aware context** | Include terminal output in AI context | Medium | "Fix the error shown in Terminal 2" |
| **Breathing indicator sync** | AI knows when Claude Code is waiting | Low | Pass terminal state to AI context |

**Source confidence:** HIGH - Based on [Cursor vs VSCode comparisons](https://is4.ai/blog/our-blog-1/cursor-vs-vscode-copilot-comparison-2026-165) and [Warp AI capabilities](https://www.warp.dev/warp-ai).

**Key differentiator for Ferrite:** Integrate AI with terminal workspace. Example: "AI panel shows suggestions based on terminal errors", "Generate markdown documentation from terminal commands".

**Critical constraint:** Codebase-wide context and multi-file editing are VERY HARD and cause significant performance issues (indexing overhead, memory usage). Start with single-file context only.

### Anti-Features

Features to avoid based on 2026 industry lessons.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| **Multiple AI extensions** | Conflicts, performance degradation | Single unified AI panel with model switching |
| **Automatic code execution** | Security risk, user loses control | Always require explicit confirmation |
| **Cloud-side indexing** | Privacy concerns, latency | Local models (Ollama) or API-only (no indexing) |
| **Auto-accept suggestions** | Leads to 19% slowdown (verification overhead) | Always require Tab to accept |
| **Excessive context window** | Slow, expensive, often unnecessary | Start with current file + 100 lines, expand if needed |
| **Hidden AI actions** | Users don't trust black box behavior | Always show what AI sees (context panel) |

**Source confidence:** HIGH - Based on [AI coding slowdown study](https://www.infoworld.com/article/4020931/ai-coding-tools-can-slow-down-seasoned-developers-by-19.html) and [verification overhead analysis](https://www.cerbos.dev/blog/productivity-paradox-of-ai-coding-assistants).

**Critical lesson from 2026 research:** AI assistants make experienced developers 19% slower due to verification overhead. Users accept less than 44% of suggestions and spend significant time reviewing code. Solution: Make AI suggestions easy to review (inline diffs, clear context).

---

## Power Terminal

### Table Stakes

Features users expect from modern terminal emulators.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **SSH session management** | Connect to remote servers | Medium | Store connection profiles (host, user, key) |
| **Command history search** | Find previous commands | Low | Ctrl+R reverse search, persistent history |
| **Session persistence** | Reconnect after close | Medium | Save working directory, environment |
| **Multiple shell support** | PowerShell, bash, WSL, etc. | Low | Already implemented in Ferrite |
| **Copy/paste that works** | Ctrl+Shift+C/V standard | Low | Already implemented in Ferrite |
| **Searchable scrollback** | Find text in terminal output | Medium | Ctrl+F in terminal buffer |

**Source confidence:** HIGH - Based on [Termius](https://termius.com), [MobaXterm](https://mobaxterm.mobatek.net), and [iTerm2](https://iterm2.com) feature sets.

### Differentiators

Features that create competitive advantage for power users.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Block-based output** | Commands as distinct blocks (Warp style) | High | Parse command start/end, treat as units |
| **Command sharing** | Share terminal blocks as permalinks | Medium | Export block as JSON/image/HTML |
| **AI command generation** | Natural language → shell command | Medium | Integrate with AI panel: "Generate command to..." |
| **Runbooks/workflows** | Save command sequences | Medium | Already partially implemented (macros/playback) |
| **Smart command detection** | Detect build/test/deploy commands | Medium | Already implemented (git status, build detection) |
| **GPU acceleration** | Faster rendering for large output | Very High | Requires low-level graphics work |
| **Collaboration features** | Share live terminal session | Very High | Requires server infrastructure (out of scope) |

**Source confidence:** HIGH - Based on [Warp Terminal](https://www.warp.dev) (blocks, AI, sharing) and [Wave Terminal](https://waveterm.dev) (workflows).

**Key differentiator for Ferrite:** Extend existing smart detection. Examples: "Auto-save failed command to runbook", "AI suggests fix for error in Terminal 2", "Export terminal block to markdown documentation".

### Anti-Features

Features to avoid in pursuit of lightweight performance.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| **Web-based terminal** | Electron bloat, slow startup | Native egui rendering (already done) |
| **Cloud sync for sessions** | Latency, complexity, privacy | Local save/load only, export for sharing |
| **Built-in SFTP/FTP** | Feature creep, security complexity | Use standard tools (scp, rsync) in terminal |
| **Tabbed SSH tunneling UI** | Complex, niche use case | Use standard SSH commands in terminal |
| **Built-in text editor in terminal** | Redundant with Ferrite's main editor | Edit files in Ferrite editor, run commands in terminal |
| **Session recording playback** | Complex, storage-heavy | Export as HTML/text, skip video playback |

**Source confidence:** MEDIUM - Based on [terminal emulator bloat discussions](https://blog.codeminer42.com/modern-terminals-alacritty-kitty-and-ghostty/) and minimalist design principles.

**Rationale:** Ferrite already has terminal workspace foundations (splits, themes, smart detection). Build on strengths, avoid recreating SSH client features that belong in dedicated tools.

---

## Database Tools

### Table Stakes

Features users expect from database GUIs.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **SQL query editor** | Write and execute SQL | Medium | Syntax highlighting, execute button |
| **Table data viewer** | Browse table rows in grid | Medium | Spreadsheet-like view, sortable columns |
| **Schema browser** | See tables, columns, types | Medium | Tree view of database structure |
| **Basic CRUD** | Create, read, update, delete rows | Medium | Double-click cell to edit, save/cancel |
| **Multiple database support** | SQLite, PostgreSQL, MySQL | High | Different connection drivers per DB type |
| **Export data** | CSV, JSON, SQL export | Low | Export query results or table data |
| **Connection management** | Save database connections | Low | Store connection strings, test connection |

**Source confidence:** HIGH - Based on [DBeaver](https://dbeaver.io), [Beekeeper Studio](https://www.beekeeperstudio.io), and [DB Browser for SQLite](https://sqlitebrowser.org).

### Differentiators

Features that create competitive advantage while staying lightweight.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Markdown table generation** | Export query results as markdown | Low | Perfect fit for Ferrite's markdown focus |
| **SQLite first-class support** | Local-first, zero-config databases | Low | Single-file databases, no server setup |
| **Query history** | Re-run previous queries | Low | Save queries in `.ferrite/db-history/` |
| **Quick data inspection** | Open .db file from file tree → instant view | Medium | Drag .sqlite file into Ferrite, auto-open viewer |
| **Inline query results in markdown** | Execute SQL in markdown code blocks | Medium | Run query, insert results as table in document |
| **Fast startup** | No Electron, no heavy ORM | Low | Native egui + rusqlite/postgres crates |
| **Read-only mode default** | Safety: prevent accidental data changes | Low | Require explicit "Enable editing" checkbox |

**Source confidence:** HIGH - Based on [Beekeeper Studio](https://www.beekeeperstudio.io) (lightweight focus) and [TablePlus](https://tableplus.com) (speed emphasis).

**Key differentiator for Ferrite:** Deep integration with markdown editing. Example: "Run SQL query in code block, insert results as markdown table", "Generate markdown documentation from database schema".

**MVP recommendation:** Start with SQLite only. Defer PostgreSQL/MySQL to post-MVP. Rationale:
- SQLite requires no server setup (single file)
- Rust has excellent SQLite support (rusqlite crate)
- Fits "lightweight" philosophy
- Covers 80% of developer use cases (local testing, data storage)

### Anti-Features

Features to avoid to maintain speed and simplicity.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| **Visual query builder** | Complex UI, slow to use | SQL editor with autocomplete only |
| **ER diagram designer** | Feature bloat, complex rendering | Show schema as text/tree view |
| **Database migrations UI** | Belongs in migration tools (Flyway, Liquibase) | Execute migration scripts in SQL editor |
| **User management UI** | Niche feature, complex | Run GRANT/REVOKE commands in SQL editor |
| **Stored procedure editor** | Complex, language-specific | Edit .sql files in main editor, execute in DB panel |
| **NoSQL databases** | Scope creep, different paradigm | Focus on SQL databases only |
| **Built-in data modeling** | Complex, belongs in dedicated tools | Import existing database, browse schema |

**Source confidence:** HIGH - Based on [database GUI bloat analysis](https://www.beekeeperstudio.io/alternatives/dbeaver) (DBeaver as cautionary example).

**Rationale:** Tools like DBeaver become slow and complex by trying to support every database feature. Keep Ferrite focused: query editor + data viewer + SQLite focus = fast, simple, useful.

---

## Feature Dependencies

### Interactions with Existing Capabilities

| New Feature | Depends On Existing Feature | Integration Point |
|-------------|------------------------------|-------------------|
| **Task tracking** | Session persistence | Store tasks in `.ferrite/tasks.json` per workspace |
| **Pomodoro timer** | Sound notification system | Reuse existing rodio audio for timer alerts |
| **AI chat panel** | Terminal smart detection | Include terminal state in AI context ("Terminal 2 shows error X") |
| **AI inline completions** | Markdown syntax highlighting | AI understands markdown structure for better suggestions |
| **Command history search** | Terminal scrollback buffer | Search existing buffer with regex |
| **SSH sessions** | Terminal tabs + shell selector | Add "SSH" as shell type, store connection profiles |
| **Database viewer** | File tree workspace | Detect .db/.sqlite files, add "Open in DB Viewer" context menu |
| **Query results export** | Markdown editor | Insert query results as markdown table in current file |

### Feature Interaction Matrix

**Productivity Hub + AI:**
- Pomodoro timer auto-starts when AI detects Claude Code session begins
- AI generates task list from git commit messages
- Tasks link to specific files (click task → open file in editor)

**AI + Terminal:**
- AI panel shows suggestions based on terminal errors
- "Fix error in Terminal 2" command sends error context to AI
- AI generates shell commands from natural language

**Terminal + Database:**
- Run SQL queries in terminal (psql, sqlite3)
- Database panel shows results in grid view
- Export terminal SQL output to markdown table

**All features + Workspace:**
- All panels save state per workspace (`.ferrite/` folder)
- Session export includes tasks, notes, terminal layouts, database connections
- Workspace switcher restores entire state (files + terminals + tasks + DB)

### Dependency Constraints

**Must be implemented in order:**
1. **AI panel foundation** → Required before inline completions (panel provides model selection, API key management)
2. **SQLite viewer** → Required before multi-database support (prove architecture works with one DB first)
3. **Task storage** → Required before git/timer integration (need basic persistence first)

**Can be implemented in parallel:**
- Productivity hub (tasks, pomodoro, notes) - independent panel
- AI chat panel - independent panel
- Database viewer - independent panel
- SSH sessions - extends existing terminal

---

## MVP Feature Prioritization

### Phase 1: Foundation (Minimum Viable Product)

**Goal:** Prove each feature category with simplest implementation.

| Feature | Scope | Why MVP | Complexity |
|---------|-------|---------|------------|
| **Task list** | Markdown checkbox syntax, auto-save | Core productivity primitive | Low |
| **Pomodoro timer** | 25/5 cycle, sound alert, start/stop | Standard focus tool | Low |
| **Quick notes** | Plain text panel, per-workspace storage | Capture thoughts quickly | Low |
| **AI chat panel** | Single-file context, Claude/OpenAI API | Prove AI integration works | Medium |
| **Command history** | Ctrl+R search in terminal | Expected terminal feature | Low |
| **SQLite viewer** | Read-only table browser, query editor | Prove database integration | Medium |

**Total MVP effort:** ~4-6 weeks focused development

### Phase 2: Differentiation (Post-MVP)

Features that create competitive advantage but require MVP foundation.

| Feature | Dependency | Value |
|---------|-----------|-------|
| **AI inline completions** | Requires AI panel (model selection, API keys) | High - major differentiator |
| **Git task integration** | Requires task storage | Medium - useful for developers |
| **Terminal error → AI** | Requires AI panel + terminal detection | High - unique integration |
| **Markdown table from SQL** | Requires DB viewer + markdown editor | Medium - fits Ferrite's focus |
| **Session export (all panels)** | Requires all panels implemented | Low - nice to have |

### Phase 3: Polish (Long-term)

Advanced features that require significant effort.

| Feature | Effort | Priority |
|---------|--------|----------|
| **Codebase-wide AI context** | Very High | Low - complex, may slow app |
| **PostgreSQL/MySQL support** | High | Medium - after SQLite proven |
| **Block-based terminal** | Very High | Low - major architecture change |
| **GPU-accelerated terminal** | Very High | Low - marginal benefit for Ferrite's use case |

---

## Sources

### Developer Productivity Hub
- [Super Productivity - Open-Source Deep Work Task Manager](https://super-productivity.com/)
- [Best To-Do Apps for Developers in 2025](https://super-productivity.com/blog/developer-todo-app/)
- [Top 11 Pomodoro Timer Apps for 2026](https://reclaim.ai/blog/best-pomodoro-timer-apps)
- [The 6 warning signs of overengineering - LeadDev](https://leaddev.com/software-quality/the-6-warning-signs-of-overengineering)

### AI-First Editor
- [Inline suggestions from GitHub Copilot in VS Code](https://code.visualstudio.com/docs/copilot/ai-powered-suggestions)
- [Best AI Code Editors 2026 (I Tested 10+)](https://playcode.io/blog/best-ai-code-editors-2026)
- [Cursor vs VS Code with Copilot: Best AI Code Editor 2026](https://is4.ai/blog/our-blog-1/cursor-vs-vscode-copilot-comparison-2026-165)
- [AI coding tools can slow down seasoned developers by 19%](https://www.infoworld.com/article/4020931/ai-coding-tools-can-slow-down-seasoned-developers-by-19.html)
- [The Productivity Paradox of AI Coding Assistants](https://www.cerbos.dev/blog/productivity-paradox-of-ai-coding-assistants)
- [Newer AI Coding Assistants Are Failing in Insidious Ways](https://spectrum.ieee.org/ai-coding-degrades)

### Power Terminal
- [Termius - Modern SSH Client](https://termius.com/changelog)
- [Warp Terminal in 2026: A First-Person Guide](https://thelinuxcode.com/warp-terminal-in-2026-a-first-person-guide-to-fast-ai-first-command-work/)
- [Warp: Warp vs. iTerm2 - Comparison with Pros & Cons](https://www.warp.dev/compare-terminal-tools/iterm2-vs-warp)
- [The Modern Terminals Showdown: Alacritty, Kitty, and Ghostty](https://blog.codeminer42.com/modern-terminals-alacritty-kitty-and-ghostty/)
- [23 Best Terminal Emulators Reviewed in 2026](https://thectoclub.com/tools/best-terminal-emulator/)

### Database Tools
- [DBeaver Community | Free Open-Source Database Management Tool](https://dbeaver.io/)
- [Beekeeper Studio | Easy To Use DBeaver Alternative](https://www.beekeeperstudio.io/alternatives/dbeaver)
- [DB Browser for SQLite](https://sqlitebrowser.org/)
- [Top 8 SQL Database Viewer Tools for 2026](https://www.devart.com/dbforge/top-8-sql-database-viewer-tools.html)
- [Best DBeaver Alternatives: Top Database Management Tools in 2025](https://alternativeto.net/software/dbeaver/)

### Feature Bloat & Anti-Patterns
- [What is Feature Bloat? Causes, Risks, and How to Prevent It](https://hellopm.co/what-is-feature-bloat/)
- [Software bloat - Wikipedia](https://en.wikipedia.org/wiki/Software_bloat)
- [Why AI Coding Assistants Are Making You Slower](https://medium.com/@_wadew/why-ai-coding-assistants-are-making-you-slower-and-what-nobodys-telling-you-to-fix-it-357be6050db1)
