---
title: "RFP: Shell Build"
category: rfp
audience: [all]
status: stable
last-verified: 2026-03-15
depends-on: [docs/project/proposals/shell-construction.md, docs/project/rfp/shell-construction.md]
---

# RFP: Shell Build

*Decisions from [shell-construction.md](../proposals/shell-construction.md), captured for the follow-on proposal.*

---

## Decisions Made

### Feature 1: Parser Enhancement — Incomplete Input Detection

**Decision (strategy):** Do not add a `parse_chunk()` heuristic. Instead, add grammar productions for incomplete constructs — unterminated strings, lists, blocks, etc. The parser should match invalid input rather than returning errors, so the VM can generate good error messages. The REPL maintains a list of productions that indicate incomplete (rather than invalid) input.

**Key insight:** The incomplete productions should match on `EOI` (end of input), not end of line. When the user submits `{ let x = 5` and input ends, an `unterminated_block` rule matches because it accepts `EOI` in place of the closing `}`. When the input is complete (`{ let x = 5 }`), the normal `block` rule matches first (PEG ordered choice).

### Feature 1: Inline Execution Flag

**Decision:** Use `-c` (bash convention).

### Feature 6: Exit Code Variable

**Decision:** Implement `$?` as a synthetic environment variable set by the VM after each shell command.

---

## Scope of Follow-on Proposal

The follow-on proposal should provide:

1. A complete inventory of all grammar constructs that can be "unterminated" — every rule in the grammar that has a closing delimiter.
2. For each, the concrete `EOI`-based grammar production to add.
3. Corresponding AST node additions (new variants or a wrapper).
4. AST builder mappings for the new rules.
5. The REPL's incomplete-detection logic: which AST nodes trigger continuation vs. error display.
6. Interaction with the multiline validator (Feature 3 from shell-construction) — does the validator become unnecessary, or does it still serve as the keystroke-level fast path?

---

## Referenced by

- [docs/project/rfp/INDEX.md](INDEX.md)
