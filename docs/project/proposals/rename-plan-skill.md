---
title: "Proposal: Rename /plan Skill to Avoid Copilot Builtin Conflict"
category: proposal
audience: [ai-dev]
status: accepted
last-verified: 2026-03-19
depends-on: [docs/project/rfp/rename-plan-skill.md, .github/skills/plan/SKILL.md, AGENTS.md, docs/ai-guide/orientation.md]
---

# Proposal: Rename /plan Skill to Avoid Copilot Builtin Conflict

*Generated from [rename-plan-skill.md](../rfp/rename-plan-skill.md) on 2026-03-19.*

---

## Decision Register

All decisions made during design, consolidated here as the authoritative reference.

| # | Decision | Outcome |
|---|----------|---------|
| 1 | What should the new skill name be? | `/plan-implementation` — hyphens are the Copilot convention |
| 2 | Should historical documents be updated? | No — leave as historical records |
| 3 | Should the completed proposal/plan documents be updated? | No — leave as historical records |

---

## Questions and Answers

### Q: What is the conflict with Copilot's builtin plan feature?

VS Code Copilot has a builtin "plan" mode/feature. When a skill is named `plan`, invoking `/plan` in the Copilot chat may trigger the builtin plan feature instead of (or in addition to) loading the project's custom `/plan` skill. Renaming the skill to a unique name avoids this ambiguity and ensures the custom skill is reliably invoked.

---

## Feature: Rename /plan Skill to /plan-implementation

### Issues to Watch Out For

1. **Many cross-references.** The `/plan` skill is referenced in multiple active documents: `AGENTS.md`, `docs/ai-guide/orientation.md`. All must be updated consistently.

2. **History files contain `/plan` references.** The history directory `docs/project/history/2026-03-18-proposal-process-improvements/` and completed proposal/plan documents reference `/plan`. These are historical records and must not be modified.

3. **Skill YAML frontmatter.** The `name:` field and `description:` field in the SKILL.md frontmatter must be updated. Trigger words must avoid the bare word "plan" to prevent re-triggering the Copilot builtin.

4. **Directory rename.** The skill directory `.github/skills/plan/` must be renamed to `.github/skills/plan-implementation/`.

### Implementation

#### 1. Rename the skill directory and update SKILL.md

- Rename `.github/skills/plan/` → `.github/skills/plan-implementation/`
- In `.github/skills/plan-implementation/SKILL.md`:
  - Update `name: plan` → `name: plan-implementation`
  - Update the description's trigger words to replace bare "plan" with "plan-implementation" or "implementation plan"
  - Update references to `/plan` → `/plan-implementation` in the procedure body

#### 2. Update AGENTS.md

- Update the Task Playbooks table: change `.github/skills/plan/SKILL.md` → `.github/skills/plan-implementation/SKILL.md`

#### 3. Update docs/ai-guide/orientation.md

- Update the skill list from `/plan` to `/plan-implementation`

#### 4. Do NOT update historical documents

The following are historical records and must not be modified:
- `docs/project/history/2026-03-18-proposal-process-improvements/v1.md`, `v2.md`, `v3.md`, `summary.md`
- `docs/project/proposals/proposal-process-improvements.md`
- `docs/project/plans/proposal-process-improvements.md`
- `audit-proposal-process-improvements.md`

---

## Documentation Updates

| File | What changes |
|------|-------------|
| `.github/skills/plan/SKILL.md` | Renamed directory + updated name, description, trigger words, body |
| `AGENTS.md` | Task Playbooks table entry |
| `docs/ai-guide/orientation.md` | Skill list in Proposal Process section |

## History Updates

- [ ] Create `docs/project/history/2026-03-19-rename-plan-skill/` directory
- [ ] Add `summary.md` with narrative prose describing the rename and its motivation
- [ ] Update `docs/project/history/INDEX.md`

---

## Referenced by

- [docs/project/proposals/INDEX.md](INDEX.md)
