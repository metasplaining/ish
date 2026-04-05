*Extracted verbatim from [docs/project/proposals/agent-pipeline-skills.md](../../../proposals/agent-pipeline-skills.md) §Feature: Backchannel Communication.*

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
