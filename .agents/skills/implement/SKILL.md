---
name: implement
description: 'Execute an implementation plan following authority order with checkpoint verification. Use when: an implementation plan exists and is ready to execute. Trigger words: implement, build, execute plan, start implementation.'
argument-hint: 'Path to the implementation plan'
---

# Implement

Execute an implementation plan following authority order with checkpoint verification. Track progress by marking TODO items in the plan.

## When to Use

- An implementation plan exists with status "ready"
- The user wants to begin or continue implementation
- The user invokes `/implement <plan-path>`

## Procedure

1. **Read the implementation plan.**
   - If the path points to a `.md` file: single-file plan. Proceed as normal.
   - If the path points to a directory: read `overview.md` first. Note the phase order from the Phase Dependency Graph. Then execute phases in the order specified in `overview.md`.

2. **Verify the design proposal is accepted** and the plan status is "ready" (or "in-progress" if resuming).

3. **Set the plan status to "in-progress."**

4. **Execute TODO items in order:**

   Before executing each TODO item, verify that the item is unambiguous and achievable.

   **If a task is unclear:**
   1. Re-read the relevant specification section and proposal section.
   2. If still unclear: do not attempt the task. Write a clarification request at `docs/project/clarifications/<date>-<topic>.md`. Mark the plan item `BLOCKED: see clarifications/<date>-<topic>.md`. Stop.

   **If a task appears impossible:**
   1. Verify the task requirements against the accepted proposal decisions.
   2. Try one alternative interpretation.
   3. If still impossible: write a clarification request. Mark the plan item `BLOCKED`. Stop.

   **Do not continue to subsequent items after blocking.**

   For each unblocked item:
   a. Before each item, mark it as in-progress in the plan.
   b. After each item, verify the change against the requirements.
   c. Mark the item as completed in the plan.
   d. At checkpoint items, pause and verify all preceding work against the implementation plan.
   e. At checkpoint items, update the maturity matrix (`docs/project/maturity.md`) if the completed work affects feature maturity.

   **For directory plans:** at the end of each phase, run the verify procedure inline (do not use the Skill tool — invoking it creates a new conversation turn and terminates implement). Read the phase file's Verification section, execute each listed check, and confirm all pass. If any check fails, stop and report the inconsistency. Do not proceed to the next phase until all checks pass.

5. **After all items are complete,** run acceptance tests.

6. **Set the plan status to "completed."**

7. **Report completion.**

## Blocked Plan Items

Blocked items are marked:
```
- [x] BLOCKED 5. <task> — see docs/project/clarifications/<date>-<topic>.md
```

## Resuming

If the plan status is "in-progress", find the first uncompleted TODO item (`- [ ]`) and resume from there. Do not re-execute completed items.

## Output Status

Each invocation appends a status line to its output:
```
Status: completed | blocked (see clarifications/<date>-<topic>.md)
```
