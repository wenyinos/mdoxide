# Session Handover - [Project] [Version]

<!--
TEMPLATE: Minimal Handover
USE WHEN: Independent tasks with no subtasks, fresh session start

Delete this comment block when using.
-->

## Rules

<!--
Customize per project. These are behavioral instructions for the AI.
Common rules to consider:
-->

- Never auto-update this file - only update when explicitly requested
- Complete entire task before requesting next instruction
- Run build/check after changes to verify code compiles
- Follow existing code patterns and conventions
- Update task status via Task Master when starting (`in-progress`) and completing (`done`)
- Use Context7 MCP tool to fetch library documentation when needed (resolve library ID first, then fetch docs)
- Document by feature (e.g., `editor-widget.md`), not by task (e.g., `task-1.md`)
- Update `docs/index.md` when adding new documentation

## Environment

- **Project:** [Project Name]
- **Path:** [Full path to project]
- **Tech Stack:** [Language, frameworks, key dependencies]
- **Version:** [Current version being worked on]

---

## Current Task

| Field | Value |
|-------|-------|
| **ID** | [Task ID from Task Master] |
| **Title** | [Task title] |
| **Status** | `pending` |
| **Priority** | [High/Medium/Low] |
| **Dependencies** | [None or list of prerequisite task IDs] |

### Description

<!--
Copy from Task Master or summarize from PRD.
What needs to be done and why.
-->

### Implementation Notes

<!--
Specific guidance for this task:
- Key decisions already made
- Approach to take
- Gotchas to watch out for
-->

### Test Strategy

<!--
How to verify the task is complete:
- Manual testing steps
- Automated tests to run
- Acceptance criteria
-->

---

## Key Files

<!--
ONLY files relevant to THIS task.
Remove any files from previous tasks.
-->

| File | Purpose |
|------|---------|
| `path/to/file.rs` | [Why this file matters for this task] |

---

## Context (Optional)

<!--
Only include if there's specific knowledge needed that isn't in the task description.
Examples:
- A decision made in a previous session that affects this task
- A known issue to work around
- External documentation to reference

If nothing is needed, delete this section entirely.
-->
