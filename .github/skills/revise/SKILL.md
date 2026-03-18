---
name: revise
description: 'Revise a design proposal based on a punch list or inline decisions. Use when: the user has reviewed a design proposal and wants changes incorporated. Handles both separate punch list documents and inline decisions (-->  notation). Trigger words: revise, punch list, update proposal, incorporate decisions.'
argument-hint: 'Path to the design proposal (optionally followed by path to punch list)'
---

# Revise

Update a design proposal based on a punch list or inline decisions. Produces a complete replacement that is internally consistent with all decisions.

## When to Use

- The user has reviewed a design proposal and wants changes made
- The user has added inline decisions (`-->` notation) to the proposal
- The user provides a separate punch list of corrections or additions
- The user invokes `/revise <proposal-path>`

## Procedure

1. **Read the current design proposal.**

2. **Identify the punch list:**
   a. If the user provides a separate punch list document, read it.
   b. If the user indicates that decisions have been made inline in the proposal (via `-->` notation or annotations), treat the inline edits as the punch list.

3. **Save the current proposal version to the design history directory** (as a new version file). The history directory is at `docs/project/history/<date>-<slug>/`. Name the file `v<N>.md` where N is the next version number.

4. **Update the decision register first** — add or update all decisions from the punch list.

5. **Rewrite the body of the proposal** to be consistent with the updated decision register. Produce a complete replacement; do not reference previous versions.

6. **Scan the entire document for internal contradictions.** Resolve any contradictions before saving.

7. **Save the replacement** at the same path, overwriting the previous version.

8. **Append a narrative** to the `summary.md` file in the design history directory explaining what the punch list requested and how the proposal changed.

9. **If an implementation plan exists** from a prior iteration (in `docs/project/plans/`), delete it — the design has changed and the plan is stale.
