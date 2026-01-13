# Session Handover - [Project] [Version] - Bug Fix

<!--
TEMPLATE: Bug Fix Handover
USE WHEN: Investigating or fixing a specific bug, need reproduction steps and prior attempts

Delete this comment block when using.
-->

## Rules

- Never auto-update this file - only update when explicitly requested
- Verify the bug is reproducible before attempting fixes
- Run build/check after changes to verify code compiles
- Test the fix against the reproduction steps
- Update task status via Task Master when starting (`in-progress`) and completing (`done`)
- Use Context7 MCP tool to fetch library documentation when needed
- Document by feature (e.g., `editor-widget.md`), not by task (e.g., `task-1.md`)
- Update `docs/index.md` when adding new documentation

## Environment

- **Project:** [Project Name]
- **Path:** [Full path to project]
- **Tech Stack:** [Language, frameworks, key dependencies]
- **Version:** [Current version]

---

## Bug Details

| Field | Value |
|-------|-------|
| **Issue** | [#XX or N/A] |
| **Task ID** | [Task Master ID if applicable] |
| **Severity** | [Critical / High / Medium / Low] |
| **Reproducible** | [Yes / No / Intermittent] |
| **Reported By** | [User / Tester / Self] |
| **First Seen** | [Version or date] |

### Summary

<!--
One paragraph describing the bug.
-->

### Reproduction Steps

<!--
Exact steps to reproduce the bug.
Be specific - include file contents, actions, etc.
-->

1. [Step 1]
2. [Step 2]
3. [Step 3]
4. **Bug occurs:** [What happens]

### Expected Behavior

<!--
What should happen instead.
-->

### Actual Behavior

<!--
What actually happens. Include error messages, screenshots references, etc.
-->

---

## Investigation

### Hypothesis

<!--
Current best guess about the root cause.
-->

### Key Files

| File | Relevance |
|------|-----------|
| `path/to/file.rs` | [Why this file is suspected] |

### Previous Attempts (if any)

<!--
Document what has been tried before.
This prevents repeating failed approaches.
-->

| Attempt | What Was Tried | Result |
|---------|----------------|--------|
| 1 | [Approach] | [Why it didn't work] |
| 2 | [Approach] | [Why it didn't work] |

---

## Test Strategy

### Verify Fix

<!--
How to confirm the bug is fixed.
-->

1. Follow reproduction steps above
2. [Expected: Bug no longer occurs]

### Regression Check

<!--
What else to test to ensure the fix doesn't break anything.
-->

- [ ] [Related feature 1 still works]
- [ ] [Related feature 2 still works]
