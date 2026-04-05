---
title: Agent Pipeline Skills
category: proposal
audience: [ai-dev]
status: accepted
last-verified: 2026-04-04
depends-on:
  - docs/project/rfp/agent-instructions-improvements.md
  - .agents/skills/revise/SKILL.md
  - .agents/skills/plan-implementation/SKILL.md
  - .agents/skills/implement/SKILL.md
  - GLOSSARY.md
---

# Proposal: Agent Pipeline Skills

*Generated from [agent-instructions-improvements.md](../rfp/agent-instructions-improvements.md) on 2026-04-04.*
*Split from the combined agent-instructions-improvements proposal. Implement this proposal before [agent-infrastructure.md](agent-infrastructure.md) so that AGENTS.md can reference all final skills.*

---

## Decision Register

| # | Decision | Outcome |
|---|----------|---------|
| 1 | Revise split threshold | Split when ≥ 10 implementation steps can be extracted independently |
| 2 | Backward-compatibility check in Revise | Removed; project is in prototype stage, backward compatibility is not a concern |
| 3 | No-backward-compatibility rule | Add to AGENTS.md (handled by agent-infrastructure.md) |
| 4 | Plan directory threshold | > 5 steps: use directory structure; ≤ 5 steps: single file |
| 5 | Context file content | Verbatim extraction from authoritative sources (no summarizing) |
| 6 | Implement on unclear item | Stop at the first unclear item; write a clarification request |
| 7 | Bug Fix / Troubleshoot relationship | Bug Fix proceeds only after root cause is identified; if root cause is unknown, run Troubleshoot first |
| 8 | Bug Fix documentation scope | Fix all documented instances of the behavior, across all artifact types |
| 9 | Debug instrumentation | Scratch file (deleted after troubleshooting) |
| 10 | Verify integration | Standalone skill; also automatically invoked at each implement checkpoint |
| 11 | Clarification request location | `docs/project/clarifications/` |
| 12 | Terminology | skill = SKILL.md artifact; role = pipeline stage; agent = AI process executing a skill; backchannel, clarification request, phase (see GLOSSARY additions) |

---

## Background: The Agent Pipeline

The complete agent pipeline has eight roles:

| Role | Skill | Purpose |
|------|-------|---------|
| Propose | `/propose` | Create a design proposal from an RFP |
| Revise | `/revise` | Refine a proposal with decisions and gap detection |
| Accept | `/accept` | Finalize all decisions; prepare for planning |
| Plan | `/plan-implementation` | Generate an ordered implementation plan |
| Implement | `/implement` | Execute the plan following authority order |
| Bug Fix | `/bug-fix` | Fix a confirmed bug across all artifact types |
| Troubleshoot | `/troubleshoot` | Identify root cause without making changes |
| Verify | `/verify` | Check post-implementation consistency |

**Project-wide rule**: Creativity is allowed only in the Propose and Revise roles, in cooperation with a human. All other roles are fully predictable. When a non-creative role encounters a situation requiring creativity, it writes a clarification request and stops.

---

## Feature: Revise Skill — Gap Detection and Proposal Splitting

### What Changes

The Revise skill gains two new behaviors after incorporating the punch list:

1. **Gap detection**: After rewriting, scan each feature for specification completeness. Add open questions where gaps are found.
2. **Proposal splitting**: If the revised proposal contains ≥ 10 implementation steps that can be extracted and implemented independently, propose a split.

### Gap Detection Checklist

For each feature section, check:
- [ ] Are the error cases specified?
- [ ] Is concurrency behavior addressed (if the feature touches async code)?
- [ ] Is there a testability statement (what acceptance test would verify this)?

Note: backward-compatibility analysis is explicitly excluded. This project is in the prototype stage; backward compatibility is not a concern.

For each unchecked item, inject:
```
**Open Question:** [specific description of the gap]
-->
```

### Proposal Splitting

After gap detection, count the features in the proposal. Apply the split heuristic:

- **Trigger**: ≥ 10 implementation steps that can be implemented before and independently of the remaining features.
- **Do not split** if the features are tightly coupled (shared data structures, shared configuration, interleaved implementation steps).

Split procedure:
1. Identify the independent group and the dependent group.
2. Order so the independent group is implemented first.
3. Present the split as a decision:
   ```
   **Decision:** Split into Proposal A ([list features]) and Proposal B ([list features])?
   -->
   ```
4. If the human accepts the split: create new proposal files, update the history directory, and mark the parent proposal as split in its frontmatter.

### Issues to Watch Out For

- **History linking**: When splitting, both new proposals must reference the parent history directory. Use a `split-from` field in their frontmatter.
- **Gap detection false positives**: Not every feature needs explicit error case documentation. Features that are pure refactors or documentation-only changes can skip the error case check.
- **Circular dependencies**: When evaluating independence for splitting, check for shared types, shared constants, or shared test fixtures that would force a specific implementation order.

### Updated Revise Skill Procedure

Add the following steps to the Revise skill, between step 6 (scan for contradictions) and step 7 (save):

**Step 6a — Gap Detection:**
1. For each feature section, apply the gap detection checklist above.
2. For each uncovered item, add an Open Question prompt in the relevant section.
3. If any open questions were added, note this in the summary.

**Step 6b — Split Evaluation:**
1. Count extractable, independent implementation steps.
2. If ≥ 10: draft a split proposal and present it as a decision point.
3. If < 10: continue with the single proposal.

---

## Feature: Plan Skill — Phase Directories and Context Files

### What Changes

For proposals with more than 5 implementation steps, the plan skill produces a directory instead of a single file. Each directory contains an overview, verbatim context files extracted from the authoritative docs, and one file per phase.

### Directory Structure

```
docs/project/plans/<name>/
  overview.md          — summary, requirements, phase dependency graph
  context/
    <topic>.md         — verbatim extracts from spec/architecture docs
  phase-1.md
  phase-2.md
  ...
```

For plans with ≤ 5 steps: single file `docs/project/plans/<name>.md` (current behavior unchanged).

### Phase File Format

```markdown
# Phase N: <name>

*Part of: [<plan-name>/overview.md](overview.md)*

## Context Files
- [context/<topic>.md](context/<topic>.md) — <what it contains>

## Requirements
- <testable statement of expected behavior>

## Tasks
- [ ] 1. <task> — `<file>`
- [ ] 2. <task> — `<file>`
...

## Verification
Run: `<command>`
Check: <what to look for in the output>
Invoke: `/verify <plan-name>/phase-N.md`
```

### Context File Rules

- Content is verbatim from the authoritative source (spec, architecture doc, or proposal).
- Each context file identifies its source at the top: `*Extracted verbatim from [source](../../path/to/source.md) §Section Name.*`
- Do not paraphrase or summarize. If summarizing seems necessary because the source is too large, extract the relevant section verbatim and note what was omitted.
- If a context file would require information from a source that does not exist or is incomplete, this is a gap. Add the gap to a gap list and return the proposal for revision rather than continuing.

### Gap Detection Before Saving

Before saving, the plan skill scrutinizes each phase:
- Does each task have enough detail for an agent to execute without reading the original spec?
- Does each verification step have a concrete command to run?
- Is every context file available and non-empty?

If any phase fails scrutiny: write a gap list and write a clarification request (see Backchannels). Do not save the incomplete plan.

### Implement Skill Adaptation

The implement skill must detect whether a plan is a single file or a directory:
- Single file: current behavior.
- Directory: read `overview.md` first, then execute phases in order. After each phase, invoke `/verify <plan>/<phase>.md`.

### Issues to Watch Out For

- **Stale context files**: If the source doc changes after the plan is created, context files may be out of date. The implement skill should note the source location so the agent can verify currency.
- **Phase ordering**: The overview must explicitly state the implementation order and why (dependency graph). Agents must not re-derive ordering from the phase files alone.
- **Transition period**: Existing plans are single files. The implement skill must handle both formats.

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

## Feature: Bug Fix Skill (New)

### Purpose

Fix a confirmed bug — a behavior that is specified one way and implemented another way. Bug Fix proceeds only after root cause is identified (either provided by the human or by Troubleshoot). The skill fixes the behavior everywhere it is documented.

### Documentation Coverage Rule

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

### Skill Procedure

New skill: `.agents/skills/bug-fix/SKILL.md`

1. Read the bug report. Confirm root cause is identified. If not, stop and run `/troubleshoot` first.
2. Read the implementation plan for the affected feature.
3. Read the clarification document from Troubleshoot (if available) at `docs/project/clarifications/<date>-<topic>.md`.
4. Identify the behavior being fixed (one specific, named behavior).
5. Search all artifact types for every mention of that behavior. Build a fix checklist.
6. Fix the code.
7. Fix unit tests. Rules:
   - May fix test wording, format, or file references.
   - Must not change assertion logic.
   - If an assertion expects wrong behavior: stop; report to human. Do not fix.
8. Fix acceptance tests. Same rules.
9. Fix architecture docs, spec docs, user guide, AI guide (only sections referencing the broken behavior).
10. Add a history entry: what was wrong, what was fixed, which artifacts were updated.
11. Mark the fix in the implementation plan (add a note to the relevant TODO item).
12. Run: `cd proto && cargo test --workspace`
13. Run: `cd proto && bash ish-tests/run_all.sh`
14. For each new failure: if unrelated to the fix, document it; do not fix. If related to the fix and the test was wrong, it should have been caught in step 7–8 — re-examine.
15. Report completion with list of artifacts updated.

### Issues to Watch Out For

- **Root cause confidence**: If the human provides a root cause but it turns out to be wrong (the fix doesn't work), stop and return to Troubleshoot. Do not improvise an alternative fix.
- **Cascading fixes**: Fixing one behavior may reveal that a related behavior is also wrong. Do not fix the related behavior — document it as a new bug report and stop.
- **Test meaning vs. test form**: The prohibition on changing assertion logic is strict. "This test checks the wrong thing" → human decision. "This test references the wrong file path" → fix it.

---

## Feature: Troubleshoot Skill (New)

### Purpose

Determine the root cause of a problem without fixing it. The troubleshoot agent diagnoses; the bug-fix agent repairs. Troubleshoot must not change the specification, even if the specification appears incorrect.

### Specification Evaluation

Before concluding that the bug is an implementation error, evaluate the specification:
- Is the specification complete? (Does it address the failing case?)
- Is the specification unambiguous? (Could a careful reader reach a different interpretation?)
- Is the specification achievable? (Given the architecture, can it be implemented as written?)

If the answer to any question is "no": the problem requires a specification fix, not a code fix. Write a clarification request and stop. Do not attempt to fix the specification.

### Skill Procedure

New skill: `.agents/skills/troubleshoot/SKILL.md`

1. Read the problem report.
2. Read the implementation plan and accepted proposal for the affected feature.
3. Read the relevant spec and architecture docs.
4. Create a scratch file at `docs/project/clarifications/debug-scratch.md` (deleted on completion).
5. Form a hypothesis. Record it in the scratch file.
6. Gather evidence:
   - Add debug logging if needed. Document each change in the scratch file.
   - Run targeted tests: `cd proto && cargo test <test_name>` or equivalent.
   - Do not change behavior beyond adding debug instrumentation.
7. Confirm or refute the hypothesis. Update the scratch file.
8. Repeat steps 5–7 until root cause is identified.
9. Remove all debug instrumentation from code.
10. Delete the scratch file.
11. Evaluate the specification (see above).
12. If specification problem: write a clarification request at `docs/project/clarifications/<date>-<topic>.md`. Include: what was found, why it requires specification change, suggested resolution. Stop.
13. If implementation error: write a root-cause document at `docs/project/clarifications/<date>-<topic>.md`. Include: what is wrong, where in the code, what the correct behavior should be (per spec). Hand off to Bug Fix.
14. Report findings, with path to the clarification document.

### Issues to Watch Out For

- **Scope creep**: If additional bugs are found during investigation, document them separately. Do not expand scope.
- **"Achievable" judgment**: The troubleshoot agent must not conclude that a spec is "unachievable" simply because the current architecture makes it difficult. Consult the architecture docs before concluding unachievability.
- **Premature handoff**: Do not hand off to Bug Fix until root cause is confirmed, not just hypothesized.

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

New skill: `.agents/skills/verify/SKILL.md`

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

---

## Feature: Backchannel Communication

### What Changes

Agents that cannot proceed write a structured clarification request file and stop. This replaces (or supplements) in-conversation stopping, enabling async and sub-agent scenarios.

### Clarification Request File Format

```markdown
# Clarification Request: <topic>

*Created by: <skill name> on <date>*

## Context
<What was being done when the agent stopped>

## Blocked On
<The specific unclear, impossible, or under-specified item>

## Questions
1. <Question needing human decision>

## Recommended Resolution
<send back to Revise / new RFP / direct human answer / other>
```

Path: `docs/project/clarifications/<date>-<slug>.md`
Index: `docs/project/clarifications/INDEX.md`

Cleanup: After resolution, delete the clarification file and add a one-line note to the relevant history file.

### Plan Item Blocked Status

When an implement agent writes a clarification request, it also marks the plan item:
```
- [x] BLOCKED 5. <task> — see docs/project/clarifications/<date>-<topic>.md
```

### Skill Output Status

Each skill appends a status line to its output:
```
Status: completed | blocked (see clarifications/<date>-<topic>.md)
```

### Implementation

1. Create `docs/project/clarifications/INDEX.md`.
2. Update all skill SKILL.md files to include the clarification request format and output status convention.
3. Update plan file format to include the `BLOCKED` convention.

---

## Feature: Terminology Canonicalization

### GLOSSARY.md Additions

Add to GLOSSARY.md:

| Term | Definition |
|------|-----------|
| **Role** | A conceptual stage in the agent pipeline: Propose, Revise, Accept, Plan, Implement, Bug Fix, Troubleshoot, or Verify. A skill implements a role. |
| **Skill** | A SKILL.md file defining the procedure for an agent fulfilling a specific role. Loaded at prompt time by the agent's execution environment. |
| **Agent** | The AI process executing a skill within a session. |
| **Backchannel** | A mechanism for an agent to send structured feedback upstream when it cannot proceed. Currently implemented as clarification request files. |
| **Clarification request** | A structured file written by a blocked agent to request human input. Stored in `docs/project/clarifications/`. |
| **Phase** | A subdivision of an implementation plan that can be assigned to a single sub-agent. Each phase has its own file in the plan directory, with context files, tasks, and verification instructions. |

---

## Documentation Updates

- `.agents/skills/revise/SKILL.md` — add gap detection and split evaluation steps
- `.agents/skills/plan-implementation/SKILL.md` — add phase directory structure, context file rules, scrutiny step
- `.agents/skills/implement/SKILL.md` — add guard rail steps, phase-directory handling, Verify invocation
- `.agents/skills/bug-fix/SKILL.md` (new) — bug fix skill
- `.agents/skills/troubleshoot/SKILL.md` (new) — troubleshoot skill
- `.agents/skills/verify/SKILL.md` (new) — verify skill
- `GLOSSARY.md` — add six new terms
- `docs/project/clarifications/INDEX.md` (new) — clarification request index
- `docs/ai-guide/orientation.md` — update agent pipeline description
- `AGENTS.md` — add new skills to task playbooks table (handled by agent-infrastructure.md)

Update `## Referenced by` sections in: GLOSSARY.md.

---

## History Updates

- [ ] This proposal shares the history directory `docs/project/history/2026-04-04-agent-instructions-improvements/`
- [ ] Update `summary.md` when this proposal is accepted
- [ ] Add version files for each revision
