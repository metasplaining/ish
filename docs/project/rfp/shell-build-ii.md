---
title: "RFP: Shell Build II"
category: rfp
audience: [all]
status: stable
last-verified: 2026-03-15
depends-on: [docs/project/proposals/shell-build.md, docs/project/rfp/shell-build.md]
---

# RFP: Shell Build II

*Decisions from [shell-build.md](../proposals/shell-build.md), captured for the follow-on proposal.*

---

## Decisions Made

### Object literal ambiguity

Object literals should not be a top-level production. `{ EOI` is always treated as an unterminated block, never an unterminated object literal.

### Parser-matches-everything philosophy

Almost nothing should be a parse error. The grammar should match on almost every input — it may match something that the system treats as an error (like an unterminated string), but the parser itself treats it as a successful match. This eliminates the Tier 1 / Tier 2 distinction. Every delimited construct gets an unterminated production, including single-line strings, extended strings, annotations, type annotations, and all other previously deferred constructs.

### Unterminated single-line strings

The parser matches unterminated single-line strings as `unterminated_string_literal` / `unterminated_interp_string`. The shell treats these as errors (not as "waiting for more input"), but the parser still matches them successfully.

### Unterminated block comments

Must be a grammar production, not just caught by a validator. All incomplete productions are needed for good error messages when reading whole files, not only for the REPL.

### Bracket-counting validator — eliminated

Counting brackets is fundamentally broken. `let x = '{'` would cause a false incomplete detection because the bracket counter doesn't understand string quoting. There should be NO bracket counting. The `IshValidator` state machine from shell-construction Feature 3 is eliminated entirely. The parser is the only authority on completeness.

### `has_incomplete` location

The `has_incomplete` / `stmt_is_incomplete` / `expr_is_incomplete` functions belong in `ish-ast`, not `ish-shell`, so they are available to all crates (VM error messages, future tooling, etc.).

---

## Scope of Follow-on Proposal

The follow-on proposal should:

1. Provide the complete inventory of ALL unterminated productions (no tiers — everything).
2. Explain how the REPL works with parser-only validation (no bracket-counting layer).
3. Categorize each `IncompleteKind` as "wait for more input" vs. "report as error" from the REPL's perspective.
4. Specify the `has_incomplete` API in `ish-ast`.
5. Address how the parser-matches-everything philosophy interacts with the existing `Result<Program, Vec<ParseError>>` API — does `parse()` still return `Err`, or does it always return `Ok`?

---

## Referenced by

- [docs/project/rfp/INDEX.md](INDEX.md)
