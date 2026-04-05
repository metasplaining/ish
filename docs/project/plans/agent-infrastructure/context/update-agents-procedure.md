*Extracted verbatim from [agent-infrastructure.md](../../proposals/agent-infrastructure.md) §Feature: AGENTS.md Maintenance Skill.*

---

## What Changes

A new skill `/update-agents` checks AGENTS.md for staleness, broken references, missing skill entries, and line count. It rewrites AGENTS.md and presents the diff for human confirmation before saving.

## Issues to Watch Out For

- **Conservative pruning**: The skill must apply "Would the agent make a mistake without this?" conservatively. When uncertain, keep the line.
- **Line count**: 500 is a target, not a hard limit. The skill flags if exceeded but does not auto-prune to meet the target.
- **Trigger timing**: Run this skill after any change that adds/removes skills, renames crates, or adds new doc sections. Document this in CONTRIBUTING.md.

## Skill File Location

New skill: `.agents/skills/update-agents/SKILL.md`

## Procedure (to be written into the skill file)

1. Read current AGENTS.md.
2. Read `docs/INDEX.md`, the skills directory listing, and `proto/Cargo.toml` (crate list). If `docs/INDEX.md` is missing, report the gap to the human and abort.
3. For each file reference in AGENTS.md, verify the file exists.
4. For each skill in `.agents/skills/`, verify it appears in the Task Playbooks table.
5. For each crate in `proto/Cargo.toml`, verify it appears in the Prototype Crate Map table.
6. Check that the Never Touch list is current.
7. Count lines; note if over 500.
8. For each line, evaluate "Would the agent make a mistake without this?" — flag lines that seem redundant.
9. Produce a proposed replacement AGENTS.md.
10. Present the diff to the human.
11. If the human confirms, save. Otherwise discard.

## Trigger Documentation (to be added to CONTRIBUTING.md)

Run `/update-agents` after any change that adds or removes skills, renames crates, or adds new doc sections.
