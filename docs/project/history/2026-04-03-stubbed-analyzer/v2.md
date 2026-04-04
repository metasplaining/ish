---
title: "Proposal: Stubbed Analyzer and Yielding/Unyielding Function Refactoring"
category: proposal
audience: [ai-dev]
status: proposal
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

*Generated from [stubbed-analyzer.md](../rfp/stubbed-analyzer.md) on 2026-04-03. Revised 2026-04-03 (v2).*

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

Yes. The `Shim` type is `Rc<dyn Fn(&[Value]) -> Result<Value, RuntimeError>>` — a synchronous function. The type signature stays the same for both variants. What changes is what the shim does internally:

- **Unyielding shim**: Calls `exec_statement_unyielding` synchronously, returns result.
- **Yielding shim**: Spawns a Tokio `spawn_local` task that calls `exec_statement_yielding`, returns `Value::Future`.

This difference is behavioral — no type-level variant is needed.

### Q: What about `apply` returning a `Value::Future` when passed a yielding function?

If `apply` is a normal compiled function whose shim calls the passed-in function's shim, and the passed-in function is yielding, then the yielding function's shim will return `Value::Future`. `apply` just returns that `Value::Future` as its own result. This is the desired behavior — `apply(yielding_fn, args)` returns a `Future` that can be `await`ed.

---

## Feature 1: Stubbed Code Analyzer

### Issues to Watch Out For

1. **Nested functions.** A function body may contain inner function declarations or lambdas. The analyzer does not recurse into them — an inner function's yielding is independent of its parent's (Decision 9).
2. **Conditional yielding.** A function might have `await` in one branch of an `if` but not another. The stub analyzer finds *any* yielding node in the body — it does not reason about reachability.
3. **`CommandSubstitution`.** The `$()` syntax executes a shell command via `tokio::process::Command`, which is inherently async. It is treated as yielding (Decision 8).
4. **`is_async` flag.** Functions declared with `async fn` have `is_async: true` in the AST. This immediately classifies as yielding without walking the body.
5. **Calls to yielding functions.** When a function calls another function without explicit `await` or `spawn`, the interpreter performs an implied await if the callee is yielding. This means the caller is also yielding. The analyzer must check function calls against the known set of already-declared functions (Decision 10).
6. **Undefined functions.** The stub analyzer requires functions to be defined before they are referenced. If a function body calls an undefined function, the analyzer throws an error. Forward references and call cycles are not supported in the stub — this is a known limitation that will be addressed later.
7. **VM access.** Because the analyzer must look up the yielding classification of already-declared functions, it needs access to the VM environment. This reinforces the decision to place the analyzer in `ish-vm` (Decision 1).

### Proposed Implementation

Create `proto/ish-vm/src/analyzer.rs`. The analyzer lives in `ish-vm` (Decision 1) so it has access to the `Environment` and can look up function definitions.

```rust
use ish_ast::{Statement, Expression};

pub enum YieldingClassification {
    Yielding,
    Unyielding,
}

/// Walk a function body and determine whether the function is yielding or unyielding.
///
/// This is a stub implementation. It checks for:
/// - The `is_async` flag on the function declaration
/// - Presence of `await`, `spawn`, `yield`, or `$()` in the body
/// - Calls to functions that are already declared as yielding (implied await)
///
/// Limitations:
/// - Does not support forward references — called functions must be declared first
/// - Does not support call cycles
/// - Throws an error if a function body calls an undefined function
pub fn classify_function(
    body: &Statement,
    is_async: bool,
    env: &Environment,
) -> Result<YieldingClassification, RuntimeError> {
    if is_async {
        return Ok(YieldingClassification::Yielding);
    }
    if contains_yielding_node(body, env)? {
        Ok(YieldingClassification::Yielding)
    } else {
        Ok(YieldingClassification::Unyielding)
    }
}
```

The `contains_yielding_node` function walks the AST recursively, returning `true` if it finds:

- `Expression::Await` — explicit await
- `Expression::Spawn` — explicit spawn
- `Statement::Yield` — explicit yield
- `Expression::CommandSubstitution` — shell command (uses `tokio::process::Command`)
- `Expression::FunctionCall` where the callee resolves to a function with `has_yielding_entry == Some(true)` — implied await makes the caller yielding

It does **not** recurse into `Statement::FunctionDecl` or `Expression::Lambda` bodies — those are independent functions with their own classifications (Decision 9).

When the function encounters a `FunctionCall` whose callee is an `Identifier`, it looks up the identifier in the environment. If the identifier is not defined, it returns an error. If the identifier resolves to a function, it checks `has_yielding_entry`. If the callee is yielding, the containing function is also yielding.

---

## Feature 2: Remove the Ambiguous Case

### Issues to Watch Out For

1. **Specification changes.** The concurrency spec documents the three-state model: `Some(true)`, `Some(false)`, `None`. The `None` case and its pass-through behavior must be removed from the spec.
2. **Open questions.** The open question on "Function yielding categorization at declaration" is resolved by this change — it should be marked done.
3. **Test breakage.** Any tests that rely on the ambiguous pass-through behavior (calling unclassified functions with `await` and getting non-`Future` values passed through) will break.
4. **Lambda classification.** Lambdas currently always get `has_yielding_entry: None`. They must go through the analyzer too.

### Proposed Implementation

The `has_yielding_entry` field on `IshFunction` remains `Option<bool>` (Decision 6). The data structure will need a more significant refactoring later — there is no point in doing half the job now. However, the *semantic meaning* of `None` changes: it is treated as unyielding instead of ambiguous.

1. **Update the analyzer call sites.** Both `Statement::FunctionDecl` and `Expression::Lambda` call `classify_function`, which returns `Yielding` or `Unyielding`. These map to `Some(true)` or `Some(false)` on `has_yielding_entry`.
2. **Remove ambiguous-case handling.** In `call_function_inner`, `await` handling, and `spawn` handling, remove the `None` pass-through paths. If `has_yielding_entry` is `None` (which should no longer happen for newly declared functions), treat it as unyielding — `await` on it throws E012, `spawn` on it throws E013.
3. **Update builtins.** Existing builtins that use `None` for `has_yielding_entry` should be updated to `Some(false)` where appropriate, or left as `None` (treated as unyielding by the updated code).
4. **Update the concurrency spec.** Remove the three-state model. Document the two-state model (yielding/unyielding). Note that `Option<bool>` is retained for now with `None` treated as unyielding.
5. **Close the open question** on function yielding categorization at declaration.
6. **Update the concurrency spec table:**

| Callee classification | `await` behavior | `spawn` behavior |
|----------------------|------------------|------------------|
| Yielding (`has_yielding_entry: Some(true)`) | Normal await — call, then await the resulting `Future` | Normal spawn — spawn a task that calls the function |
| Unyielding (`has_yielding_entry: Some(false)` or `None`) | **E012** — thrown before calling | **E013** — thrown before calling |

---

## Feature 3: Remove PENDING_INTERP_CALL and Fix Shim Architecture

### Issues to Watch Out For

1. **Shim type is synchronous.** `Shim = Rc<dyn Fn(&[Value]) -> Result<Value, RuntimeError>>`. Yielding shims need to spawn a `tokio::task::spawn_local` to return a `Value::Future`. This is safe because the shim is called from within the interpreter which already runs on the Tokio `LocalSet`.
2. **Shim captures.** Yielding shims capture the function body (`Statement`), the declaration-time environment (`Environment`), parameter names, and a reference to the VM (`Rc<RefCell<IshVm>>`). This is a lot of captured state but it's all `Clone` or `Rc`-shared.
3. **TaskContext and YieldContext.** For yielding shims that spawn a new task, a new `TaskContext` and `YieldContext` are created inside the spawned task. For unyielding shims, a new `TaskContext` is created for defer stack management.
4. **Return value handling.** The ControlFlow → Value conversion (`Return(v)` → `v`, `ExprValue(v)` → `v`, `None` → `Null`, `Throw(v)` → error) moves into the shim.
5. **Parameter type auditing and return type auditing.** These remain in `call_function_inner` as a wrapper around the shim call (Decision 5).
6. **Defer stack.** For yielding shims, the spawned task manages its own defer stack. For unyielding shims, defer management happens in the shim's own `TaskContext`.

### Proposed Implementation

The shim is self-contained — it captures the VM, body, and environment, and executes the function body directly (Decision 5). Yielding shims spawn a task and return `Value::Future`; unyielding shims execute synchronously (Decision 4).

**Unyielding interpreted function shim:**
```rust
let shim: Shim = Rc::new(move |args: &[Value]| {
    let call_env = captured_env.child();
    for (param, arg) in captured_params.iter().zip(args.iter()) {
        call_env.define(param.clone(), arg.clone());
    }
    let mut task = TaskContext::new();
    let result = Interpreter::exec_statement_unyielding(
        &captured_vm, &mut task, &captured_body, &call_env
    )?;
    // Run defers
    Interpreter::run_defers_unyielding(&captured_vm, &mut task);
    match result {
        ControlFlow::Return(v) | ControlFlow::ExprValue(v) => Ok(v),
        ControlFlow::None => Ok(Value::Null),
        ControlFlow::Throw(v) => Err(RuntimeError::thrown(v)),
    }
});
```

**Yielding interpreted function shim:**
```rust
let shim: Shim = Rc::new(move |args: &[Value]| {
    let call_env = captured_env.child();
    for (param, arg) in captured_params.iter().zip(args.iter()) {
        call_env.define(param.clone(), arg.clone());
    }
    let body = captured_body.clone();
    let vm = captured_vm.clone();
    let env = call_env.clone();
    let future = FutureRef::new(async move {
        let mut task = TaskContext::new();
        let mut yc = YieldContext::new_yielding();
        task.push_defer_frame();
        let result = Interpreter::exec_statement_yielding(
            &vm, &mut task, &mut yc, &body, &env
        ).await;
        Interpreter::pop_and_run_defers_static(&vm, &mut task, &mut yc).await;
        match result? {
            ControlFlow::Return(v) | ControlFlow::ExprValue(v) => Ok(v),
            ControlFlow::None => Ok(Value::Null),
            ControlFlow::Throw(v) => Err(RuntimeError::thrown(v)),
        }
    });
    Ok(Value::Future(future))
});
```

**Simplified `call_function_inner`:**

With shims handling body execution, `call_function_inner` becomes synchronous:

```rust
fn call_function_inner(
    vm: &Rc<RefCell<IshVm>>,
    func: &Value,
    args: &[Value],
) -> Result<Value, RuntimeError> {
    match func {
        Value::Function(f) => {
            // Arity check
            if !f.params.is_empty() && args.len() != f.params.len() {
                return Err(arity_error(f, args));
            }
            // Parameter type audit
            for (i, (param_name, arg)) in f.params.iter().zip(args.iter()).enumerate() {
                let param_type = f.param_types.get(i).and_then(|t| t.as_ref());
                Self::audit_type_annotation(vm, param_name, arg, param_type)?;
            }
            // Call the shim — it handles everything
            let result = (f.shim)(args)?;
            // Return type audit
            Self::audit_type_annotation(vm,
                &format!("return of '{}'", f.name.as_deref().unwrap_or("anonymous")),
                &result,
                f.return_type.as_ref(),
            )?;
            Ok(result)
        }
        _ => Err(not_callable_error(func)),
    }
}
```

The async handling (awaiting `Value::Future` results) moves to `eval_expression_yielding`'s `FunctionCall` and `Await` arms. The `await` expression calls `call_function_inner` (sync), gets a `Value::Future`, then `.await`s the future inline.

**Removing PENDING_INTERP_CALL:**

1. Delete the `InterpCall` struct
2. Delete the `PENDING_INTERP_CALL` thread-local
3. Delete all stores into the thread-local (in function declaration and lambda shim creation)
4. Delete the retrieval in `call_function_inner`

**Removing ledger builtin intercepts:**

The special-case intercepts in `call_function_inner` for `active_standard`, `feature_state`, `has_standard`, `has_entry_type`, `ledger_state`, `has_entry`, and `apply` are removed. These are all compiled builtins with proper shims that work through normal dispatch.

---

## Feature 4: Yielding/Unyielding Execution Context

### Proposed Implementation

The execution context is implicit in which function is called (Decision 2). There are two separate function pairs:

- `exec_statement_yielding` / `eval_expression_yielding` — async, returns `Pin<Box<dyn Future<...>>>`, receives and uses `YieldContext` for budget management
- `exec_statement_unyielding` / `eval_expression_unyielding` — synchronous, returns `Result<ControlFlow, RuntimeError>` / `Result<Value, RuntimeError>`, no `YieldContext`, no yield checks

The unyielding path is truly synchronous — no `Pin<Box<dyn Future>>`, no `.await`, no Tokio runtime requirement. This is the whole point of the split.

The `YieldContext` is still passed to yielding functions for budget management. Unyielding functions receive a `TaskContext` only (for defer stack management).

Unyielding variants enforce runtime invariants for defense-in-depth: encountering `Await`, `Spawn`, or `Yield` nodes in unyielding mode returns an error (the analyzer should have prevented this, but we check anyway).

---

## Feature 5: Split eval_expression and exec_statement

### Issues to Watch Out For

1. **Code duplication.** The match arms for `exec_statement` span ~600 lines. The match arms for `eval_expression` span ~300 lines. Without careful extraction, the yielding/unyielding variants would double this.
2. **Reusable helpers.** Most match arm logic is the same between yielding and unyielding — variable lookup, assignment, object construction, arithmetic, etc. The only differences are: (a) recursive calls use the yielding or unyielding variant, (b) yield budget checks, (c) `await`/`spawn`/`yield` handling.
3. **`call_function_inner` is synchronous.** Both variants call it the same way. The difference is what happens after: the yielding variant handles implied await, the unyielding variant returns the result directly.

### Proposed Implementation

Pure computation is extracted into helpers; the match arm structure is kept in both variants (Decision 3).

**Step 1: Extract synchronous helpers** for each match arm where the arm body does significant work beyond recursive evaluation:

```rust
// Binary operation logic (after both operands are evaluated)
fn apply_binary_op(op: &BinaryOp, left: &Value, right: &Value) -> Result<Value, RuntimeError> { ... }

// Variable declaration (after the initializer is evaluated)
fn define_variable(env: &Environment, name: &str, value: Value, type_ann: &Option<TypeAnnotation>) -> Result<ControlFlow, RuntimeError> { ... }

// Assignment (after value is evaluated, handles property/index targets)
fn perform_assignment(env: &Environment, target: &Expression, value: Value) -> Result<ControlFlow, RuntimeError> { ... }
```

**Step 2: Create the four variants.** Each variant's match structure follows the same pattern — evaluate sub-expressions (via the yielding or unyielding recursive call), then call the extracted helper:

```rust
// Yielding (async) — essentially the current code with renamed recursive calls
fn exec_statement_yielding<'a>(
    vm: &'a Rc<RefCell<IshVm>>,
    task: &'a mut TaskContext,
    yc: &'a mut YieldContext,
    stmt: &'a Statement,
    env: &'a Environment,
) -> Pin<Box<dyn Future<Output = Result<ControlFlow, RuntimeError>> + 'a>> {
    Box::pin(async move {
        match stmt {
            Statement::VariableDecl { name, value, .. } => {
                let val = Self::eval_expression_yielding(vm, task, yc, value, env).await?;
                define_variable(env, name, val, type_ann)
            }
            // ...
        }
    })
}

// Unyielding (sync) — same structure, no async, no yield checks
fn exec_statement_unyielding(
    vm: &Rc<RefCell<IshVm>>,
    task: &mut TaskContext,
    stmt: &Statement,
    env: &Environment,
) -> Result<ControlFlow, RuntimeError> {
    match stmt {
        Statement::VariableDecl { name, value, .. } => {
            let val = Self::eval_expression_unyielding(vm, task, value, env)?;
            define_variable(env, name, val, type_ann)
        }
        // ...
    }
}
```

**Step 3: Unyielding variants return errors for async-only constructs:**

```rust
// In eval_expression_unyielding:
Expression::Await { .. } => Err(RuntimeError::system_error(
    "await not allowed in unyielding context",
    ErrorCode::AwaitUnyielding,
)),
Expression::Spawn { .. } => Err(RuntimeError::system_error(
    "spawn not allowed in unyielding context",
    ErrorCode::SpawnUnyielding,
)),

// In exec_statement_unyielding:
Statement::Yield => Err(RuntimeError::system_error(
    "yield not allowed in unyielding context",
    ErrorCode::YieldUnyielding,
)),
```

---

## Feature 6: Fix Function Declaration and apply

### Issues to Watch Out For

1. **Capturing the VM reference.** Yielding shims need `Rc<RefCell<IshVm>>` to call `exec_statement_yielding`. The VM is accessed via `Rc<RefCell<IshVm>>`, which is `Clone`, so this works.
2. **`apply` yielding classification.** `apply` itself is unyielding — it calls a shim synchronously. The result may be a `Value::Future`, but `apply` does not unwrap it.
3. **Existing cross-boundary tests.** The tests in `proto/ish-tests/functions/cross_boundary.sh` currently test `apply(fn, [args])` with unyielding lambdas. New tests must include yielding functions.
4. **Ledger builtin intercepts.** The special-case intercepts in `call_function_inner` for `active_standard`, `feature_state`, etc. are removed — they work through normal shim dispatch.
5. **Static method visibility.** `exec_statement_yielding` and `exec_statement_unyielding` are static methods on `Interpreter`. They need to be `pub(crate)` so shim closures can call them.

### Proposed Implementation

**`apply` as a normal compiled function** (Decision 7):

```rust
fn apply_shim(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::system_error(
            "apply expects 2 arguments: a function and a list of arguments",
            ErrorCode::ArgumentCountMismatch,
        ));
    }
    let func = &args[0];
    let arg_list = match &args[1] {
        Value::List(list) => list.borrow().iter().cloned().collect::<Vec<_>>(),
        _ => return Err(RuntimeError::system_error(
            "apply expects a list as the second argument",
            ErrorCode::TypeMismatch,
        )),
    };
    match func {
        Value::Function(f) => (f.shim)(&arg_list),
        _ => Err(RuntimeError::system_error(
            format!("cannot call {}", func.type_name()),
            ErrorCode::NotCallable,
        )),
    }
}
```

No special-casing in `call_function_inner`. No need for `vm`, `task`, or `yc`. The shim just calls the passed-in function's shim. If the passed-in function is yielding, its shim returns `Value::Future`; if unyielding, it returns the result directly.

**Function declaration update** (`Statement::FunctionDecl`):

1. Run the analyzer: `analyzer::classify_function(body, is_async, env)`.
2. Based on classification, create either a yielding or unyielding shim.
3. Set `has_yielding_entry` to `Some(true)` or `Some(false)`.

**Lambda expression update** (`Expression::Lambda`):

1. Run the analyzer: `analyzer::classify_function(body, is_async, env)`.
2. Create the appropriate shim variant.
3. Set `has_yielding_entry` to `Some(true)` or `Some(false)`.

**New test cases:**

```bash
# Yielding function through apply
output=$(run_ish 'async fn slow(x) { return x + 1 }; let result = await apply(slow, [10]); println(result)')
assert_output "apply yielding" "11" "$output"

# Verify apply returns Future for yielding functions
output=$(run_ish 'async fn slow(x) { return x + 1 }; let f = apply(slow, [10]); println(type_of(f))')
assert_output "apply returns future" "Future" "$output"

# Unyielding function through apply (existing tests continue to pass)
output=$(run_ish 'println(apply((x) => x + 1, [10]))')
assert_output "apply unyielding" "11" "$output"
```

---

## Sequencing and Dependencies

The features must be implemented in this order:

1. **Feature 1: Stubbed Code Analyzer** — No dependencies. Creates `analyzer.rs`. Requires access to the `Environment` for function lookup.
2. **Feature 5: Split eval_expression and exec_statement** — Can proceed in parallel with Feature 1. Extract helpers, create yielding/unyielding variants.
3. **Feature 2: Remove the Ambiguous Case** — Depends on Feature 1 (analyzer must classify all functions). Updates `None` handling to treat as unyielding.
4. **Feature 4: Yielding/Unyielding Execution Context** — Depends on Feature 5 (the context is implicit in which function is called).
5. **Feature 3: Remove PENDING_INTERP_CALL** — Depends on Features 1, 2, 4, 5. Rewrites shim creation with self-contained shims.
6. **Feature 6: Fix Function Declaration and apply** — Depends on all above. Final integration.

---

## Documentation Updates

The following documentation files will need updates:

| File | Changes |
|------|---------|
| [docs/spec/concurrency.md](../../spec/concurrency.md) | Remove three-state yielding model. Document two-state (yielding/unyielding). Remove ambiguous case from classification table. Note that `Option<bool>` is retained with `None` treated as unyielding. |
| [docs/architecture/vm.md](../../architecture/vm.md) | Remove PENDING_INTERP_CALL documentation. Document self-contained shim architecture. Document yielding/unyielding exec variants. Document the code analyzer. |
| [docs/project/open-questions.md](../open-questions.md) | Close "Function yielding categorization at declaration" question. Close "Code analyzer scope" question (partially — stub only). |
| [GLOSSARY.md](../../../GLOSSARY.md) | Add: `code analyzer`, `yielding classification`. Update: `function implementation` (now with self-contained shims). |
| [AGENTS.md](../../../AGENTS.md) | Update build/test commands if needed. |
| [docs/architecture/runtime.md](../../architecture/runtime.md) | Update `IshFunction` documentation — `None` now treated as unyielding, not ambiguous. |

Remember to update `## Referenced by` sections in all affected files.

---

## History Updates

- [ ] Create `docs/project/history/2026-04-03-stubbed-analyzer/` directory
- [ ] Add `summary.md` with narrative prose
- [ ] Update [docs/project/history/INDEX.md](../history/INDEX.md)

---

## Referenced by

- [docs/project/rfp/stubbed-analyzer.md](../rfp/stubbed-analyzer.md)
- [docs/project/proposals/INDEX.md](INDEX.md)
