# Phase 1: Roadmap — Mark In Progress

*Part of: [agent-infrastructure/overview.md](overview.md)*

## Requirements

- `docs/project/roadmap.md` lists "Agent Infrastructure" under the "In Progress" section before implementation begins.

## Tasks

- [x] 1. Add "Agent Infrastructure" to the In Progress section of `docs/project/roadmap.md` — `docs/project/roadmap.md`

  Add this line under `### In Progress`:
  ```
  - [ ] Agent Infrastructure (vendor-agnostic file layout, AGENTS.md rewrite, /update-agents skill)
  ```

## Verification

Run: `grep -n "Agent Infrastructure" docs/project/roadmap.md`
Check: line appears under the `### In Progress` heading.

Invoke: `/verify agent-infrastructure/phase-1.md`
