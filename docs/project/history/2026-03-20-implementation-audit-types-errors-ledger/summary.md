---
title: "History: Implementation Audit — Types, Errors, and Assurance Ledger"
category: history
audience: [all]
status: draft
last-verified: 2026-03-20
depends-on: [docs/project/proposals/implementation-audit-types-errors-ledger.md]
---

# Implementation Audit — Types, Errors, and Assurance Ledger

## Origin

This proposal originated from a post-implementation review of the types, errors, and assurance ledger consistency plan. After an AI agent implemented the full plan (types-errors-assurance-consistency), the human reviewer ran the acceptance tests and examined the code, discovering that several bugs had been introduced. The implementing agent had systematically misunderstood when the assurance ledger should perform its operations, gating all type-related ledger operations on the presence of a `types` feature in the active standard. The reviewer filed a detailed bug report describing seven issues ranging from incorrect feature gates in the interpreter to spec regressions and useless acceptance tests.

## v1 — Initial Audit Proposal (2026-03-20)

The first version performed a thorough audit of the implementation bugs and organized them into nine issues with detailed findings, critical analysis, and alternatives for each. The research phase examined the interpreter's if-statement handler, throw audit, type annotation checker, entry type registry, acceptance tests, and both the assurance-ledger and errors specs.

The central finding was that the interpreter drew a hard line between "types feature active" and "no types feature" — when the types feature was absent (as in the streamlined standard), the entire narrowing mechanism was skipped, the throw audit was skipped, and type annotations were ignored. This violated the spec's language that narrowing is "the natural consequence of the ledger maintaining entry sets through control flow," which implies unconditional operation. But the spec never explicitly stated this as a principle, making the implementing agent's interpretation an understandable — if incorrect — reading.

Beyond the feature-gating bugs, the audit found that the error type hierarchy had been implemented using inheritance in Rust (with domain types like TypeError and FileError extending SystemError), that the errors.md spec had regressed the defer scoping decision from function-scoped back to block-scoped, and that all eight type narrowing acceptance tests were vacuous — they asserted printed constants without ever checking whether the ledger actually performed narrowing.

Nine decisions were left open for the reviewer.

## v2 — Decisions and Structural Error Model (2026-03-20)

The reviewer's inline decisions resolved all nine open questions and surfaced a fundamental insight about how assurance levels work in ish.

**Assurance levels reframed.** The most consequential decision was the reviewer's correction of how low-assurance ish behaves. The proposal had suggested that streamlined mode "expects zero type checking overhead," which the reviewer identified as a misunderstanding. Low assurance does not disable checking — it defers it to runtime. The assurance continuum goes from "all checking at runtime" (low assurance) to "all checking at build time with runtime checks pruned where proven unnecessary" (high assurance). The developer in streamlined mode doesn't need to think about types, but ish still performs all compatibility checks at runtime. High assurance *reduces* runtime overhead by proving checks at build time and removing proven-unnecessary runtime checks during optimization. This insight required a new spec section (Decision 11) and reframed Issue 8's resolution from "alternative C (separate recording from checking)" to "alternative A (always check present annotations)" — because annotations are always meaningful and checking is never disabled.

**Structural error hierarchy adopted.** The reviewer's decision on the error hierarchy was more radical than any of the proposal's alternatives. Rather than fixing the inheritance tree (changing parents from SystemError to CodedError) or adopting the structural model as the proposal described it, the reviewer clarified that error hierarchy types are not special at all. They are ordinary ish types — not registered in Rust, not predefined by the system, not part of the ledger's built-in machinery. The *only* predefined entry type is `@Error`, which the throw audit adds to qualifying objects. Everything else — `CodedError`, `TypeError`, `FileError`, `SystemError` — is defined in ish as structural and union types: `CodedError = Error & { code: String }`, `FileNotFoundError = CodedError & { code: "E008" }`, `FileError = FileNotFoundError | PermissionError`, `SystemError = FileError | TypeError | ...`. The key insight is that SystemError is defined *in terms of* the domain types as a union, not as their parent. A FileNotFoundError is a SystemError because SystemError's union includes it — not because it inherits from SystemError. For now these types live in the spec and in acceptance tests; they'll move to the standard library when the module system is complete.

**Feature gates universally removed.** All three feature-gating issues (narrowing, throw audit, type annotation checking) were resolved identically: remove the gates. Entry maintenance always happens. The throw audit always runs. Present type annotations are always checked. Only the missing-annotation discrepancy (no annotation at all) is governed by the types feature at `required` level.

**Test observability established.** The reviewer chose to add both `ledger_state()` and `has_entry()` builtins, with a deferred TODO noting that `ledger_state` should eventually return a complex object rather than a string (pending a generic `to_string` mechanism). The acceptance tests will be rewritten to assert actual ledger state.

The revised proposal consolidated all decisions into an 11-item register, rewrote each issue's resolution to reflect the decisions, and removed all alternatives analysis — the v2 document is the authoritative statement of what will be implemented.

## Acceptance (2026-03-20)

A final cleanup removed the last vestiges of `CodedError` as an entry type from Issue 2's problem description. The pre-Decision-8 throw audit steps (which still listed auto-adding `@CodedError` entries) and the explanatory Note paragraph were collapsed into the final three-rule throw audit that only adds `@Error` entries. With all 11 decisions resolved, no `-->` markers remaining, and the body rewritten as settled fact, the proposal was accepted.

The accepted proposal covers: unconditional entry maintenance (Decisions 1, 2, 10), the structural error hierarchy with `@Error` as the sole predefined entry type (Decisions 7, 8), spec clarifications for maintenance-vs-auditing and assurance-level semantics (Decisions 4, 11), ledger observability builtins and test rewrites (Decisions 5, 6), defer scoping fix (Decision 9), and the complete throw audit (Decision 3).

## Implementation (2026-03-20)

The implementation plan organized twenty-seven TODOs across eight checkpoints, following authority order from spec docs through architecture, tests, code, and documentation cleanup.

**Spec changes landed cleanly.** The assurance-ledger spec gained two new subsections: "Entry Maintenance vs. Auditing" (establishing that maintenance is unconditional while auditing is governed by the active standard's feature states) and "Assurance Level Semantics" (explaining the low-to-high continuum where low assurance defers checking to runtime rather than disabling it). The built-in entry types table was trimmed from twelve rows to five — Error, Mutable, Type, Open, Closed — with a note that all other error classifications are structural ish types. The errors spec's hierarchy section was completely rewritten: a single-row `@Error` entry type table replaced the old inheritance tree, followed by structural type definitions in ish syntax (`CodedError = Error & { code: String }`, leaf types with specific code values, union types for domain classifications). The defer scoping regression was fixed in two places in errors.md — both the brief description and the detailed explanation now correctly say "function-scoped" with a cross-reference to the defer-scoping proposal.

**Architecture and error catalog aligned.** The vm.md architecture doc was updated across four sections: throw/try-catch now mentions only `@Error` as an entry type with a note about structural types, defer says "function" instead of "block," the architecture section explains that the VM notifies the ledger which performs maintenance unconditionally, and the error handling section notes the structural error model. The error catalog (errors/INDEX.md) replaced its "Hierarchy" column with "Structural Type," removing all `→ CodedError → SystemError → ...` chains.

**Acceptance tests rewritten for real observability.** The type narrowing tests are the most visible change. All eight tests had the `typed_std` standard definition removed — they now work without any standard active. Four tests gained ledger assertions using the new `ledger_state()` and `has_entry()` builtins: the null-exclusion test checks that `ledger_state("x")` contains "ExcludeNull," and the `has_entry` tests verify boolean returns. The remaining four tests retain behavioral assertions (checking else-branch execution and entry restoration). The entry type tests were updated to assert that CodedError and SystemError are *not* registered as entry types. The throw/catch tests were updated to expect wrapping behavior (accessing `e.original` instead of `e` directly). The type checking test that previously asserted "no checking without standard" now asserts that present annotations are always checked.

**The entry type registry shrink was surgical.** Seven entry type registrations were removed from `register_builtins` in entry_type.rs (CodedError, SystemError, TypeError, ArgumentError, FileError, FileNotFoundError, PermissionError), leaving only Error with its `message: String` requirement alongside the four non-error entry types. The six unit tests that exercised the removed hierarchy were rewritten: inheritance tests now use custom types registered in-test, subtype tests use custom hierarchies, and the builtins-registered test explicitly asserts that the removed types are absent.

**Interpreter changes removed all three feature gates.** The if-handler's `types_active` conditional was collapsed — the narrowing use-import, condition analysis, snapshot save/restore, and branch merging now run unconditionally. The throw handler lost its `active_features().contains_key("types")` gate entirely; in its place, the full three-rule throw audit runs on every throw: objects with `message: String` get an `@Error` ledger entry and pass through unchanged, objects without `message: String` get wrapped in a system error object `{ message, code: "E001", original }` with the `@Error` entry, and non-objects get the same wrapping treatment. The audit_type_annotation method was refactored: the early return on missing types feature was removed, and the logic now always checks present annotations for compatibility while only reporting missing-annotation discrepancies when the types feature is `required`.

**Two new builtins complete the ledger observability story.** The `ledger_state(variable_name)` and `has_entry(variable_name, entry_type)` builtins follow the same stub-plus-interception pattern as the existing ledger query functions: they're registered as named builtins in builtins.rs (where their closure bodies are unreachable error stubs) and intercepted by name in the interpreter's function call handler. `ledger_state` formats all entries on a variable as a comma-separated string (e.g., "ExcludeNull" or "Type(type: i32), ExcludeNull"), and `has_entry` returns a boolean.

**Build and test results:** 317 cargo tests pass (0 failures), 255 acceptance tests pass (7 groups, all green), and the manual inline verification confirms narrowing works without a standard — `ledger_state("x")` prints "ExcludeNull" inside a null-checked branch.

---

## Referenced by

- [docs/project/history/INDEX.md](../INDEX.md)
