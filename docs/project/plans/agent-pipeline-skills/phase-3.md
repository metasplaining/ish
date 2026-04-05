---
title: "Phase 3: Plan Skill"
category: plan-phase
audience: [ai-dev]
status: ready
last-verified: 2026-04-04
---

# Phase 3: Plan Skill

*Part of: [agent-pipeline-skills/overview.md](overview.md)*

## Context Files

- [context/plan-skill.md](context/plan-skill.md) — full feature specification for phase directories and context files
- [context/backchannel.md](context/backchannel.md) — clarification request format and output status convention

## Requirements

- For > 5 steps: the skill produces `docs/project/plans/<name>/` with `overview.md`, `context/`, and `phase-N.md` files.
- For ≤ 5 steps: the skill produces `docs/project/plans/<name>.md` (current behavior unchanged).
- Phase files follow the format in [context/plan-skill.md](context/plan-skill.md) §Phase File Format.
- Context files contain verbatim extracts, each with a source attribution header.
- Before saving, the skill scrutinizes each phase for completeness. If any phase fails: write a clarification request and stop without saving.
- The skill appends `Status: completed | blocked (see clarifications/<date>-<topic>.md)` to its output.

## Tasks

- [ ] 6. Add phase-directory structure to plan skill — `.claude/skills/plan-implementation/SKILL.md`

  Replace the current step 8 ("Save to `docs/project/plans/<name>.md`") with a conditional:

  **Step 8 — Save:**
  - If the plan has ≤ 5 implementation steps: save to `docs/project/plans/<name>.md` (current behavior).
  - If the plan has > 5 implementation steps: save to a directory:
    ```
    docs/project/plans/<name>/
      overview.md          — summary, requirements, phase dependency graph
      context/
        <topic>.md         — verbatim extracts from authoritative sources
      phase-1.md
      phase-2.md
      ...
    ```

  Also update the Output Format section in the skill to show both the single-file and directory formats, with the phase file format from [context/plan-skill.md](context/plan-skill.md) §Phase File Format.

- [ ] 7. Add context file rules to plan skill — `.claude/skills/plan-implementation/SKILL.md`

  Add a new step before step 8 (saving), or as a substep of step 6 (generate TODO list):

  **Context File Rules:**
  - Content is verbatim from the authoritative source (spec, architecture doc, or proposal).
  - Each context file begins with: `*Extracted verbatim from [source](path/to/source.md) §Section Name.*`
  - Do not paraphrase or summarize.
  - If a context file requires information from a source that does not exist or is incomplete, treat this as a gap: add to a gap list and do not proceed to saving.

- [ ] 8. Add gap-detection-before-saving (scrutiny) step — `.claude/skills/plan-implementation/SKILL.md`

  Add a new step after generating all phase files and before saving:

  **Step 7 (new) — Scrutiny:**
  Before saving, check each phase:
  - Does each task have enough detail for an agent to execute without reading the original spec?
  - Does each verification step have a concrete command to run?
  - Is every context file available and non-empty?

  If any phase fails: write a clarification request at `docs/project/clarifications/<date>-<topic>.md` using the format in [context/backchannel.md](context/backchannel.md). Do not save the incomplete plan. Output `Status: blocked (see clarifications/<date>-<topic>.md)` and stop.

- [ ] 9. Add output status line to plan skill — `.claude/skills/plan-implementation/SKILL.md`

  Add at the end of the Procedure section:

  Each invocation appends a status line to its output:
  ```
  Status: completed | blocked (see clarifications/<date>-<topic>.md)
  ```

## Verification

Run: `grep -n "directory\|context/\|phase-\|scrutin\|≤ 5\|> 5" /home/dan/git/ish/.claude/skills/plan-implementation/SKILL.md`
Check: Directory structure, context file rules, and scrutiny step are present.

Run: `grep -n "Status:" /home/dan/git/ish/.claude/skills/plan-implementation/SKILL.md`
Check: Status line convention is present.

Invoke: `/verify agent-pipeline-skills/phase-3.md`

Status: completed | blocked (see clarifications/<date>-<topic>.md)
