# Update Handover Instructions

Task is complete. Update the handover for the next session.

---

## 1. Mark Current Task Done
Use Task Master MCP tool to set status:
`set_task_status --id=<current-task-id> --status=done`
Prefer MCP tools over CLI commands for Task Master operations.

## 2. Create Documentation
Create feature-based documentation for completed work.
1. Identify what was implemented (group by feature, not task).
2. Create doc in `docs/technical/` (or `docs/` for top-level topics).
3. **Update `docs/index.md`** with the new entry and a 1-line description.

*Naming:* Good: `lsp-on-demand-startup.md`. Bad: `task-33.md`.

## 3. Get Next Task
Fetch the next task using MCP: `next_task` or `get_task --id=<next-task-id>`

## 4. Update current-handover-prompt.md
Replace the current task sections with the new task:
- **Current Task:** Full details of next task (ID, title, description, complexity, etc.)
- **Key Files:** Only files relevant to the NEW task.
- **Model Selection:** Update complexity and model recommendation.

*Crucial Rule:* Remove ANY previous task details. Do NOT accumulate a task history.

## 5. Update ai-context.md (if needed)
If the completed task changed the architecture significantly:
- Update the Architecture section or "Where Things Live" table.
- Keep the file under ~100 lines.
- Do NOT add task history or "Recently Changed" entries.

## 6. Verification Checklist
- [ ] Current task marked as `done` in Task Master
- [ ] Feature documentation created
- [ ] `docs/index.md` updated with new doc entry
- [ ] `current-handover-prompt.md` updated with ONLY next task context
- [ ] `ai-context.md` updated if architecture changed
- [ ] Code compiles and tests pass: `cargo build`
