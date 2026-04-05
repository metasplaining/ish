# Phase 4: /update-agents Skill

*Part of: [agent-infrastructure/overview.md](overview.md)*

## Context Files

- [context/update-agents-procedure.md](context/update-agents-procedure.md) — full skill procedure (11 steps), issues, trigger documentation note

## Requirements

- `.agents/skills/update-agents/SKILL.md` exists and contains the 11-step procedure.
- Step 2 of the procedure specifies: if `docs/INDEX.md` is missing, report the gap to the human and abort.
- The skill is accessible via `.claude/skills/update-agents/SKILL.md` (through the symlink from Phase 2).

## Tasks

- [x] 1. Create `.agents/skills/update-agents/` directory and write `SKILL.md` — `.agents/skills/update-agents/SKILL.md`

  Write a SKILL.md that includes:
  - A "When to Use" section: after any change that adds/removes skills, renames crates, or adds new doc sections.
  - A "Procedure" section with the 11-step procedure from `context/update-agents-procedure.md`.
    - Step 2 must explicitly state: "If `docs/INDEX.md` is missing, report the gap to the human and abort."
  - An "Issues" section with the three notes from the context file.
  - Standard SKILL.md formatting (consistent with existing skills in `.agents/skills/`).

## Verification

Run: `cat .agents/skills/update-agents/SKILL.md`
Check: file exists, contains 11 numbered steps, step 2 mentions aborting if `docs/INDEX.md` is missing.

Run: `cat .claude/skills/update-agents/SKILL.md | head -5`
Check: same content (symlink from Phase 2 resolves correctly).

Invoke: `/verify agent-infrastructure/phase-4.md`
