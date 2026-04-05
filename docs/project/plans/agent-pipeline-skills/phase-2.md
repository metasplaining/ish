---
title: "Phase 2: Revise Skill"
category: plan-phase
audience: [ai-dev]
status: ready
last-verified: 2026-04-04
---

# Phase 2: Revise Skill

*Part of: [agent-pipeline-skills/overview.md](overview.md)*

## Context Files

- [context/revise-skill.md](context/revise-skill.md) — full feature specification for gap detection and split evaluation
- [context/backchannel.md](context/backchannel.md) — clarification request format and output status convention

## Requirements

- After incorporating the punch list, the Revise skill applies the gap detection checklist to each feature section.
- The gap detection checklist checks: error cases specified, concurrency behavior addressed, testability statement present. Backward-compatibility is explicitly excluded.
- For each gap found, the skill injects an Open Question with `-->` marker.
- After gap detection, the skill counts extractable independent steps and presents a split decision if ≥ 10.
- The skill appends `Status: completed | blocked (see clarifications/<date>-<topic>.md)` to its output.

## Tasks

- [ ] 3. Add Step 6a (Gap Detection) to revise skill — `.claude/skills/revise/SKILL.md`

  Insert a new step between the current step 6 ("Scan the entire document for internal contradictions") and step 7 ("Save the replacement"). Number it step 6a. Content verbatim from [context/revise-skill.md](context/revise-skill.md) §Updated Revise Skill Procedure → Step 6a.

  The new step reads:

  **Step 6a — Gap Detection:**
  1. For each feature section, apply the gap detection checklist:
     - Are the error cases specified?
     - Is concurrency behavior addressed (if the feature touches async code)?
     - Is there a testability statement (what acceptance test would verify this)?
     - Note: backward-compatibility analysis is explicitly excluded.
  2. For each uncovered item, inject:
     ```
     **Open Question:** [specific description of the gap]
     -->
     ```
  3. If any open questions were added, note this in the summary.

- [ ] 4. Add Step 6b (Split Evaluation) to revise skill — `.claude/skills/revise/SKILL.md`

  Insert immediately after step 6a (before step 7). Number it step 6b. Content verbatim from [context/revise-skill.md](context/revise-skill.md) §Updated Revise Skill Procedure → Step 6b.

  The new step reads:

  **Step 6b — Split Evaluation:**
  1. Count extractable, independent implementation steps.
  2. If ≥ 10: draft a split proposal and present it as a decision point:
     ```
     **Decision:** Split into Proposal A ([list features]) and Proposal B ([list features])?
     -->
     ```
     Also note: if the human accepts the split, create new proposal files, update the history directory, and mark the parent proposal as split in its frontmatter.
  3. If < 10: continue with the single proposal.

- [ ] 5. Add output status line to revise skill — `.claude/skills/revise/SKILL.md`

  Add at the end of the Procedure section (after step 9, or as a new §Output section):

  Each invocation appends a status line to its output:
  ```
  Status: completed | blocked (see clarifications/<date>-<topic>.md)
  ```

## Verification

Run: `grep -n "6a\|6b\|Gap Detection\|Split Evaluation\|gap detection\|split" /home/dan/git/ish/.claude/skills/revise/SKILL.md`
Check: Steps 6a and 6b appear with gap detection checklist and split evaluation logic.

Run: `grep -n "Status:" /home/dan/git/ish/.claude/skills/revise/SKILL.md`
Check: Status line convention is present.

Invoke: `/verify agent-pipeline-skills/phase-2.md`

Status: completed | blocked (see clarifications/<date>-<topic>.md)
