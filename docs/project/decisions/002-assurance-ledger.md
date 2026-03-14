---
title: "ADR-002: Assurance Ledger"
status: accepted
date: 2026-03-14
---

# ADR-002: Assurance Ledger

## Context

ish needs a unified system for configuring and checking language features (types, mutability, null safety, overflow, memory model, polymorphism strategy, etc.) across its low-assurance ↔ high-assurance continuum. The original design used the "agreement" metaphor from linguistics, with "marked features" and "encumbrance levels." This terminology was unfamiliar to most developers and frequently confused with unrelated concepts (contractual agreements, legal encumbrances).

Three proposals explored alternatives:

1. [Agreement Metaphor and Syntax](../proposals/agreement-metaphor-and-syntax.md) — evaluated 8 metaphor candidates and 6 syntax approaches.
2. [Ledger System Syntax](../proposals/ledger-system-syntax.md) — refined naming to "Assurance Ledger" with Standard/Entry terminology.
3. [Assurance Ledger Syntax](../proposals/assurance-ledger-syntax.md) — consolidated all decisions into a complete syntax design.

## Options Considered

1. **Agreement** (original) — linguistic metaphor with "marked features" and "encumbrance." Unfamiliar, easily confused.
2. **Contract** — too closely associated with Eiffel's Design by Contract and legal/business contexts.
3. **Ledger** — accounting metaphor. Natural fit: entries record facts, standards govern what's required, audits detect discrepancies.
4. **Constraint** — too generic, overlaps with constraint programming.
5. **Proof** — implies formal verification, which ish does not provide.

## Decision

Adopt the **Assurance Ledger** metaphor with the following terminology:

| Old term | New term |
|----------|----------|
| Agreement | Assurance Ledger |
| Marked feature | Entry / Entry type |
| Agreement violation | Discrepancy |
| Agreement checking | Audit (pre-audit or live audit) |
| Encumbrance | Assurance level |
| Encumbered ish | High-assurance ish |
| Streamlined ish | Low-assurance ish |
| Profile | Standard |

Syntax constructs:

- **Standards** applied with `@standard[name]`, defined with `standard name extends base [feature(state), ...]`
- **Entries** applied with `@[entry(params)]`, defined with `entry type name { ... }`
- Native syntax (`: T`, `mut`, `async`, `throws E`) and entry annotations are fully interchangeable
- Feature states are parameterized (`optional`, `live`, `pre`, plus feature-specific states)
- Built-in standards (`streamlined`, `cautious`, `rigorous`) defined in the standard library

## Consequences

- All documentation referencing "agreement," "marked feature," "encumbrance," or "encumbered/streamlined" must be updated.
- The spec file `docs/spec/agreement.md` is renamed to `docs/spec/assurance-ledger.md` and rewritten.
- User guide `docs/user-guide/encumbrance.md` is renamed to `docs/user-guide/assurance-levels.md`.
- AI guide playbooks are renamed (`playbook-encumbered.md` → `playbook-high-assurance.md`, etc.).
- The prototype's `TypeAnnotation` AST nodes will eventually gain Standard, Entry, and EntryType counterparts.
- The feature state table is a placeholder pending further review.
- Custom entry trackability mechanism and custom entry discrepancy messages remain TBD.

---

## Referenced by

- [docs/project/decisions/INDEX.md](INDEX.md)
