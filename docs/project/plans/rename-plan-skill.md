---
title: "Plan: Rename /plan Skill to /plan-implementation"
category: plan
audience: [ai-dev]
status: completed
last-verified: 2026-03-19
depends-on: [docs/project/proposals/rename-plan-skill.md, .github/skills/plan/SKILL.md, AGENTS.md, docs/ai-guide/orientation.md]
---

# Plan: Rename /plan Skill to /plan-implementation

*Derived from [rename-plan-skill.md](../proposals/rename-plan-skill.md) on 2026-03-19.*

## Overview

Rename the `/plan` skill to `/plan-implementation` to avoid a naming conflict with VS Code Copilot's builtin "plan" feature. This involves renaming the skill directory, updating the SKILL.md contents, and updating all actively-referenced documents. Historical documents are left unchanged.

## Requirements

- R1: The skill directory is renamed from `.github/skills/plan/` to `.github/skills/plan-implementation/`.
- R2: The SKILL.md `name:` field is `plan-implementation`.
- R3: The SKILL.md description and trigger words do not contain the bare word "plan" as a standalone trigger.
- R4: All references in `AGENTS.md` point to `.github/skills/plan-implementation/SKILL.md`.
- R5: All references in `docs/ai-guide/orientation.md` use `/plan-implementation`.
- R6: History files under `docs/project/history/2026-03-18-proposal-process-improvements/` are not modified.
- R7: `docs/project/proposals/proposal-process-improvements.md`, `docs/project/plans/proposal-process-improvements.md`, and `audit-proposal-process-improvements.md` are not modified.

## Authority Order

This change does not introduce new terms, specifications, architecture, or code. The relevant subset of the standard authority order:

1. User guide / AI guide (`docs/ai-guide/orientation.md`)
2. Agent documentation (`AGENTS.md`, skill SKILL.md)
3. History
4. Index files

## TODO

- [x] 1. ~~**Rename skill directory** — `mv .github/skills/plan/ .github/skills/plan-implementation/`~~
- [x] 2. ~~**Update SKILL.md contents** — `.github/skills/plan-implementation/SKILL.md`~~
  - Change `name: plan` → `name: plan-implementation`
  - Change description trigger words: replace `"plan, implementation plan, ready to implement"` with `"plan-implementation, implementation plan, ready to implement"`
  - Change `/plan <proposal-path>` → `/plan-implementation <proposal-path>` in the "When to Use" section
  - Change `/accept` cross-reference wording if it mentions `/plan` by name
- [x] 3. ~~**Update docs/ai-guide/orientation.md** — change `/plan` → `/plan-implementation` in the skill list~~
- [x] 4. ~~**Update AGENTS.md** — change `.github/skills/plan/SKILL.md` → `.github/skills/plan-implementation/SKILL.md` in the Task Playbooks table~~
- [x] 5. ~~**CHECKPOINT** — verify R1–R5 by reading the updated files~~
- [x] 6. ~~**Update history** — append to `docs/project/history/2026-03-19-rename-plan-skill/summary.md` noting implementation completion~~
- [x] 7. ~~**CHECKPOINT** — verify R6–R7 by confirming historical files were not modified~~

## Reference

### Original file locations

| Original path | New path |
|---------------|----------|
| `.github/skills/plan/SKILL.md` | `.github/skills/plan-implementation/SKILL.md` |

### Original SKILL.md frontmatter

```yaml
name: plan
description: 'Generate an implementation plan from a design proposal. Use when: a design proposal has been accepted and is ready for implementation. Creates a consolidated TODO with authority-ordered file changes. Trigger words: plan, implementation plan, ready to implement.'
argument-hint: 'Path to the design proposal'
```

### Files that must NOT be modified

- `docs/project/history/2026-03-18-proposal-process-improvements/v1.md`
- `docs/project/history/2026-03-18-proposal-process-improvements/v2.md`
- `docs/project/history/2026-03-18-proposal-process-improvements/v3.md`
- `docs/project/history/2026-03-18-proposal-process-improvements/summary.md`
- `docs/project/proposals/proposal-process-improvements.md`
- `docs/project/plans/proposal-process-improvements.md`
- `audit-proposal-process-improvements.md`

---

## Referenced by

- [docs/project/proposals/rename-plan-skill.md](../proposals/rename-plan-skill.md)
