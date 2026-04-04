---
title: "Proposal: Stubbed Analyzer and Yielding/Unyielding Function Refactoring"
category: proposal
audience: [ai-dev]
status: accepted
last-verified: 2026-04-03
depends-on:
  - docs/project/rfp/stubbed-analyzer.md
  - docs/spec/concurrency.md
  - docs/architecture/vm.md
  - docs/project/proposals/concurrency-correctness.md
  - docs/project/proposals/runtime-extraction.md
  - proto/ish-vm/src/interpreter.rs
  - proto/ish-runtime/src/value.rs
---

# Proposal: Stubbed Analyzer and Yielding/Unyielding Function Refactoring

*Generated from [stubbed-analyzer.md](../rfp/stubbed-analyzer.md) on 2026-04-03. Accepted 2026-04-03 (v3).*

---

## Decision Register

All decisions made during design, consolidated here as the authoritative reference.

| # | Decision | Outcome |
|---|----------|---------|
| 1 | Where does analyzer.rs live? | `ish-vm` — close to the interpreter that consumes it; move later if `ish-codegen` needs it |
| 2 | What mechanism tracks yielding/unyielding execution context? | Separate function pairs — context is implicit in which function you call |
| 3 | How are reusable match-arm functions organized? | Extract pure computation into helpers, keep parallel structure for match arms |
| 4 | Do we need separate yielding and unyielding shim variants? | Yes — yielding shims spawn a task and return `Value::Future`; unyielding shims execute synchronously and return the result directly |
| 5 | How does the shim execute the function body without PENDING_INTERP_CALL? | Shim is self-contained — captures VM, body, env, executes directly |
| 6 | What happens to `has_yielding_entry: None`? | Keep `Option<bool>`, treat `None` as unyielding. The data structure will need a more significant refactoring later. |
| 7 | How does `apply` work as a normal compiled function? | Simple shim that calls the passed-in function's shim directly |
| 8 | What about `CommandSubstitution` — yielding or unyielding? | Yielding — it uses `tokio::process::Command` under the hood |
| 9 | What about nested function declarations — does the analyzer recurse into them? | No — inner functions are classified independently |
| 10 | Does the analyzer check function calls against known definitions? | Yes — calls to yielding functions make the caller yielding (implied await). Calls to undefined functions are an error. Forward references and cycles are not supported in the stub. |

---

## Questions and Answers

### Q: How does the current PENDING_INTERP_CALL mechanism work, and why is it a defect?

The mechanism uses a thread-local `RefCell<Option<InterpCall>>` to bridge the synchronous `Shim` type signature (`Fn(&[Value]) -> Result<Value, RuntimeError>`) with the async interpreter. When an interpreted function's shim is called, it stores the function body and captured environment in the thread-local, returns `Value::Null` as a placeholder, and then `call_function_inner` picks up the pending call and executes the body asynchronously.

The defect is that this only works when shims are called from `call_function_inner`. If a compiled function calls an interpreted function's shim directly (which is the whole point of cross-boundary calls), the shim stores the pending call but nobody retrieves it. The `apply` builtin was implemented as a special-case intercept in `call_function_inner` specifically to work around this limitation.

### Q: Do we need yielding and unyielding shim variants?

Yes. The `Shim` type signature stays the same for both variants (`Rc<dyn Fn(&[Value]) -> Result<Value, RuntimeError>>`). What changes is what the shim does internally:

- **Unyielding shim**: Calls `exec_statement_unyielding` synchronously, returns result.
- **Yielding shim**: Spawns a Tokio `spawn_local` task that calls `exec_statement_yielding`, returns `Value::Future`.

No type-level variant is needed.

### Q: What about `apply` returning a `Value::Future` when passed a yielding function?

`apply` is a normal compiled function whose shim calls the passed-in function's shim. If the passed-in function is yielding, its shim returns `Value::Future`. `apply` returns that `Value::Future` as its own result. `apply(yielding_fn, args)` returns a `Future` that can be `await`ed.

---

## Feature 1: Stubbed Code Analyzer

A new `analyzer.rs` module in `ish-vm` that classifies functions as yielding or unyielding at declaration time by walking the AST.

The analyzer immediately classifies a function as yielding if `is_async` is true. Otherwise, it walks the AST body looking for yielding nodes:

- `Expression::Await` — explicit await
- `Expression::Spawn` — explicit spawn
- `Statement::Yield` — explicit yield
- `Expression::CommandSubstitution` — shell command (uses `tokio::process::Command`)
- `Expression::FunctionCall` where the callee resolves to a function with `has_yielding_entry == Some(true)` — implied await propagates yielding classification

The analyzer does **not** recurse into `Statement::FunctionDecl` or `Expression::Lambda` bodies — inner functions are classified independently.

The analyzer requires functions to be defined before they are referenced. Calls to undefined functions produce an error. Forward references and call cycles are not supported — this is a known stub limitation.

Because the analyzer must look up function definitions in the environment, it lives in `ish-vm` and takes an `Environment` parameter. Its `classify_function` returns `Result<YieldingClassification, RuntimeError>`.

---

## Feature 2: Remove the Ambiguous Case

The `has_yielding_entry` field remains `Option<bool>` (to avoid a premature pervasive refactoring), but `None` is now treated as unyielding instead of ambiguous.

Both `Statement::FunctionDecl` and `Expression::Lambda` call the analyzer, mapping `Yielding` → `Some(true)` and `Unyielding` → `Some(false)`. The ambiguous pass-through paths in `await` handling (passing non-`Future` values through when `has_yielding_entry` is `None`) and `spawn` handling are removed. Awaiting or spawning a function with `has_yielding_entry == Some(false)` or `None` throws E012/E013.

The concurrency spec is updated to the two-state model (yielding/unyielding). The open question on function yielding categorization at declaration is closed.

---

## Feature 3: Remove PENDING_INTERP_CALL and Fix Shim Architecture

The `InterpCall` struct, `PENDING_INTERP_CALL` thread-local, all stores, and the retrieval in `call_function_inner` are deleted.

Interpreted function shims become self-contained — each captures the VM (`Rc<RefCell<IshVm>>`), function body (`Statement`), environment (`Environment`), and parameter names at declaration time.

**Unyielding shims** create a `TaskContext`, call `exec_statement_unyielding` synchronously, run defers, convert `ControlFlow` to `Value`, and return the result directly.

**Yielding shims** create a child environment, then call `tokio::task::spawn_local` with an async block that creates a `TaskContext` and `YieldContext`, calls `exec_statement_yielding`, runs defers, and converts `ControlFlow` to `Value`. The shim returns `Ok(Value::Future(FutureRef::new(handle)))`.

`call_function_inner` becomes **synchronous**: arity check, parameter type audit, call the shim, return type audit, return.

The `apply` intercept and all ledger builtin intercepts are removed from `call_function_inner`. Ledger builtins are restructured so their shims capture the VM reference and handle ledger access directly (registered after VM construction).

---

## Feature 4: Yielding/Unyielding Execution Context

Two separate function pairs with no shared context tracking:

- `exec_statement_yielding` / `eval_expression_yielding` — async, returns `Pin<Box<dyn Future<...>>>`, uses `YieldContext` for budget management
- `exec_statement_unyielding` / `eval_expression_unyielding` — synchronous, returns `Result<ControlFlow/Value, RuntimeError>`, no `YieldContext`, no yield checks

The unyielding path is truly synchronous — no `Pin<Box<dyn Future>>`, no `.await`, no Tokio runtime requirement. Unyielding variants enforce runtime invariants: encountering `Await`, `Spawn`, or `Yield` nodes returns an error.

---

## Feature 5: Split eval_expression and exec_statement

Pure synchronous computation is extracted into helper functions (binary operations, variable declaration, assignment, etc.). Both yielding and unyielding variants call the same helpers after evaluating sub-expressions via their respective recursive calls.

The yielding variants are essentially the current code with `eval_expression` renamed to `eval_expression_yielding`. The unyielding variants have the same match structure without async, `.await`, or yield budget checks.

**FunctionCall in yielding context**: Calls `call_function_inner` (sync), then performs implied await — if the result is `Value::Future`, awaits it inline. If not, returns it directly.

**FunctionCall in unyielding context**: Calls `call_function_inner` (sync), returns the result directly.

**Await in yielding context**: Calls `call_function_inner` (sync), then explicitly awaits the `Value::Future` result.

**Spawn in yielding context**: Calls `call_function_inner` (sync). For yielding functions, the shim already spawned a task and returned `Value::Future` — this is the spawn result.

---

## Feature 6: Fix Function Declaration and apply

**Function declaration**: The analyzer classifies the function, then creates either a yielding or unyielding shim based on the classification. `has_yielding_entry` is set to `Some(true)` or `Some(false)`.

**Lambda expressions**: Same treatment — analyzer classifies, appropriate shim created.

**`apply`**: A normal compiled function registered in `builtins.rs`. Its shim extracts the function and argument list from its parameters, then calls `(func.shim)(&arg_list)`. No special-casing in `call_function_inner`. If the passed-in function is yielding, its shim returns `Value::Future`; `apply` returns that directly.

---

## Sequencing

1. Feature 1: Stubbed Code Analyzer
2. Feature 5: Split eval_expression and exec_statement (parallel with 1)
3. Feature 2: Remove the Ambiguous Case (depends on 1)
4. Feature 4: Yielding/Unyielding Execution Context (depends on 5)
5. Feature 3: Remove PENDING_INTERP_CALL (depends on 1, 2, 4, 5)
6. Feature 6: Fix Function Declaration and apply (depends on all above)

---

## Documentation Updates

| File | Changes |
|------|---------|
| [docs/spec/concurrency.md](../../spec/concurrency.md) | Two-state yielding model. Remove ambiguous case. |
| [docs/architecture/vm.md](../../architecture/vm.md) | Self-contained shim architecture. Yielding/unyielding exec variants. Code analyzer. Remove PENDING_INTERP_CALL. |
| [docs/project/open-questions.md](../open-questions.md) | Close function yielding categorization. Partially close code analyzer scope. |
| [GLOSSARY.md](../../../GLOSSARY.md) | Add: code analyzer, yielding classification. Update: function implementation. |
| [docs/architecture/runtime.md](../../architecture/runtime.md) | `None` treated as unyielding. |

---

## Referenced by

- [docs/project/rfp/stubbed-analyzer.md](../rfp/stubbed-analyzer.md)
- [docs/project/proposals/INDEX.md](INDEX.md)
