---
title: "History: Analyzer Correctness Fixes"
category: history
audience: [all]
status: active
date: 2026-04-04
---

# History: Analyzer Correctness Fixes

## Accepted (v3)

All six decisions resolved. Proposal accepted and ready for implementation planning. Final state: nine features covering `is_yielding` builtin, spawn yielding reclassification, undefined-function error behavior, implied await scope fix, `await expr` grammar expansion (E014, `Await{expr}` AST), unyielding context test repair, shell integration test cleanup, missing analyzer acceptance tests, and cross-boundary yielding test rewrite.

---

## v1 → v2

**Punch list:** Five inline decisions made in the proposal via `-->` notation.

**Decisions resolved:**

- **Decision 1** (`is_yielding` implementation): Confirmed as a **builtin**. The option to extend `type_of` was rejected as conflating type information with execution semantics.

- **Decision 3** (`await` grammar scope): Expanded to **any expression**, not just identifiers/variables. The decision also directs that tests cover complex expressions that resolve to futures (e.g., list indexing, conditional results), not only simple variable references.

- **Decision 4** (error code for `await non-future`): Confirmed as **E014** (`AwaitNonFuture`).

- **Decision 5** (`Await` AST node restructure): Confirmed **yes** — `{callee, args}` → `{expr}`.

- **Decision 6** (unyielding path allows `Spawn`): Confirmed **yes** — spawn is valid in unyielding execution contexts.

**Decision 2** ("replace or add alongside" for the spawn classification test) remains pending. The proposal body already recommends replacement; the implementation should proceed on that basis unless overridden.

**Changes to the proposal body:**

- Decision register updated to reflect resolved outcomes.
- `-->` markers removed; decisions are now recorded only in the register.
- Critical Analysis sections for Features 1 and 5 collapsed: since Option A was chosen in both cases, the rejected options were removed from the implementation sections.
- Feature 5 test cases expanded: added a complex-expression test (`await fs[0]` where `fs` is a list of futures) to satisfy Decision 3's directive to cover expressions beyond simple identifiers.
- Feature 6 implementation section updated to include the new spawn-is-valid positive test (consistent with Decision 6).
- Minor prose cleanup throughout; no content changes beyond the above.
