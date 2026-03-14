---
title: Assurance Ledger Design
category: project
audience: [all]
status: draft
last-verified: 2026-03-14
depends-on: [docs/project/proposals/assurance-ledger-syntax.md, docs/project/decisions/002-assurance-ledger.md]
---

# Assurance Ledger Design

*March 14, 2026*

## The Starting Point

ish had always intended to support a continuum from lightweight, dynamically-checked code to heavily-annotated, statically-verified code. The original design called this continuum "encumbrance" and used a metaphor borrowed from linguistics: "agreement." In natural language, agreement means that related parts of a sentence must be consistent — a subject's number must agree with its verb's conjugation. Applied to programming, the idea was that when a language feature is "marked," related parts of the program must be consistent with each other. A variable's declared type must agree with its assigned value. A function's declared error types must agree with the errors it actually throws.

The metaphor was elegant in theory, but it had problems in practice. "Agreement" to most developers means a contract or a promise, not a grammatical constraint. "Encumbrance" sounds like a burden or a legal lien. "Marked feature" is opaque without explanation. The terminology created a barrier to understanding rather than illuminating the concept.

## Searching for a Better Metaphor

The search for alternatives began with a prompt file that asked three questions: what terminology do other languages use for similar concepts, what alternative metaphors could replace "agreement," and what syntax options exist for expressing these ideas in code?

The first proposal evaluated eight candidate metaphors — Agreement (unchanged), Contract, Constraint, Ledger, Proof, Accord, Assay, and Canon — against criteria including conceptual fit, developer familiarity, extensibility, and distinctiveness. Each had strengths: Contract was immediately familiar from Design by Contract; Constraint aligned with type theory; Proof carried mathematical weight. But most also carried baggage that would mislead. Contract implies runtime checking of pre/post-conditions. Constraint suggests a solver or logic programming. Proof implies formal verification guarantees that ish does not provide.

The Ledger metaphor stood out for its natural vocabulary. In accounting, a ledger records entries. Entries are facts — debits, credits, line items. An auditor reviews the ledger for discrepancies. Standards govern what must be recorded and how. The metaphor extended cleanly to programming: entries record facts about code (this variable has type i32, this function throws NetworkError), standards configure what facts are required, and audits detect discrepancies between entries. The vocabulary was distinctive enough to be searchable and memorable, yet grounded enough to be intuitive.

The human chose Ledger, with a request to distance it from blockchain associations using an adjective prefix.

## From Ledger to Assurance Ledger

The second proposal tackled naming. Six adjective candidates were evaluated: Proof Ledger, Tally Ledger, Binding Ledger, Assurance Ledger, Ruling Ledger, and Source Ledger. The proposal recommended Proof Ledger for its connection to mathematical proof and accounting proof-of-balance. But the human chose Assurance Ledger — it captured the idea that the system provides assurance about code quality, and "assurance" naturally scales: low assurance for quick scripts, high assurance for safety-critical code. This also provided the replacement for "encumbrance": assurance level.

The second proposal also established the two annotation constructs. Block-scoped configurations were initially called "rulings" (an accounting term for a regulatory interpretation), but the human preferred "standard" — it works both as an accounting term (accounting standards) and a coding term (coding standard). Item-level facts kept the term "entry" from the ledger metaphor. The syntax solidified around `@standard[name]` for standards and `@[entry(params)]` for entries.

A key decision was that standards replace the earlier concept of "profiles." Rather than having a separate profile construct, standards are the single mechanism for configuring what the ledger checks. Standards can extend other standards, override individual features, and be applied at any scope — module, function, or block level.

## Consolidating the Syntax

The third proposal brought everything together. It addressed twelve specific design questions:

**Standard definitions** use a dedicated `standard` keyword with bracket-list syntax: `standard name extends base [feature(state), ...]`. Standards can be defined inside functions and blocks, not just at module level. The built-in standards — `streamlined`, `cautious`, and `rigorous` — live in the standard library rather than being hardcoded into the language.

**Feature states** are parameterized, not boolean. Each feature has a set of valid states beyond simple on/off. The three base states are `optional` (not required), `live` (checked at execution time), and `pre` (checked at build time). Some features have additional states — `overflow` takes a behavior parameter (`wrapping`, `panicking`, `saturating`), `implicit_conversions` takes `allow` or `deny`, and so on. When a feature appears in a standard without a parenthetical, it defaults to `live`, except for features that only apply to function declarations, which default to `pre`.

**Entry annotations** use `@[entry(params)]` syntax. Native syntax and entry annotations are fully interchangeable: `let mut x: i32 = 7` and `@[mutable] @[type(i32)] let x = 7` produce identical ledger entries. Custom entry types are defined with `entry type name { ... }` blocks and support inheritance via `extends`.

**Error declarations** were simplified: `undeclared_errors` replaces the separate `checked_exceptions` feature. It takes a list of allowed entry types — for example, `@standard[@undeclared_errors(@Error)]` means functions may return any value annotated with `@Error` without declaring it.

**Cross-module boundaries** follow a natural rule: a module's standards govern entries entailed by statements within that module's lexical scope. When module A passes a variable to a function in module B, the variable carries A's entries, and the function's parameters require B's entries. The audit checks for discrepancies at the boundary. Modules of different assurance levels can interoperate as long as the entries are compatible.

**Discrepancy reporting** includes an audit trail that traces back through the chain of standards and statements that led to the conflict, making it clear why a discrepancy was raised and where each contributing entry originated.

Two items were explicitly left as TBD: the mechanism by which custom entry types become trackable by standards, and the format of discrepancy messages for custom entries. The feature state table was marked as a placeholder pending further review.

## The Decision

All decisions from the three proposals were accepted and recorded in [ADR-002](../decisions/002-assurance-ledger.md). The implementation plan covers updating approximately 40 documentation files, renaming `docs/spec/agreement.md` to `docs/spec/assurance-ledger.md`, and eventually adding Standard, Entry, and EntryType AST nodes to the prototype.

---

## Referenced by

- [docs/project/history/INDEX.md](INDEX.md)
