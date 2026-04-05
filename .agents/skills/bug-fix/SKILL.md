---
name: bug-fix
description: 'Fix a confirmed bug across all artifact types. Use when: root cause is identified and the behavior needs to be corrected in code, tests, and docs. Trigger words: bug fix, fix bug, fix confirmed bug.'
argument-hint: 'Path to the bug report or clarification document with root cause'
---

# Bug Fix

Fix a confirmed bug — a behavior that is specified one way and implemented another way. Bug Fix proceeds only after root cause is identified (either provided by the human or by Troubleshoot). The skill fixes the behavior everywhere it is documented.

## When to Use

- Root cause of a bug has been identified (by Troubleshoot or the human)
- The behavior needs to be corrected in code, tests, and docs
- The user invokes `/bug-fix <path-to-bug-report>`

## Documentation Coverage Rule

Each system behavior is documented exactly once per artifact type:

| Artifact | Example location |
|----------|-----------------|
| Code | Implementation in `proto/` |
| Unit tests | Test in `proto/` |
| Acceptance tests | Test in `proto/ish-tests/` |
| Architecture doc | File in `docs/architecture/` |
| Spec doc | File in `docs/spec/` |
| User guide | File in `docs/user-guide/` |
| AI guide | File in `docs/ai-guide/` |
| History | File in `docs/project/history/` |
| Errors catalog | File in `docs/errors/` (if applicable) |

Exception: language syntax features are documented twice — once for behavior, once for syntax.

When fixing a bug, identify the behavior being corrected, then locate and fix every instance across all artifact types. Do not perform a general re-audit; focus only on the specific behavior that was wrong.

## Skill Procedure

1. Read the bug report. Confirm root cause is identified. If not, stop and run `/troubleshoot` first.
2. Read the implementation plan for the affected feature.
3. Read the clarification document from Troubleshoot (if available) at `docs/project/clarifications/<date>-<topic>.md`.
4. Identify the behavior being fixed (one specific, named behavior).
5. Search all artifact types for every mention of that behavior. Build a fix checklist.
6. Fix the code.
7. Fix unit tests. Rules:
   - May fix test wording, format, or file references.
   - Must not change assertion logic.
   - If an assertion expects wrong behavior: write a clarification request at `docs/project/clarifications/<date>-<topic>.md` using the Clarification Request Format below. Use: Context = which test and fix are being processed; Blocked On = the assertion that expects wrong behavior (quote it); Questions = should this test be updated to reflect the correct behavior?; Recommended Resolution = human decision required. Add a row to `docs/project/clarifications/INDEX.md`. Stop.
8. Fix acceptance tests. Same rules.
9. Fix architecture docs, spec docs, user guide, AI guide (only sections referencing the broken behavior).
10. Add a history entry: what was wrong, what was fixed, which artifacts were updated.
11. Mark the fix in the implementation plan (add a note to the relevant TODO item).
12. Run: `cd proto && cargo test --workspace`
13. Run: `cd proto && bash ish-tests/run_all.sh`
14. For each new failure: if unrelated to the fix, document it; do not fix. If related to the fix and the test was wrong, it should have been caught in step 7–8 — re-examine.
15. Report completion with list of artifacts updated.

## Issues to Watch Out For

- **Root cause confidence**: If the human provides a root cause but it turns out to be wrong (the fix doesn't work), stop and return to Troubleshoot. Do not improvise an alternative fix.
- **Cascading fixes**: Fixing one behavior may reveal that a related behavior is also wrong. Do not fix the related behavior — document it as a new bug report and stop.
- **Test meaning vs. test form**: The prohibition on changing assertion logic is strict. "This test checks the wrong thing" → human decision. "This test references the wrong file path" → fix it.

## Clarification Request Format

When writing a clarification file, use this structure:

```markdown
# Clarification Request: <topic>

*Created by: bug-fix on <date>*

## Context
<What was being done when the agent stopped>

## Blocked On
<The specific unclear, impossible, or under-specified item>

## Questions
1. <Question needing human decision>

## Recommended Resolution
<send back to Revise / new RFP / direct human answer / other>
```

After writing, add a row to `docs/project/clarifications/INDEX.md`:
`| <date> | <slug> | bug-fix | open | [<date>-<slug>.md](<date>-<slug>.md) |`

## Output Status

Each invocation appends a status line to its output:
```
Status: completed | blocked (see clarifications/<date>-<topic>.md)
```
