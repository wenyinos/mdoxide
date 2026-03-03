# Handover: Model Assignment for Remaining Tasks

**Use this prompt in a new chat with a high-tier model** (e.g. Claude Opus, GPT-4, or your best available) to get recommended model tiers per task.

---

## Your goal

Analyze the **remaining active tasks** for the Ferrite markdown editor project and **suggest which model tier to use for implementing each task**.

- **Bias toward a higher tier when unsure.** If it's ambiguous, recommend the higher model so we don't assign a weaker model and get mistakes or rework.
- Consider: parser/state-machine work, cross-platform behavior, refactors that must preserve behavior, and integration surface (many files vs. one).

Output a clear table or list: **Task ID | Title | Suggested model tier | Short reason**.

---

## Project context

- **Project**: Ferrite – Rust markdown editor (egui, custom Ferrite editor, markdown/mermaid).
- **Branch**: `master`
- **Tasks**: 18 total. **3 done** (11, 12, 13), **3 deferred** (22, 23, 24). **12 active** (14–21, 25–28).
- **Complexity report**: Research-backed analysis already run. Location:  
  `.taskmaster/reports/task-complexity-report.json`  
  Use the scores and reasoning below; you may also read that file for full detail.

---

## Active tasks with complexity (from report)

| ID | Title | Score | Recommended subtasks | Notes from analysis |
|----|--------|-------|----------------------|----------------------|
| 14 | Implement large file detection and warning toast | 3 | 0 | Straightforward: metadata + size check + toast. |
| 15 | Implement wikilinks parsing, resolution, and navigation | 8 | 4 | Parser extension, path resolution, tie-breakers, ambiguity, spaces, multi-system integration. |
| 16 | Create backlinks panel with graph-based indexing | 9 | 5 | Link parsing, graph index, incremental updates, UI, >50 files scaling, reactivity. |
| 17 | Refactor flowchart into modular components with rendering cache | 7 | 3 | Module split + texture cache + behavior preservation; deep Mermaid knowledge. |
| 18 | Implement native macOS window controls and icon polish | 4 | 1 | eframe/egui config + theme/icon checks; platform-specific. |
| 19 | Implement lazy CSV row parsing with byte-offset indexing | 7 | 3 | Offset index, virtual scroll, large-file memory; CSV + UI. |
| 20 | Add Vim mode settings toggle and status bar indicator | 4 | 0 | Settings + status bar; minimal logic. |
| 21 | Implement Vim mode core modal state machine and key handling | 9 | 5 | Normal/Insert/Visual, key handling, egui shortcut precedence, editor integration. |
| 25 | Open file in current window as tab (not new window) | 6 | 2 | File tree + single-instance / OS associations; cross-platform. |
| 26 | Windows MSI: optional file associations with user choice | 5 | 1 | WiX UI + conditional ProgId registration. |
| 27 | Fix images not displaying in rendered markdown view | 5 | 2 | Path resolution, image load, egui Image, formats, errors. |
| 28 | Add German and Japanese to language settings | 2 | 0 | Enum + locale wiring; trivial. |

---

## Suggested model tiers (for you to define or use)

Define 2–4 tiers that match your available models, for example:

- **High** – Complex parsing, state machines, refactors, cross-platform, many integration points.
- **Medium** – Moderate logic, several files, some platform or format handling.
- **Low** – Simple settings, toasts, single-module changes.

If unsure between two tiers for a task, **pick the higher one**.

---

## What to produce

1. **Per-task recommendation**: Task ID, title, suggested tier, one-line reason.
2. **Optional**: A short “when to use high vs medium vs low” summary based on this set.
3. **Optional**: Which tasks (if any) should be expanded into subtasks before implementation (you can use the “Recommended subtasks” column as a hint).

---

## Files to reference if needed

- Task list: `.taskmaster/tasks/tasks.json`
- Complexity report: `.taskmaster/reports/task-complexity-report.json`
- Project rules: `.cursor/rules/ferrite.mdc`, `docs/ai-context.md`
