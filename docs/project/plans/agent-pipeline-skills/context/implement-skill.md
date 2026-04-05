*Extracted verbatim from [docs/project/proposals/agent-pipeline-skills.md](../../../proposals/agent-pipeline-skills.md) §Feature: Implement Skill — Guard Rails.*

---

## Feature: Implement Skill — Guard Rails

### What Changes

Add explicit rules for handling unclear or impossible tasks: stop at the first unclear item and write a clarification request.

### Updated Implement Skill Step 4

> **Before executing each TODO item:**
>
> Verify that the item is unambiguous and achievable.
>
> **If a task is unclear:**
> 1. Re-read the relevant specification section and proposal section.
> 2. If still unclear: do not attempt the task. Write a clarification request at `docs/project/clarifications/<date>-<topic>.md`. Mark the plan item `BLOCKED: see clarifications/<date>-<topic>.md`. Stop.
>
> **If a task appears impossible:**
> 1. Verify the task requirements against the accepted proposal decisions.
> 2. Try one alternative interpretation.
> 3. If still impossible: write a clarification request. Mark the plan item `BLOCKED`. Stop.
>
> **Do not continue to subsequent items after blocking.** Plans are too interconnected to skip steps.

---

*Additional context: the implement skill also gains phase-directory handling and Verify invocation. See [context/plan-skill.md](plan-skill.md) §Implement Skill Adaptation and [context/verify-skill.md](verify-skill.md) §Integration with Implement.*
