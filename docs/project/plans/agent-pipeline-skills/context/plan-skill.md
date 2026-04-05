*Extracted verbatim from [docs/project/proposals/agent-pipeline-skills.md](../../../proposals/agent-pipeline-skills.md) §Feature: Plan Skill — Phase Directories and Context Files.*

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
