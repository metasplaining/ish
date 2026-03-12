---
title: "Proposal: Defer Scoping — Function-Scoped vs. Block-Scoped"
category: proposal
audience: [all]
status: proposal
last-verified: 2026-03-12
depends-on: [docs/user-guide/error-handling.md, docs/project/proposals/error-handling.md, docs/spec/agreement.md]
---

# Proposal: Defer Scoping — Function-Scoped vs. Block-Scoped

*Generated from `defer` on 2026-03-12.*

---

## Questions and Answers

### Q: Why should defer be function-scoped rather than block-scoped?

The `defer` file states:

> defer should be function scoped. The value of a defer primitive is to use control flow to allocate resources in some order that is unpredictable at compile time, then release them in reverse order. Unwinding at the block scope prevents this.

The core argument is about **dynamic resource accumulation**. When resources are acquired inside control flow (loops, conditionals, match arms), the set of resources that need cleanup is not known until runtime. Function-scoped defer naturally handles this because all defers — regardless of which block they were registered in — accumulate into a single LIFO stack that unwinds at function exit.

With block-scoped defer, a resource acquired inside a conditional block is released when that block ends — not when the function ends. This means:

1. **You can't use a conditionally-acquired resource after the block.** The resource is already cleaned up.
2. **Loops that accumulate resources release them each iteration.** If you `defer close(conn)` inside a loop that opens N connections, block scoping closes each connection at the end of each iteration, while function scoping keeps all N connections open and closes them all at function exit.
3. **Conditional cleanup order is lost.** If blocks A and B each defer cleanup, the block-scoped order depends on nesting structure. Function-scoped defer preserves the true chronological order of acquisition.

### Q: What does Go do, and why?

Go's `defer` is function-scoped. A deferred call is pushed onto a per-function stack and executes when the surrounding function returns — not when the enclosing block (if/for/switch) exits. Go's rationale (from the [Go Blog](https://go.dev/blog/defer-panic-and-recover)):

- Defer exists primarily for cleanup paired with acquisition. Since functions are the unit of resource ownership, function scope is the natural boundary.
- Block-scoped defer would be surprising in loops: developers expect `defer f.Close()` inside a loop to run at function exit, not on each iteration. (In practice, Go developers who want per-iteration cleanup use a helper function or explicit close.)
- Function scope makes defer predictable: every defer in a function runs exactly once, at function exit, in LIFO order.

### Q: What do other languages do?

| Language | Mechanism | Scope |
|----------|-----------|-------|
| Go | `defer` | Function |
| Swift | `defer` | Block (enclosing scope) |
| Zig | `defer` / `errdefer` | Block |
| Odin | `defer` | Block |
| C++ | Destructors (RAII) | Block (object lifetime) |
| Rust | `Drop` (RAII) | Block (object lifetime) |
| Java | `try-with-resources` | Block |
| Python | `with` / `contextmanager` | Block |

The split is roughly: Go chose function scope; most others chose block scope (or block-scoped RAII). However, the block-scoped languages mostly use RAII or `with` blocks rather than `defer` — their cleanup is tied to object lifetime, not arbitrary statements.

**Key insight:** In RAII languages, block scoping works because the *object* owns the cleanup. Moving the object out of the block extends the cleanup. In defer-based languages, the *statement* owns the cleanup, so it can't be moved. This makes block-scoped defer less flexible than block-scoped RAII.

---

## Feature: Function-Scoped Defer

### Issues to Watch Out For

- **Backwards incompatibility.** The prototype currently implements block-scoped defer. Existing tests (`test_defer_executes_on_block_exit`, `test_defer_lifo_order`) verify block-scoped behavior. Changing to function scope will alter observable behavior and break tests.
- **Interaction with `with` blocks.** The `with` block is inherently block-scoped — resources are closed when the `with` block exits. If `defer` moves to function scope but `with` stays block-scoped, there's an asymmetry. This is arguably correct (different tools for different purposes) but needs clear documentation.
- **Nested functions and lambdas.** Function-scoped defer must be clear about which function. A `defer` inside a lambda should bind to the lambda, not the outer function. Go handles this correctly — each function invocation has its own defer stack.
- **Defer inside loops.** With function scoping, `defer` inside a loop accumulates N deferred calls. This is the *desired* behavior per the `defer` file, but it can be surprising to developers from Swift/Zig/Rust backgrounds who expect per-iteration cleanup. It can also cause resource exhaustion if the loop is large.
- **Loss of block-scoped cleanup.** Some use cases genuinely want block-scoped cleanup without introducing a `with` block. Developers who want "run this when this block exits" would need to either use `with` or extract the block into a function.

### Critical Analysis

**Alternative A: Function-scoped defer (as proposed in the `defer` file)**
- Pros:
  - Matches the core use case: accumulate resources in unpredictable control flow, release in reverse order at function exit.
  - Follows Go's proven semantics. Go's defer is widely used and well-understood.
  - Predictable: all defers in a function run at function exit, period.
  - Avoids the problem of block-scoped defer releasing resources too early when they're needed across blocks.
- Cons:
  - Defer in loops accumulates unboundedly — can exhaust resources.
  - No block-scoped cleanup primitive beyond `with`. Developers wanting arbitrary block-exit cleanup must wrap in a function.
  - Breaking change from current prototype behavior.

**Alternative B: Block-scoped defer (current implementation)**
- Pros:
  - Matches Swift, Zig, Odin — familiar to developers from those languages.
  - Natural fit with block-scoped `with` — both cleanup mechanisms work the same way.
  - Defer in loops cleans up each iteration, preventing resource accumulation.
  - Already implemented and tested in the prototype.
- Cons:
  - Prevents the dynamic resource accumulation pattern described in the `defer` file.
  - Resources acquired in conditional blocks can't outlive those blocks.
  - If a developer wants function-scoped cleanup, they must restructure their code.

**Alternative C: Block-scoped defer with `func defer` for function scope**
Introduce two forms: `defer` (block-scoped, current behavior) and `func defer` (function-scoped).
- Pros:
  - Both use cases are supported explicitly.
  - No breaking change — `defer` keeps current semantics.
  - Developers choose the right scope for each situation.
- Cons:
  - Two mechanisms for overlapping use cases — complexity.
  - The `func defer` syntax is not precedented in other languages.
  - May confuse developers about which to use by default.

**Alternative D: Function-scoped defer with explicit block-scoped cleanup via immediately-invoked functions**
Use function-scoped defer as the default. For block-scoped cleanup, developers invoke a helper:
```
// Block-scoped cleanup via helper function
fn withTemp() {
    let tmp = createTemp();
    defer cleanup(tmp);
    // use tmp...
}  // defer runs here because withTemp returns

// Call site:
withTemp();
// tmp is cleaned up
```
- Pros:
  - Clean separation: `defer` is always function-scoped. Block-scoped cleanup uses `with` or helper functions.
  - Follows Go's idiom exactly (Go FAQ recommends extracting to a function for per-iteration cleanup).
  - No syntactic complexity.
- Cons:
  - Requires extracting code into functions for block-scoped cleanup — this can be verbose.
  - Helper functions for cleanup are a pattern, not a language feature.

### Recommendation

**Alternative A (function-scoped defer)**, supplemented by Alternative D's idiom for block-scoped cases.

The argument in the `defer` file is compelling: the primary value of `defer` is managing resources acquired in dynamic, unpredictable order. Block scoping prevents this use case. The `with` block already covers the common case of block-scoped resource cleanup (closeable objects). For arbitrary block-scoped cleanup, extracting to a helper function is a reasonable (and Go-proven) idiom.

The asymmetry between `defer` (function-scoped) and `with` (block-scoped) is a feature, not a bug — they serve different purposes:
- `with` = "this object needs deterministic cleanup when I'm done with it in this scope"
- `defer` = "this action needs to happen when the function exits, regardless of how"

### Proposed Implementation

**Step 1: Modify the interpreter's Block handling.**

Currently in [proto/ish-vm/src/interpreter.rs](../../proto/ish-vm/src/interpreter.rs), the `Block` statement collects defers and runs them at block exit. Change this so that `Block` propagates defer statements upward to the enclosing function rather than executing them.

Specifically:
- Add a `deferred: &mut Vec<Statement>` parameter to `exec_statement` (or use a per-function defer stack on the interpreter).
- In `Block` handling, when encountering a `Defer`, push it onto the function-level defer stack instead of a block-local vector.
- In `call_function`, after the body executes, run all accumulated defers in LIFO order.

**Step 2: Update `call_function`.**

In [proto/ish-vm/src/interpreter.rs](../../proto/ish-vm/src/interpreter.rs), the `call_function` method must:
1. Initialize an empty defer stack for the function invocation.
2. Pass it through to `exec_statement` calls.
3. After the body completes (normally, via return, or via throw), execute all deferred statements in LIFO order.
4. Deferred statements should not override in-flight control flow (same as current behavior).

**Step 3: Handle top-level code.**

Code outside any function (e.g., in the shell/REPL) needs its own defer stack. The entry point for program execution should act as an implicit function boundary for defer purposes.

**Step 4: Update tests.**

- `test_defer_executes_on_block_exit` — update to verify defer runs at function exit, not block exit.
- `test_defer_lifo_order` — should still pass (LIFO order is preserved).
- Add new test: `test_defer_function_scoped` — defer inside a conditional block runs at function exit.
- Add new test: `test_defer_loop_accumulates` — defer inside a loop accumulates N deferred calls.
- Add new test: `test_defer_lambda_boundary` — defer inside a lambda binds to the lambda, not the outer function.

**Step 5: Update documentation.**

- [docs/user-guide/error-handling.md](../../user-guide/error-handling.md) — update the Defer section to describe function scoping.
- [docs/project/open-questions.md](../../project/open-questions.md) — close the defer scoping question.

**Files affected:**
- `proto/ish-vm/src/interpreter.rs` — core change: defer collection and execution
- `proto/ish-vm/src/interpreter.rs` (tests) — update and add tests
- `docs/user-guide/error-handling.md` — update defer documentation
- `docs/project/open-questions.md` — close the open question

### Decisions

**Decision:** Should defer be function-scoped (Go-style) or remain block-scoped (current implementation)?
--> function-scoped

**Decision:** If function-scoped, should there also be a block-scoped defer variant (Alternative C's `func defer` / `block defer` split)?
--> No.

**Decision:** Should defer inside a loop be considered a footgun worth warning about (lint/analyzer warning for defer in loops)?
--> No.

---

## Documentation Updates

The following documentation files will be affected:

- [docs/user-guide/error-handling.md](../../user-guide/error-handling.md) — update the Defer section to reflect new scoping rules
- [docs/project/open-questions.md](../../project/open-questions.md) — close: "Should `defer` follow Go's function-scoped semantics or be block-scoped?"
- [docs/project/proposals/error-handling.md](error-handling.md) — add cross-reference to this proposal for the defer scoping decision
- [docs/architecture/vm.md](../../architecture/vm.md) — update interpreter documentation for defer execution model
- [GLOSSARY.md](../../../GLOSSARY.md) — review defer definition for accuracy after scoping change

Remember to update `## Referenced by` sections in all affected files.

---

## History Updates

- [ ] Add `docs/project/history/2026-03-12-defer-scoping.md` after decisions are made
- [ ] Update `docs/project/history/INDEX.md`

---

## Referenced by

- [docs/project/proposals/INDEX.md](INDEX.md)
