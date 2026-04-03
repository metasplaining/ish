---
title: "History: Concurrency Correctness Fixes"
category: history
audience: [all]
status: draft
last-verified: 2026-04-01
depends-on: [docs/project/proposals/concurrency-correctness.md]
---

# Concurrency Correctness Fixes — History

## Summary

Three correctness issues were identified in the concurrency prototype after the initial implementation (completed 2026-03-31 under the concurrency-code plan). The issues were surfaced during a post-implementation review.

### v1 — Initial Proposal (2026-04-01)

The proposal identified three issues:

1. **FutureRef equality** — The `PartialEq` implementation for futures returned `false` for all comparisons, including `f == f`. The proposal recommended `Rc::ptr_eq` for identity-based equality, with two alternatives (ID comparison and keeping never-equal).

2. **Await on non-future** — The interpreter silently returned non-future values from `await`, treating it as an identity operation. The proposal recommended making this always an error, with alternatives ranging from a plain E004 TypeError to a new E012 code.

3. **Blocking builtins** — `print`, `println`, `read_file`, and `write_file` perform synchronous I/O on the Tokio `LocalSet` thread. The proposal recommended a two-signature architecture with simple and parallel builtin kinds, where parallel builtins use `spawn_blocking`.

Six decisions were presented for review.

### v2 — Revised with Inline Decisions (2026-04-01)

The reviewer made decisions on all six items and added two significant corrections:

**Decision outcomes:** `Rc::ptr_eq` for future equality (Decision 1). New error code E012 for await-on-non-future (Decisions 2–3). Two-signature architecture for parallel builtins (Decision 4). Only `print`, `println`, `read_file`, `write_file` become parallel (Decision 5). Parallel builtins return `Future<T>` that callers must handle (Decision 6).

**Key correction on await semantics (Decision 2):** The reviewer noted that await-on-non-future interacts with function yielding categorization. Ideally, every function would be categorized as yielding or unyielding at declaration time, and `await` would check the callee's category, not just the result type. Since this categorization doesn't exist yet, the prototype uses value-type checking (is it a Future?) as a pragmatic stopgap. An open question was created to track eventual function categorization.

**Key correction on parallel builtins (Decision 6 + Issue annotation):** The original proposal stated parallel builtins would be "transparent to the user" — auto-awaited internally. The reviewer corrected this: parallel builtins should return `Future<T>` and require explicit `await` or `spawn`, just like any other yielding function. When `await_required` is `required`, no special treatment. The reviewer also noted that builtins are temporary scaffolding that should eventually be replaced by standard library functions, and requested an open question to track this.

A seventh decision (Decision 7) was added, explicitly codifying that parallel builtins respect the `await_required` feature.

The proposal was rewritten to incorporate all decisions, removing alternatives that were rejected. The implementation section was updated to account for the `Future`-returning design, including the `spawn_blocking` → `spawn_local` bridge pattern to handle `Rc`/`!Send` constraints, and noting the significant test migration required (all `println(...)` calls become `await println(...)`).

### v3 — Revised with Further Corrections (2026-04-01)

The reviewer made four further corrections that significantly changed Features 2 and 3.

**Reversal on await-on-non-future (Decision 2 revised):** The v2 proposal specified that `await` on a non-future always throws E012. The reviewer demonstrated why this is wrong with a concrete example: `fn my_fn() { println('I yield') }; await my_fn()`. Here, `my_fn` is a yielding function (it calls a parallel builtin internally), but the prototype has no Yielding entry for it. With implied await, `println(...)` inside `my_fn` resolves to a plain value, so `my_fn()` returns a plain value — and E012 would incorrectly reject the `await`. The reviewer concluded that since the prototype cannot distinguish yielding from unyielding functions, it must allow await-on-non-future as a pass-through to avoid breaking correct programs. E012 is reserved for future use when function categorization exists.

**New spawn validation (Decision 4 added):** The reviewer introduced E013 for invalid spawn targets: spawning anything other than a function or builtin is an error, and spawning a function with an explicit unyielding entry is also E013.

**Parallel builtins use implied await (Decision 7 revised):** Rather than requiring all existing `println(...)` calls to become `await println(...)`, the reviewer specified that at low assurance (no `await_required` active), calling a parallel builtin without `await` or `spawn` implies an `await`. This means `println("hello")` is equivalent to `await println("hello")` — the future is created, immediately awaited, and the resolved value returned. This preserves backward compatibility: existing tests do not need to change. Only under `@standard[cautious]` (where `await_required` is `required`) must the user write explicit `await` or `spawn`. The only way to obtain a `Future` value is via `spawn`.

**I/O completion guarantee (Decision 9 added):** The reviewer specified that parallel builtin futures must not resolve until the I/O operation is actually complete. For interactive shell mode (using the ExternalPrinter crossbeam channel), this means the shell must send an acknowledgment back after writing output. A `oneshot::channel` is included with each print message for this purpose.

The decision register grew from 7 to 9 decisions. The proposal was restructured: Feature 2 was renamed "Spawn Validation and Await Semantics" to reflect the new E013 spawn validation alongside the revised await behavior. Feature 3's test impact section was simplified — existing tests no longer need migration at low assurance.

### v4 — E012 Refinement and Builtin Parity (2026-04-01)

The reviewer corrected two misinterpretations in v3.

**E012 is not blanket-disabled (Decision 2 refined):** The v3 proposal treated the reviewer's earlier clarifications as a full reversal, disabling E012 entirely and reserving it for the future. The reviewer clarified that the earlier decisions were clarifications, not reversals. E012 should be thrown in all clear-cut cases:

- Awaiting anything other than a function or builtin call (e.g., `await 42`) — E012
- Awaiting a call to a function or builtin with an explicit unyielding entry — E012

The **only** case where E012 is not thrown is the ambiguous case: a function with neither `async` nor `@[unyielding]` annotation that returns a non-Future value. In that case the result passes through, because the prototype cannot determine whether the function yields internally. The reviewer's original example — `fn my_fn() { println('I yield') }` — falls into this ambiguous case and is handled correctly without throwing E012. The error was in generalizing this exception to all non-Future values.

A decision table was added to make the behavior exhaustive and unambiguous.

**Builtin–function parity (Decision 10 added):** The reviewer specified that builtins should appear to outside observers exactly like regular functions. Currently, `BuiltinFn` carries only a name and a closure — no parameter metadata, no return type, no yielding classification. The interpreter dispatches builtin calls with `(b.func)(args)` — no param count checking, no type auditing. Regular functions get all of this.

The proposal was expanded to give `BuiltinFn` the same metadata as `IshFunction`: parameter names and types, return type, and a yielding flag. Simple builtins get an unyielding entry. Parallel builtins get yielding and parallel entries. The interpreter applies the same parameter-count checking and type auditing to builtin calls as it does to regular functions. A full table of all 34+ simple builtins with their parameter signatures and return types was added.

This decision has two effects beyond parity: it makes E012 and E013 work correctly for builtins. `await len([1,2])` throws E012 because `len` has an unyielding entry. `spawn len([1,2])` throws E013 for the same reason.

The decision register grew from 9 to 10 decisions.

### v5 — Grammar Restriction and Compiled Functions (2026-04-01)

The reviewer identified two structural problems in v4's implementation approach and provided a clarification about the parser-matches-everything philosophy.

**Grammar-level await/spawn restriction (Decision 11):** The v4 proposal struggled with checking callee yielding classification because `Expression::Await` and `Expression::Spawn` wrap a generic `Box<Expression>`. The reviewer asked: "Why does Expression::Await have an expression as a parameter, instead of a FunctionCall?" This was identified as a grammar-level error. `await` and `spawn` should syntactically require a function call, not accept any expression.

The reviewer clarified that parser-matches-everything does not mean valid and invalid productions match the same tokens. The parser always succeeds on any input, but it encodes invalid forms (like `await 42`) as `Incomplete` AST nodes rather than valid `Await` nodes. The grammar rules change from `await_op ~ unary` to `await_op ~ call_expr`. The AST `Await` and `Spawn` nodes change from wrapping `Box<Expression>` to containing `callee: Box<Expression>` and `args: Vec<Expression>`, mirroring `FunctionCall`.

This resolves the v4 problem of needing to check callee classification after evaluation — with the grammar guaranteeing a call, the callee can be resolved and its yielding classification checked *before* calling. E012/E013 are thrown before the function executes.

**FunctionImplementation enum and builtin elimination (Decisions 12, 5 revised, 10 revised):** The reviewer rejected the v4 approach of expanding `BuiltinFn` with metadata. Instead, they specified a `FunctionImplementation` enum with three variants: `Interpreted(Statement)` for user-defined functions, `Concurrent(Shim)` for compiled synchronous functions (simple builtins, ledger builtins), and `Parallel(ParallelShim)` for compiled parallel functions (I/O builtins). `IshFunction.body: Statement` is replaced by `IshFunction.implementation: FunctionImplementation`. `BuiltinFn` and `Value::BuiltinFunction` are eliminated entirely — all builtins become regular `Value::Function` with compiled implementations.

A `ConcurrentShim` has the same capability as `exec_statement` — full access to `IshVm`, `TaskContext`, `YieldContext`, and `Environment`. A `ParallelShim` reads arguments from the environment and produces a `Send`-safe `ParallelCall` for `spawn_blocking`. This unified approach means `call_function_inner` only handles `Value::Function`, with a single path for arity checking, parameter type auditing, environment binding, and then dispatching on the implementation variant.

To distinguish the ambiguous case (no Yielding entry) from explicitly unyielding, a new field `has_yielding_entry: Option<bool>` was added to `IshFunction`: `None` for ambiguous, `Some(true)` for yielding, `Some(false)` for unyielding.

The decision register grew from 10 to 12 decisions.

### v6 — Simplified Compiled Function Architecture (2026-04-01)

The reviewer simplified the compiled function architecture introduced in v5, making three inline decisions that reduced complexity.

**Two variants, not three (Decision 12 revised):** The v5 proposal had three `FunctionImplementation` variants: `Interpreted`, `Concurrent(ConcurrentShim)`, and `Parallel(ParallelShim)`, with two distinct shim types. The reviewer collapsed this to two variants: `Interpreted(Statement)` and `Compiled(Shim)`. There is one shim type, not two. A shim is a synchronous function — `Fn(&[Value]) -> Result<Value, RuntimeError>` — that receives already-validated arguments and returns a `Value`. What the shim does internally depends on the function's yielding classification:

- **Unyielding shims** (e.g., `len`, `type_of`) call the underlying logic directly and return a plain `Value`.
- **Yielding shims** spawn work via `spawn_local` and return `Value::Future`.
- **Parallel shims** (e.g., `read_file`, `println`) marshal arguments into `Send`-safe form, spawn via `spawn_blocking`, and use a `spawn_local` bridge to convert native results back to `Value`. They return `Value::Future`.

The key insight the reviewer identified: `spawn_local` and `spawn_blocking` are non-async — they schedule work and return a `JoinHandle` immediately. So the shim is synchronous: it sets up the spawn chain, wraps the handle in `FutureRef`, and returns. The `spawn_local` bridge is where native results are marshalled back into `Value`, but this runs later when the future is awaited.

**ParallelCall and ParallelResult eliminated (new decision):** The v5 proposal introduced `ParallelCall` and `ParallelResult` as intermediate types for marshalling arguments into `Send`-safe form and converting results back. The reviewer eliminated these entirely — the shim handles all marshalling directly in its closure body. There is no intermediate protocol between the shim and any other component.

**Shims are not async (clarification):** The reviewer explicitly stated that shims are not async and do not need to be awaited. In `call_function_inner`, dispatching a compiled function is simply `shim(args)` — no `.await`. The shim returns a `Value` synchronously, which may be `Value::Future` for yielding/parallel functions. The caller (Await handler, Spawn handler, or FunctionCall handler with implied-await logic) handles the future.

These simplifications mean `call_function_inner` for compiled functions is a single synchronous call. The async complexity is entirely encapsulated inside the shim closures, invisible to the dispatch logic.

The decision register remained at 12 decisions (Decision 12 was revised in place, the other two changes were refinements of v5's design).

### Acceptance (2026-04-02)

After six revisions, the proposal was accepted with all 12 decisions resolved and no open decision points remaining. The final design settled on three features:

1. **FutureRef identity equality** — `Rc::ptr_eq` for `Value::Future` comparisons. Simple, correct, and no controversy through any revision.

2. **Grammar-level spawn/await restriction** — `await` and `spawn` take `call_expr` in the grammar, producing `callee + args` in the AST. Invalid forms produce `Incomplete` nodes. E012 is thrown before calling an explicitly unyielding function via `await`; E013 for `spawn`. The ambiguous case (no Yielding entry, non-Future result) passes through.

3. **Compiled function implementation** — `FunctionImplementation` with two variants: `Interpreted(Statement)` and `Compiled(Shim)`. All builtins become `IshFunction` values with compiled shims. `BuiltinFn` and `Value::BuiltinFunction` are eliminated. Shims are synchronous; yielding/parallel shims internally spawn work and return `Value::Future`. Implied await at low assurance preserves backward compatibility.

Two open questions were deferred to future work: function yielding categorization at declaration time, and builtin replacement by the standard library.

## Version Files

- [v1.md](v1.md) — Initial proposal with 6 pending decisions
- [v2.md](v2.md) — All decisions resolved, E012 for await-on-non-future, parallel builtins return Future
- [v3.md](v3.md) — E012 disabled, E013 added, implied await, I/O completion guarantee
- [v4.md](v4.md) — E012 refined to clear-cut cases, builtin–function parity via expanded BuiltinFn
- [v5.md](v5.md) — Grammar restriction, FunctionImplementation enum with 3 variants (Interpreted/Concurrent/Parallel)
- [v6.md](v6.md) — Simplified to 2 FunctionImplementation variants, synchronous shims, eliminated ParallelCall/ParallelResult
