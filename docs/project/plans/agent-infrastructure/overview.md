---
title: "Plan: Agent Infrastructure"
category: plan
audience: [ai-dev]
status: completed
last-verified: 2026-04-05
depends-on:
  - docs/project/proposals/agent-infrastructure.md
  - AGENTS.md
  - CLAUDE.md
  - .github/copilot-instructions.md
  - CONTRIBUTING.md
---

# Plan: Agent Infrastructure

*Derived from [agent-infrastructure.md](../../proposals/agent-infrastructure.md) on 2026-04-05.*

---

## Summary

Consolidates the project's agent instruction files and skills directories into a vendor-agnostic layout. Nine existing skills move from split vendor directories (`.github/skills/`, `.claude/skills/`) to a canonical `.agents/skills/` directory; both vendor directories become symlinks. `AGENTS.md` becomes the single canonical instruction file; `CLAUDE.md` and `.github/copilot-instructions.md` become symlinks. A tenth skill `/update-agents` is added. `AGENTS.md` is rewritten to merge current content from `AGENTS.md` and `CLAUDE.md` and add new sections. `CONTRIBUTING.md` gains agent-friendly and Rust style guidelines.

---

## Requirements

1. `.agents/skills/` is the canonical skills directory containing all ten skills (nine existing + update-agents).
2. `.claude/skills/` is a symlink to `../.agents/skills`.
3. `.github/skills/` is a symlink to `../.agents/skills`.
4. `AGENTS.md` is the canonical agent instruction file containing all sections per spec.
5. `CLAUDE.md` is a symlink to `AGENTS.md`.
6. `.github/copilot-instructions.md` is a symlink to `../AGENTS.md`.
7. `.agents/skills/update-agents/SKILL.md` contains the 11-step maintenance skill procedure.
8. `/update-agents` step 2 aborts with a report to the human if `docs/INDEX.md` is missing.
9. `CONTRIBUTING.md` contains an Agent-Friendly Style Guidelines section and a Rust Style Guidelines section.

---

## Phase Dependency Graph

```
Phase 1 (Roadmap in-progress)
    ↓
Phase 2 (File layout migration)
    ↓
Phase 3 (AGENTS.md rewrite + symlinks)
    ↓
Phase 4 (/update-agents skill)
    ↓
Phase 5 (CONTRIBUTING.md extensions)
    ↓
Phase 6 (Roadmap completed + history + indexes)
```

Phases are sequential. Each phase depends on the previous completing successfully.

---

## Context Files

- [context/agents-md-content-spec.md](context/agents-md-content-spec.md) — Content specification for the new AGENTS.md (10 sections, issues, tech stack data)
- [context/update-agents-procedure.md](context/update-agents-procedure.md) — Full `/update-agents` skill procedure (11 steps + issues)
- [context/contributing-extensions.md](context/contributing-extensions.md) — Two style guideline sections to add to CONTRIBUTING.md

---

## Phases

| Phase | Topic | File |
|-------|-------|------|
| 1 | Roadmap: mark in progress | [phase-1.md](phase-1.md) |
| 2 | File layout migration | [phase-2.md](phase-2.md) |
| 3 | AGENTS.md rewrite + symlinks | [phase-3.md](phase-3.md) |
| 4 | `/update-agents` skill | [phase-4.md](phase-4.md) |
| 5 | CONTRIBUTING.md extensions | [phase-5.md](phase-5.md) |
| 6 | Roadmap completed + history + indexes | [phase-6.md](phase-6.md) |

---

## Reference

**Symlink path correctness**: The symlink targets are relative to the directory containing the symlink (not the repo root). The correct paths are:
- `.claude/skills → ../.agents/skills` (`.claude/` is one level from root)
- `.github/skills → ../.agents/skills` (`.github/` is one level from root)
- `.github/copilot-instructions.md → ../AGENTS.md` (`.github/` is one level from root)
- `CLAUDE.md → AGENTS.md` (both at repo root — no path prefix needed)

**Pre-migration skill inventory**:
- `.github/skills/`: propose, accept, audit, revise, plan-implementation, implement (6 skills)
- `.claude/skills/`: bug-fix, troubleshoot, verify (3 skills)
- Total: 9 skills to migrate; 1 new skill (update-agents) to create in Phase 4

**Tech stack versions** (for AGENTS.md Tech Stack section):
- Rust edition: 2021
- Tokio: version 1 (workspace), `rt-multi-thread`/`macros` features in ish-shell; runtime: `Runtime::new_current_thread` with `LocalSet`
- pest: ~2.7; grammar at `proto/ish-parser/src/ish.pest`
- Reedline: 0.46

**Current CLAUDE.md location**: The current `CLAUDE.md` contains only the Proposal Process, Authority Order, Implementation Discipline, and Resuming Implementation sections (46 lines). Its full content merges into AGENTS.md §8 Proposal Process in Phase 3.
