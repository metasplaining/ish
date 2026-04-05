---
title: "Phase 4: Implement Skill"
category: plan-phase
audience: [ai-dev]
status: ready
last-verified: 2026-04-04
---

# Phase 4: Implement Skill

*Part of: [agent-pipeline-skills/overview.md](overview.md)*

## Context Files

- [context/implement-skill.md](context/implement-skill.md) — guard rail step 4 specification
- [context/plan-skill.md](context/plan-skill.md) — §Implement Skill Adaptation (phase-directory handling)
- [context/verify-skill.md](context/verify-skill.md) — §Integration with Implement (Verify invocation)
- [context/backchannel.md](context/backchannel.md) — clarification request format, blocked plan item convention, output status

## Requirements

- Before executing each TODO item, the skill verifies it is unambiguous and achievable.
- If unclear: re-read spec/proposal, then if still unclear: write clarification request, mark item BLOCKED, stop.
- If impossible: try one alternative interpretation, then if still impossible: write clarification request, mark item BLOCKED, stop.
- The skill never continues to subsequent items after blocking.
- The skill detects whether the plan is a single file or a directory.
- For directory plans: reads `overview.md` first, then executes phases in order, invoking `/verify` after each phase.
- The skill appends `Status: completed | blocked (see clarifications/<date>-<topic>.md)` to its output.

## Tasks

- [ ] 10. Update step 4 in implement skill with guard rails — `.claude/skills/implement/SKILL.md`

  Replace the current step 4 with the guard-rail version from [context/implement-skill.md](context/implement-skill.md) §Updated Implement Skill Step 4. The new step 4 reads:

  **Step 4 — Execute TODO items in order:**

  Before executing each TODO item, verify that the item is unambiguous and achievable.

  **If a task is unclear:**
  1. Re-read the relevant specification section and proposal section.
  2. If still unclear: do not attempt the task. Write a clarification request at `docs/project/clarifications/<date>-<topic>.md`. Mark the plan item `BLOCKED: see clarifications/<date>-<topic>.md`. Stop.

  **If a task appears impossible:**
  1. Verify the task requirements against the accepted proposal decisions.
  2. Try one alternative interpretation.
  3. If still impossible: write a clarification request. Mark the plan item `BLOCKED`. Stop.

  **Do not continue to subsequent items after blocking.**

  (Retain the existing substeps a–e for the normal execution path.)

- [ ] 11. Add phase-directory detection to implement skill — `.claude/skills/implement/SKILL.md`

  Update step 1 (or add before step 1) to detect plan format:

  **Step 1 — Read the implementation plan:**
  - If the path points to a `.md` file: single-file plan. Current behavior.
  - If the path points to a directory: read `overview.md` first. Note the phase order from the Phase Dependency Graph.

  Update step 4 to add, for directory plans: after completing all tasks in a phase, proceed to the next phase in the order specified in `overview.md`.

- [ ] 12. Add Verify invocation at each phase checkpoint — `.claude/skills/implement/SKILL.md`

  Update step 4d (checkpoint items) for directory plans:

  > At the end of each phase in a directory plan, invoke `/verify <plan-name>/phase-N.md` before continuing to the next phase. Do not proceed to the next phase until Verify reports clean (output contains "Verified: no inconsistencies found").

- [ ] 13. Add blocked plan item format and output status to implement skill — `.claude/skills/implement/SKILL.md`

  Add to the skill (as a note or new section):

  **Blocked plan items** are marked:
  ```
  - [x] BLOCKED 5. <task> — see docs/project/clarifications/<date>-<topic>.md
  ```

  Each invocation appends a status line to its output:
  ```
  Status: completed | blocked (see clarifications/<date>-<topic>.md)
  ```

## Verification

Run: `grep -n "unclear\|impossible\|BLOCKED\|clarification\|directory\|overview\.md\|verify" /home/dan/git/ish/.claude/skills/implement/SKILL.md`
Check: Guard rails, directory handling, Verify invocation, and BLOCKED convention are present.

Run: `grep -n "Status:" /home/dan/git/ish/.claude/skills/implement/SKILL.md`
Check: Status line convention is present.

Invoke: `/verify agent-pipeline-skills/phase-4.md`

Status: completed | blocked (see clarifications/<date>-<topic>.md)
