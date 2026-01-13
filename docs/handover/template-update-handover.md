# Update Handover Instructions

<!--
PURPOSE: Instructions for the AI to update current-handover-prompt.md after task completion.

HOW TO USE:
1. Complete the current task
2. Paste this entire file into the chat
3. AI will mark task done and update the handover for the next task
4. Review the updated handover, adjust rules if needed
5. Close this chat (context is now stale)
6. Start a NEW chat and paste the updated current-handover-prompt.md

Customize the task structure section for your project's current version.
-->

Task is complete. Update the handover for the next session.

---

## 1. Mark Current Task Done

```bash
task-master set-status --id=<current-task-id> --status=done
```

## 2. Get Next Task

```bash
task-master next
```

Or if you know the specific next task:

```bash
task-master show <next-task-id>
```

## 3. Update current-handover-prompt.md

### Replace These Sections

| Section | New Content |
|---------|-------------|
| **Current Task** | Full details of next task (ID, title, description, implementation notes, test strategy) |
| **Key Files** | Only files relevant to the NEW task |
| **Context** | Remove old context, add new only if needed |

### Keep These Sections (usually unchanged)

| Section | When to Update |
|---------|----------------|
| **Rules** | Only if project rules changed |
| **Environment** | Only if version or tech stack changed |

### Remove

- Any previous task details
- Old "Last Session" content (unless using subtask template)
- Task-specific context that doesn't apply to new task

## 4. Template Selection

Choose the appropriate template based on next task:

| Situation | Template |
|-----------|----------|
| Independent task (most common) | `template-handover-minimal.md` |
| Continuing subtask chain | `template-handover-subtask.md` |
| Bug fix investigation | `template-handover-bugfix.md` |

## 5. Verification Checklist

- [ ] Current task marked as `done` in Task Master
- [ ] `current-handover-prompt.md` updated with next task
- [ ] Handover contains ONLY next task context (no previous task leftovers)
- [ ] Key Files section has only files for next task
- [ ] Code compiles: `cargo build` (or equivalent)

---

## Project Task Structure (Customize This)

<!--
Update this section for your current version/project.
Helps the AI understand task dependencies and order.
-->

```
[Version] Tasks:

[List task chains/waves here]
[e.g., Task 1 → 2 → 3 (dependency chain)]
[e.g., Tasks 4, 5, 6 (independent, can do in any order)]
```

---

## Notes

- Keep the handover **minimal and focused** on the next task
- Don't include full task lists or project overviews
- If a task needs context from a previous task, briefly note it in the Context section
- For subtask workflows, switch to `template-handover-subtask.md`
