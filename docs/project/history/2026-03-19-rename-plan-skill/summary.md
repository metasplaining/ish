---
title: "History: Rename /plan Skill"
category: project
audience: [all]
status: draft
last-verified: 2026-03-19
depends-on: [docs/project/proposals/rename-plan-skill.md]
---

# History: Rename /plan Skill

## Summary

The `/plan` skill, created as part of the proposal process improvements on 2026-03-18, was found to conflict with VS Code Copilot's builtin "plan" feature. When a user invoked `/plan` in the Copilot chat, the builtin plan mode could intercept the command instead of loading the project's custom skill.

A brief proposal was drafted on 2026-03-19 evaluating four alternative names: `/plan_implementation` (underscore), `/plan-implementation` (hyphen), `/planimpl` (abbreviated), and `/implementation_plan` (reversed word order). The human chose `/plan-implementation`, noting that hyphens are the Copilot convention for multi-word skill names.

Two additional decisions were made without debate: history files from the original proposal process improvements work would be left untouched as historical records, and the completed proposal and plan documents for that earlier feature would also be left unchanged. Only the live, actively-referenced documents — `AGENTS.md`, `docs/ai-guide/orientation.md`, and the skill's own `SKILL.md` — would be updated.

The proposal was accepted immediately and an implementation plan generated on the same day. Implementation was completed on 2026-03-19: the skill directory was renamed from `.github/skills/plan/` to `.github/skills/plan-implementation/`, the SKILL.md frontmatter and trigger words were updated, and references in `AGENTS.md` and `docs/ai-guide/orientation.md` were changed to point to the new path.

---

## Versions

| Version | Description |
|---------|-------------|
| [v1.md](v1.md) | Initial proposal with all alternatives and three pending decisions |

---

## Referenced by

- [docs/project/history/INDEX.md](../INDEX.md)
