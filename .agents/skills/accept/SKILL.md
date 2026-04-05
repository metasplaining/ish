---
name: accept
description: 'Finalize a design proposal by recording all decisions and ensuring internal consistency. Use when: all decisions have been made and the proposal is ready for implementation planning. Trigger words: accept, finalize proposal, approve proposal.'
argument-hint: 'Path to the design proposal to accept'
---

# Accept

Finalize a design proposal so that all decisions are recorded, the content is consistent with all decisions, and no open decision points remain. This skill must not produce surprises — it records decisions that have already been made.

## When to Use

- All decision points in the proposal have been resolved
- The user wants to move from design phase to planning phase
- The user invokes `/accept <proposal-path>`

## Procedure

1. **Read the current design proposal.**

2. **Verify that all decision points have been resolved** (every `-->` has a recorded outcome).

3. **If any decision points remain unresolved,** present them to the user and ask for resolution before proceeding.

4. **Save the current proposal version to the design history directory** (as a new version file). The history directory is at `docs/project/history/<date>-<slug>/`. Name the file `v<N>.md`.

5. **Update the decision register** to reflect all decisions as final.

6. **Rewrite the body of the proposal:**
   a. Remove alternatives that were not chosen.
   b. Remove decision prompts (the `-->` lines).
   c. Present the chosen design as settled fact.
   d. Ensure internal consistency throughout.

7. **Scan for contradictions** and resolve them.

8. **Set the proposal status to "accepted"** in the YAML frontmatter.

9. **Save the accepted proposal** at the same path.

10. **Append a narrative** to the `summary.md` file in the design history directory marking acceptance and summarizing the final state.
