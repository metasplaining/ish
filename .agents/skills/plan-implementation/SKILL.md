---
name: plan-implementation
description: 'Generate an implementation plan from a design proposal. Use when: a design proposal has been accepted and is ready for implementation. Creates a consolidated TODO with authority-ordered file changes. Trigger words: plan-implementation, implementation plan, ready to implement.'
argument-hint: 'Path to the design proposal'
---

# Plan Implementation

Generate an implementation plan from an accepted design proposal. The plan is the single source of truth during implementation.

## When to Use

- A design proposal has been accepted and needs an implementation plan
- The user wants to move from design to implementation
- The user invokes `/plan-implementation <proposal-path>`

## Procedure

1. **Read the design proposal.**

2. **If the proposal status is not "accepted",** run the `/accept` skill first. (The human may send the proposal back after reviewing the accepted version and the plan.)

3. **Extract all accepted decisions** from the decision register.

4. **For each feature, compile the requirements** (testable statements of behavior).

5. **Determine the authority order** (use the 12-step default unless the proposal specifies otherwise).

6. **Generate the ordered TODO list:**
   a. For each artifact type in authority order, list the files that need to change and what changes are needed.
   b. Include checkpoint items at natural boundaries.
   c. Include feature coherence audit steps at appropriate points.

7. **Include a Reference section** with any information needed during implementation that would otherwise require reading historical documents.

7a. **Context File Rules** (for directory plans):
   - Content is verbatim from the authoritative source (spec, architecture doc, or proposal).
   - Each context file begins with: `*Extracted verbatim from [source](path/to/source.md) §Section Name.*`
   - Do not paraphrase or summarize.
   - If a context file requires information from a source that does not exist or is incomplete, treat this as a gap: add to a gap list and do not proceed to saving.

7b. **Scrutiny** — before saving, check each phase:
   - Does each task have enough detail for an agent to execute without reading the original spec?
   - Does each verification step have a concrete command to run?
   - Is every context file available and non-empty?

   If any phase fails: write a clarification request at `docs/project/clarifications/<date>-<topic>.md`. Do not save the incomplete plan. Output `Status: blocked (see clarifications/<date>-<topic>.md)` and stop.

8. **Save:**
   - If the plan has ≤ 5 implementation steps: save to `docs/project/plans/<name>.md` (current behavior).
   - If the plan has > 5 implementation steps: save to a directory:
     ```
     docs/project/plans/<name>/
       overview.md          — summary, requirements, phase dependency graph
       context/
         <topic>.md         — verbatim extracts from authoritative sources
       phase-1.md
       phase-2.md
       ...
     ```

## Output Format

**Single-file plan** (≤ 5 implementation steps):

```markdown
---
title: "Plan: <Topic>"
category: plan
audience: [ai-dev]
status: ready | in-progress | completed
last-verified: <date>
depends-on: [<design-proposal>, <spec-files>, <architecture-files>]
---

# Plan: <Topic>

*Derived from [<proposal>](../proposals/<proposal>) on <date>.*

## Overview
<2-3 sentence summary of what is being implemented>

## Requirements
<Extracted from the accepted design proposal. Each requirement is a testable statement.>

## Authority Order
<List of artifact types in order of authority for this feature.
Agent must update artifacts in this order.>

1. GLOSSARY.md (if new terms)
2. Roadmap (set to "in progress")
3. Specification docs
4. Architecture docs
5. User guide / AI guide
6. Agent documentation (AGENTS.md, skills)
7. Acceptance tests
8. Code (implementation)
9. Unit tests
10. Roadmap (set to "completed")
11. History
12. Index files

## TODO

- [ ] 1. <task> — <file(s) affected>
- [ ] 2. <task> — <file(s) affected>
...

## Reference

<Any information the implementing agent needs that would otherwise
require reading historical documents — e.g., original file locations
for files being moved, original terminology for terms being renamed.>
```

**Directory plan** (> 5 implementation steps):

```
docs/project/plans/<name>/
  overview.md          — summary, requirements, phase dependency graph
  context/
    <topic>.md         — verbatim extracts from authoritative sources
  phase-1.md
  phase-2.md
  ...
```

Each phase file follows this format:

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

## Output Status

Each invocation appends a status line to its output:
```
Status: completed | blocked (see clarifications/<date>-<topic>.md)
```
