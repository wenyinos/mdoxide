# Phase 7: Productivity Hub - Context

**Gathered:** 2026-01-24
**Status:** Ready for planning

<domain>
## Phase Boundary

Add task tracking, Pomodoro timer, and quick notes as toggleable panels. Users can create/complete tasks, run timed work sessions with sound notifications, and take workspace-scoped notes. All data persists in `.ferrite/`.

</domain>

<decisions>
## Implementation Decisions

### Task Input & Display
- Markdown syntax for task creation: type `- [ ] task` directly, checkbox auto-renders
- Completed tasks stay in place with strikethrough styling (no moving/hiding)
- Priority markers supported: `!` or `!!` in task text for priority levels
- Flat list organization, manual ordering (no auto-grouping)

### Notes Panel
- Plain text editor (no markdown preview or WYSIWYG)
- Per workspace scoping: each workspace directory has its own notes
- Auto-save as you type (debounced)
- Multiple notes supported: create and switch between named notes

### Claude's Discretion
- Pomodoro timer controls and visual design
- Pomodoro break handling and cycle behavior
- Notification sound selection/timing
- How tasks/timer/notes share panel space (tabs vs sections)
- Task reordering mechanism
- Note naming conventions and storage structure
- Debounce timing for auto-save

</decisions>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 07-productivity-hub*
*Context gathered: 2026-01-24*
