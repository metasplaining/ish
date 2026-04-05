*Extracted verbatim from [docs/project/proposals/agent-pipeline-skills.md](../../../proposals/agent-pipeline-skills.md) §Feature: Verify Skill (New).*

---

## Feature: Verify Skill (New)

### Purpose

After a phase of implementation or a bug fix, check that all changed artifacts are internally consistent. Verify detects inconsistencies but does not fix them. If inconsistencies are found, hand off to Troubleshoot.

### Consistency Checks

| Check | What it means |
|-------|---------------|
| Code ↔ unit tests | The code implements what the unit tests assert |
| Code ↔ acceptance tests | The code produces the output the acceptance tests expect |
| Tests ↔ spec | The tests cover all behaviors described in the spec |
| Spec ↔ proposal | The spec reflects the accepted proposal decisions |
| Architecture ↔ code | The architecture docs accurately describe the implementation |

Scope is limited to the artifacts changed in the phase or bug fix being verified.

### Skill Procedure

New skill: `.claude/skills/verify/SKILL.md`

1. Read the phase file or bug fix description being verified.
2. Identify all artifacts changed (from the plan's TODO items or bug fix report).
3. For each changed artifact, identify its authoritative source (spec, proposal, test).
4. Run consistency checks (see table above) for changed artifacts only.
5. Collect all inconsistencies found. For each, document:
   - What was found.
   - Which files are involved.
   - Which artifact is likely authoritative.
6. If inconsistencies found: write findings to `docs/project/clarifications/<date>-verify-<topic>.md`. Invoke `/troubleshoot` with that file as the problem report.
7. If no inconsistencies: output "Verified: no inconsistencies found in [phase/fix name]."

### Integration with Implement

The implement skill invokes `/verify` at each checkpoint. The phase file's Verification section specifies what to pass to verify:
```
Invoke: `/verify <plan-name>/phase-N.md`
```

The implement skill does not continue to the next phase until Verify reports clean.

### Issues to Watch Out For

- **Incomplete spec**: If the spec does not address a behavior, verification against it trivially succeeds. Flag incomplete specs as a finding, not a pass.
- **vs. Audit**: The Audit skill is broader (all artifacts, any time) and produces a report for human review. Verify is narrower (changed artifacts, post-implementation) and hands off to Troubleshoot. They are complementary, not redundant.
