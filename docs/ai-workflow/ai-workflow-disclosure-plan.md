# AI Workflow Disclosure Plan

This document outlines how to properly share Ferrite's AI-assisted development process with the community.

---

## Goals

1. **Transparency** — Be fully open about how Ferrite is built
2. **Educational** — Help others learn from this workflow
3. **Reproducibility** — Share enough that others could adopt similar approaches
4. **Security** — Ensure no API keys, secrets, or private info is exposed

---

## Decisions Made

| Question | Decision |
|----------|----------|
| Archive scope | All available handovers, organized by version |
| Task-master config | Don't include config, just document usage |
| PRD sharing | Yes, share actual PRDs in organized folder |
| Session logs | No, too much overhead |

---

## Current Archive State (Needs Reorganization)

### Found in `archive/` (root):
```
archive/
├── HANDOVER-about-help.md           # Old handover (feature-specific)
├── HANDOVER-list-editing-bug.md     # Old handover (bug-specific)
├── HANDOVER-minor-bugs.md           # Old handover (bug-specific)
├── code-review-handover.md          # Code review prompt
├── code-review-v0.2.0-findings.md   # Review findings
├── review-tasks-prompt.md           # Review prompt
├── taskmaster-v0.2.0/               # Full taskmaster archive
│   └── docs/prd.txt, prd-v0.2.0-list-editing-bug.txt
├── taskmaster-v0.2.2/               # Partial archive
│   └── prd.txt, tasks.json
├── tasks-v0.1.x-completed.json      # Old task data
├── tasks-v0.2.0-completed.json      # Old task data
└── [various test/example files]
```

### Found in `.taskmaster/archive/`:
```
.taskmaster/archive/
├── v0.2.3-editor-productivity/
│   └── prd.md, tasks.json
└── v0.3.0-mermaid-crate/
    └── prd.txt, tasks.json
```

### Current PRD:
- `.taskmaster/docs/prd.md` — v0.2.5 PRD

### Current Handovers (in docs/):
- `docs/current-handover-prompt.md` — Active
- `docs/update-handover-prompt.md` — Instructions

---

## Proposed New Structure

```
docs/
├── ai-development-workflow.md       # Main workflow documentation (NEW)
├── current-handover-prompt.md       # Active handover (EXISTS)
├── update-handover-prompt.md        # Update instructions (EXISTS)
└── ai-workflow/                     # AI workflow folder (NEW)
    ├── README.md                    # Index and explanation
    ├── handovers/                   # Historical handovers
    │   ├── handover-about-help.md
    │   ├── handover-list-editing-bug.md
    │   ├── handover-minor-bugs.md
    │   └── code-review-v0.2.0.md
    └── prds/                        # Product Requirements Documents
        ├── prd-v0.2.0.md
        ├── prd-v0.2.0-list-editing-bug.md
        ├── prd-v0.2.2.md
        ├── prd-v0.2.3.md
        ├── prd-v0.3.0.md
        └── prd-v0.2.5.md            # Copy from current
```

---

## Security Audit Checklist

Before committing, verify each file:

| Check | What to Look For |
|-------|-----------------|
| ❌ No API keys | `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, etc. |
| ❌ No personal paths | `C:\Users\lbh\`, `/home/user/` |
| ❌ No private URLs | Non-public GitHub links, internal tools |
| ❌ No email addresses | Personal contact info |
| ❌ No tokens/secrets | Bearer tokens, session IDs |

### Files Requiring Audit

| File | Status |
|------|--------|
| `docs/current-handover-prompt.md` | ✅ Already public, verified |
| `docs/update-handover-prompt.md` | ✅ Already public, verified |
| `archive/HANDOVER-*.md` | ⚠️ **Needs human audit** |
| `archive/code-review-*.md` | ⚠️ **Needs human audit** |
| `archive/taskmaster-*/docs/*.txt` | ⚠️ **Needs human audit** |
| `.taskmaster/archive/*/prd.*` | ⚠️ **Needs human audit** |

---

## Implementation Plan

### Phase 1: Create New Structure ✅
- [x] Create `docs/ai-workflow/` folder
- [x] Create `docs/ai-workflow/handovers/` subfolder
- [x] Create `docs/ai-workflow/prds/` subfolder
- [x] Create `docs/ai-workflow/tasks/` subfolder (added)
- [x] Create `docs/ai-workflow/notes/` subfolder (added)
- [x] Create `docs/ai-workflow/README.md` index

### Phase 2: Human Audit ✅
- [x] Review each file in `archive/HANDOVER-*.md` for secrets
- [x] Review each file in `archive/taskmaster-*/docs/*.txt` for secrets
- [x] Review files in `.taskmaster/archive/*/` for secrets
- [x] Mark files as safe or flag issues

### Phase 3: Move & Organize Files ✅
- [x] Copy audited handovers to `docs/ai-workflow/handovers/`
- [x] Copy audited PRDs to `docs/ai-workflow/prds/`
- [x] Copy task JSON files to `docs/ai-workflow/tasks/`
- [x] Copy notes to `docs/ai-workflow/notes/`
- [x] Rename files consistently (lowercase, version-prefixed)

### Phase 4: Create Documentation ✅
- [x] Create `docs/ai-development-workflow.md` (full process doc)
- [x] Update README with expanded AI disclosure section

### Phase 5: Cleanup (Optional)
- [ ] Consider removing `archive/` from git (move to .gitignore)
- [ ] Or keep `archive/` for non-essential files (test files, screenshots)

---

## README Section Draft

```markdown
## 🤖 AI-Assisted Development

This project is 100% AI-generated code. All Rust code, documentation, and configuration was written by Claude (Anthropic) via [Cursor](https://cursor.com) with MCP tools.

### My Role
- **Product direction** — Deciding what to build and why
- **Testing** — Running the app, finding bugs, verifying features
- **Review** — Reading generated code, understanding what it does
- **Orchestration** — Managing the AI workflow effectively

### The Workflow
1. **Idea refinement** — Discuss concepts with multiple AIs (Claude, Perplexity, Gemini Pro)
2. **PRD creation** — Generate requirements using [Task-master MCP](https://github.com/task-master-ai/task-master)
3. **Task execution** — Claude Opus 4.5 handles implementation (preferring larger tasks over many subtasks)
4. **Session handover** — Structured prompts maintain context between sessions
5. **Human review** — Every handover is reviewed; direction adjustments made as needed

📖 **Full details:** [AI Development Workflow](docs/ai-development-workflow.md)

### Open Process
The actual prompts and documents used to build Ferrite are public:

| Document | Purpose |
|----------|---------|
| [`current-handover-prompt.md`](docs/current-handover-prompt.md) | Active session context |
| [`update-handover-prompt.md`](docs/update-handover-prompt.md) | Handover update instructions |
| [`ai-development-workflow.md`](docs/ai-development-workflow.md) | Full workflow documentation |
| [`ai-workflow/prds/`](docs/ai-workflow/prds/) | Product Requirements Documents |
| [`ai-workflow/handovers/`](docs/ai-workflow/handovers/) | Historical handover prompts |

This transparency is intentional — I want others to learn from (and improve upon) this approach.
```

---

## Next Steps

1. ✅ Plan created (this document)
2. ✅ Audit files in `archive/` for secrets
3. ✅ Create folder structure and move audited files
4. ✅ Create `docs/ai-development-workflow.md`
5. ✅ Update README with expanded AI section
6. **🔴 TODO:** Review changes and commit
7. **🔴 TODO:** Decide whether to clean up `archive/` folder
