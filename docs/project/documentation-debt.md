---
title: Documentation Debt
category: project
audience: [contributor]
status: draft
last-verified: 2026-03-10
depends-on: []
---

# Documentation Debt

Track documentation updates that are owed after code changes. Each entry records when the code changed and which documents need updating.

---

## Outstanding Debt

- [ ] String syntax (interpolation, raw strings, multiline, char literals) deferred to follow-on proposal — [docs/spec/syntax.md § Strings](../spec/syntax.md#strings) (2026-03-14)
- [ ] Feature state table in [docs/spec/assurance-ledger.md](../spec/assurance-ledger.md) is a placeholder — needs human review and completion (2026-03-14)
- [ ] Custom entry trackability mechanism TBD — [docs/spec/assurance-ledger.md § Open Questions](../spec/assurance-ledger.md#open-questions) (2026-03-14)
- [ ] Custom entry discrepancy messages TBD — [docs/spec/assurance-ledger.md § Open Questions](../spec/assurance-ledger.md#open-questions) (2026-03-14)

---

## Template

When a code change is made without a corresponding documentation update, add an entry:

```markdown
- [ ] <brief description of code change> (<date>) — <list of affected documents>
```

---

## Referenced by

- [docs/INDEX.md](../INDEX.md)
- [CONTRIBUTING.md](../../CONTRIBUTING.md)
