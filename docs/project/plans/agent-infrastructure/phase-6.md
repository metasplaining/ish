# Phase 6: Roadmap Completed + History + Index Files

*Part of: [agent-infrastructure/overview.md](overview.md)*

## Requirements

- `docs/project/roadmap.md` shows "Agent Infrastructure" under the Completed section.
- `docs/project/history/2026-04-04-agent-instructions-improvements/summary.md` contains an implementation narrative.
- `docs/project/plans/INDEX.md` includes an entry for this plan (status: completed).
- `docs/INDEX.md` references `.agents/skills/` as the canonical skills location.

## Tasks

- [x] 1. Move "Agent Infrastructure" from In Progress to Completed in `docs/project/roadmap.md` — `docs/project/roadmap.md`

  Remove the In Progress item added in Phase 1 and add to the `### Completed` list:
  ```
  - [x] Agent Infrastructure (vendor-agnostic file layout, AGENTS.md rewrite, /update-agents skill)
  ```

- [x] 2. Append implementation narrative to `docs/project/history/2026-04-04-agent-instructions-improvements/summary.md` — `docs/project/history/2026-04-04-agent-instructions-improvements/summary.md`

  Write a `## Implementation: Agent Infrastructure` section. Summarize what was done across all six phases: file layout migration (9 skills to .agents/skills/, 2 vendor symlinks), AGENTS.md rewrite (merged CLAUDE.md content, added 4 new sections), CLAUDE.md and copilot-instructions.md replaced with symlinks, /update-agents skill created, CONTRIBUTING.md extended with agent-friendly and Rust style guidelines.

- [x] 3. Add an entry for this plan to `docs/project/plans/INDEX.md` — `docs/project/plans/INDEX.md`

  Add to the table (at the top, as newest entry):
  ```
  | 2026-04-05 | Agent Infrastructure | completed | [agent-infrastructure/overview.md](agent-infrastructure/overview.md) |
  ```

- [x] 4. Update `docs/INDEX.md` to note `.agents/skills/` as the canonical skills directory — `docs/INDEX.md`

  In the Root Files table, add a row for `.agents/skills/`:
  ```
  | [.agents/skills/](../.agents/skills/) | Canonical agent skill files (vendor dirs are symlinks) |
  ```

## Verification

Run: `grep -n "Agent Infrastructure" docs/project/roadmap.md`
Check: appears under `### Completed` (with `[x]`), not under In Progress.

Run: `grep -n "Agent Infrastructure" docs/project/plans/INDEX.md`
Check: entry exists with status "completed".

Run: `grep -n "agents/skills" docs/INDEX.md`
Check: `.agents/skills/` appears in the Root Files table.

Invoke: `/verify agent-infrastructure/phase-6.md`
