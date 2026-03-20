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

2. **Verify the design proposal is accepted** and the plan status is "ready" (or "in-progress" if resuming).

3. **Set the plan status to "in-progress."**

4. **Execute TODO items in order:**
   a. Before each item, mark it as in-progress in the plan.
   b. After each item, verify the change against the requirements.
   c. Mark the item as completed in the plan.
   d. At checkpoint items, pause and verify all preceding work against the implementation plan.
   e. At checkpoint items, update the maturity matrix (`docs/project/maturity.md`) if the completed work affects feature maturity.

5. **After all items are complete,** run acceptance tests.

6. **Set the plan status to "completed."**

7. **Report completion.**

## Resuming

If the plan status is "in-progress", find the first uncompleted TODO item (`- [ ]`) and resume from there. Do not re-execute completed items.
