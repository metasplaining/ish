---
title: "Phase 5: New Skills and Infrastructure"
category: plan-phase
audience: [ai-dev]
status: ready
last-verified: 2026-04-04
---

# Phase 5: New Skills and Infrastructure

*Part of: [agent-pipeline-skills/overview.md](overview.md)*

## Context Files

- [context/bug-fix-skill.md](context/bug-fix-skill.md) — full Bug Fix skill specification
- [context/troubleshoot-skill.md](context/troubleshoot-skill.md) — full Troubleshoot skill specification
- [context/verify-skill.md](context/verify-skill.md) — full Verify skill specification
- [context/backchannel.md](context/backchannel.md) — clarification request format, INDEX.md, output status

## Requirements

- `.claude/skills/bug-fix/SKILL.md` exists with the 15-step procedure from the proposal.
- `.claude/skills/troubleshoot/SKILL.md` exists with the 14-step procedure from the proposal.
- `.claude/skills/verify/SKILL.md` exists with the 7-step procedure from the proposal.
- `docs/project/clarifications/INDEX.md` exists with a table for tracking open clarification requests.
- All three new skills append `Status: completed | blocked (see clarifications/<date>-<topic>.md)` to their output.

## Tasks

- [x] 14. Create Bug Fix skill — `.claude/skills/bug-fix/SKILL.md`
  Bug fix 2026-04-04: Added Clarification Request Format section; updated steps 7–8 to write clarification files using the format and update INDEX.md.

  Create the file. Content: SKILL.md frontmatter + the full 15-step procedure from [context/bug-fix-skill.md](context/bug-fix-skill.md).

  Frontmatter:
  ```yaml
  ---
  name: bug-fix
  description: 'Fix a confirmed bug across all artifact types. Use when: root cause is identified and the behavior needs to be corrected in code, tests, and docs. Trigger words: bug fix, fix bug, fix confirmed bug.'
  argument-hint: 'Path to the bug report or clarification document with root cause'
  ---
  ```

  Include the Documentation Coverage Rule table, the 15-step Skill Procedure, Issues to Watch Out For section, and status line convention.

- [x] 15. Create Troubleshoot skill — `.claude/skills/troubleshoot/SKILL.md`
  Bug fix 2026-04-04: Added Clarification Request Format section; updated steps 12–13 to use the format and update INDEX.md.

  Create the file. Content: SKILL.md frontmatter + the full 14-step procedure from [context/troubleshoot-skill.md](context/troubleshoot-skill.md).

  Frontmatter:
  ```yaml
  ---
  name: troubleshoot
  description: 'Identify the root cause of a problem without fixing it. Use when: a bug is reported but root cause is unknown. Produces a clarification document for handoff to Bug Fix. Trigger words: troubleshoot, diagnose, root cause, investigate bug.'
  argument-hint: 'Path to the problem report'
  ---
  ```

  Include the Specification Evaluation section, the 14-step Skill Procedure, Issues to Watch Out For section, and status line convention.

- [x] 16. Create Verify skill — `.claude/skills/verify/SKILL.md`
  Bug fix 2026-04-04: Added Clarification Request Format section; updated step 6 to use the format and update INDEX.md.

  Create the file. Content: SKILL.md frontmatter + the full 7-step procedure from [context/verify-skill.md](context/verify-skill.md).

  Frontmatter:
  ```yaml
  ---
  name: verify
  description: 'Check post-implementation consistency for changed artifacts. Use when: a phase or bug fix is complete and needs consistency verification. Invoked automatically by implement at each checkpoint. Trigger words: verify, check consistency, post-implementation check.'
  argument-hint: 'Path to the phase file or bug fix description being verified'
  ---
  ```

  Include the Consistency Checks table, the 7-step Skill Procedure, Integration with Implement section, Issues to Watch Out For section, and status line convention.

- [ ] 17. Create clarifications index — `docs/project/clarifications/INDEX.md`

  Create the file with frontmatter and an empty table:

  ```markdown
  ---
  title: Clarification Requests Index
  category: project
  audience: [ai-dev, human]
  status: stable
  last-verified: 2026-04-04
  ---

  # Clarification Requests

  Structured requests written by blocked agents. Each file follows the format in the Backchannel Communication spec.

  | Date | Slug | Created by | Status | File |
  |------|------|-----------|--------|------|

  ---

  ## Cleanup

  After resolution, delete the clarification file and add a one-line note to the relevant history file. Remove the row from this index.
  ```

## Verification

Run: `ls /home/dan/git/ish/.claude/skills/bug-fix/ /home/dan/git/ish/.claude/skills/troubleshoot/ /home/dan/git/ish/.claude/skills/verify/`
Check: All three SKILL.md files exist.

Run: `grep -c "^[0-9]" /home/dan/git/ish/.claude/skills/bug-fix/SKILL.md /home/dan/git/ish/.claude/skills/troubleshoot/SKILL.md /home/dan/git/ish/.claude/skills/verify/SKILL.md`
Check: Counts indicate numbered procedure steps are present (bug-fix: ~15, troubleshoot: ~14, verify: ~7).

Run: `cat /home/dan/git/ish/docs/project/clarifications/INDEX.md`
Check: File exists with the table structure.

Invoke: `/verify agent-pipeline-skills/phase-5.md`

Status: completed | blocked (see clarifications/<date>-<topic>.md)
