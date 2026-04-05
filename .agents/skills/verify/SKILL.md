---
name: verify
description: 'Check post-implementation consistency for changed artifacts. Use when: a phase or bug fix is complete and needs consistency verification. Invoked automatically by implement at each checkpoint. Trigger words: verify, check consistency, post-implementation check.'
argument-hint: 'Path to the phase file or bug fix description being verified'
---

# Verify

After a phase of implementation or a bug fix, check that all changed artifacts are internally consistent. Verify detects inconsistencies but does not fix them. If inconsistencies are found, hand off to Troubleshoot.

## When to Use

- A phase of implementation is complete
- A bug fix is complete
- The implement skill invokes `/verify` at a checkpoint
- The user invokes `/verify <plan-name>/phase-N.md`

## Consistency Checks

| Check | What it means |
|-------|---------------|
| Code ↔ unit tests | The code implements what the unit tests assert |
| Code ↔ acceptance tests | The code produces the output the acceptance tests expect |
| Tests ↔ spec | The tests cover all behaviors described in the spec |
| Spec ↔ proposal | The spec reflects the accepted proposal decisions |
| Architecture ↔ code | The architecture docs accurately describe the implementation |

Scope is limited to the artifacts changed in the phase or bug fix being verified.

## Skill Procedure

1. Read the phase file or bug fix description being verified.
2. Identify all artifacts changed (from the plan's TODO items or bug fix report).
3. For each changed artifact, identify its authoritative source (spec, proposal, test).
4. Run consistency checks (see table above) for changed artifacts only.
5. Collect all inconsistencies found. For each, document:
   - What was found.
   - Which files are involved.
   - Which artifact is likely authoritative.
6. If inconsistencies found: write a clarification request at `docs/project/clarifications/<date>-verify-<topic>.md` using the Clarification Request Format below. Use: Context = what phase or fix was being verified; Blocked On = the inconsistencies found (list each with files and authoritative source); Questions = which artifact should be treated as authoritative; Recommended Resolution = invoke `/troubleshoot` with that file as the problem report. Add a row to `docs/project/clarifications/INDEX.md`. Invoke `/troubleshoot` with that file.
7. If no inconsistencies: output "Verified: no inconsistencies found in [phase/fix name]."

## Integration with Implement

The implement skill runs the verify procedure **inline** at each phase checkpoint — it does not use the Skill tool to invoke verify (doing so would create a new conversation turn and terminate the implement execution). The phase file's Verification section specifies the checks to run. Implement reads those checks and executes them directly, then continues to the next phase only if all pass.

The `/verify` skill is invoked directly by the user when they want a standalone consistency check outside of an implement run.

## Issues to Watch Out For

- **Incomplete spec**: If the spec does not address a behavior, verification against it trivially succeeds. Flag incomplete specs as a finding, not a pass.
- **vs. Audit**: The Audit skill is broader (all artifacts, any time) and produces a report for human review. Verify is narrower (changed artifacts, post-implementation) and hands off to Troubleshoot. They are complementary, not redundant.

## Clarification Request Format

When writing a clarification file, use this structure:

```markdown
# Clarification Request: <topic>

*Created by: verify on <date>*

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
`| <date> | <slug> | verify | open | [<date>-<slug>.md](<date>-<slug>.md) |`

## Output Status

Each invocation appends a status line to its output:
```
Status: completed | blocked (see clarifications/<date>-<topic>.md)
```
