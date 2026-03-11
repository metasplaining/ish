---
title: Architecture Decision Records
category: project
audience: [all]
status: draft
last-verified: 2026-03-10
depends-on: [docs/INDEX.md]
---

# Architecture Decision Records

Each ADR records a design decision, its rationale, and what was considered and rejected.

---

| ADR | Title | Status | Date |
|-----|-------|--------|------|
| [001](001-documentation-structure.md) | Documentation Structure | accepted | 2026-03-10 |

---

## ADR Template

New ADRs should follow this template:

```markdown
---
title: ADR-NNN: <Decision Title>
status: proposed | accepted | superseded
date: YYYY-MM-DD
superseded-by: [ADR-NNN if applicable]
---

# ADR-NNN: <Decision Title>

## Context
What is the issue motivating this decision?

## Options Considered
1. **Option A** — description, pros, cons
2. **Option B** — description, pros, cons

## Decision
What change are we making?

## Consequences
What becomes easier or harder?
```

---

## Referenced by

- [docs/INDEX.md](../../INDEX.md)
- [docs/project/open-questions.md](../open-questions.md)
