---
title: "RFP: Shell Build III"
category: rfp
audience: [all]
status: stable
last-verified: 2026-03-15
depends-on: [docs/project/proposals/shell-build-ii.md, docs/project/rfp/shell-build-ii.md]
---

# RFP: Shell Build III

*Decisions and corrections from [shell-build-ii.md](../proposals/shell-build-ii.md), captured for the follow-on proposal.*

---

## Decisions Made

### REPL completion reclassification

The following `IncompleteKind` variants were classified as **Error** but should be **Wait**. It may be unusual for them to span lines, but it is not an error:

- `IndexAccess`
- `CatchParam`
- `FunctionType`
- `GenericParams`
- `GenericType`

This means only single-line string types remain as **Error**: `StringLiteral`, `InterpString`, `CharLiteral`, `ExtendedDoubleString`, `ExtendedSingleString`, `ShellQuotedString`, `ShellSingleString`.

### Validator input size cap

No cap. Prioritize correctness over performance.

### `has_incomplete` API style

Inherent methods on `Program`/`Statement`/`Expression`, not free functions in a module.

### Non-delimiter error productions

Out of scope for this proposal chain.

---

## Errors Found

### Stale bracket-counting text

Feature 5 of shell-build-ii contains text describing a "two-layer approach" where the `IshValidator` bracket-counting state machine is preserved as a fast path. This contradicts the decision (made in shell-build) that bracket counting is fundamentally broken and eliminated. The text about "two-layer approach", "keystroke-level validator", and "fast path" must be removed.

---

## Scope of Follow-on Proposal

The follow-on proposal should make only these changes:

1. Record all decisions above.
2. Correct the `is_continuable()` method to reflect the reclassification.
3. Correct the REPL completion category lists.
4. Remove the stale bracket-counting text.
5. No new features, no new design changes.

---

## Referenced by

- [docs/project/rfp/INDEX.md](INDEX.md)
