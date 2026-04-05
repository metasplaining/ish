# Update Agents

Maintain `AGENTS.md` for staleness, broken references, missing skill entries, and line count. Rewrites AGENTS.md and presents a diff for human confirmation before saving.

## When to Use

Run this skill after any change that:
- Adds or removes a skill from `.agents/skills/`
- Renames a crate in `proto/Cargo.toml`
- Adds new top-level doc sections to `docs/`

## Procedure

1. Read current `AGENTS.md`.
2. Read `docs/INDEX.md`, list `.agents/skills/`, and read `proto/Cargo.toml`. If `docs/INDEX.md` is missing, report the gap to the human and abort.
3. For each file reference in `AGENTS.md`, verify the file exists.
4. For each skill directory in `.agents/skills/`, verify it appears in the Task Playbooks table in `AGENTS.md`.
5. For each crate in `proto/Cargo.toml`, verify it appears in the Prototype Crate Map table.
6. Check that the Never Touch list is current.
7. Count lines; note if over 500.
8. For each line, evaluate "Would the agent make a mistake without this?" — flag lines that seem redundant. When uncertain, keep the line.
9. Produce a proposed replacement `AGENTS.md`.
10. Present the diff to the human.
11. If the human confirms, save. Otherwise discard.

## Issues

- **Conservative pruning**: Apply "Would the agent make a mistake without this?" conservatively. When uncertain, keep the line.
- **Line count**: 500 is a target, not a hard limit. Flag if exceeded but do not auto-prune to meet the target.
- **Trigger timing**: Document in `CONTRIBUTING.md` that this skill should be run after the changes listed in "When to Use" above.
