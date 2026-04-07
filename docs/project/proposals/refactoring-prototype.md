---
title: "Proposal: Prototype Code Quality Refactoring"
category: proposal
audience: [ai-dev, human-dev]
status: accepted
last-verified: 2026-04-06
depends-on:
  - docs/project/rfp/refactoring-prototype.md
  - docs/project/proposals/stubbed-analyzer.md
  - proto/ish-vm/src/interpreter.rs
  - proto/ish-vm/src/reflection.rs
  - proto/ish-vm/src/builtins.rs
  - proto/ish-parser/src/ast_builder.rs
---

# Proposal: Prototype Code Quality Refactoring

*Generated from [refactoring-prototype.md](../rfp/refactoring-prototype.md) on 2026-04-06.
Accepted inline decisions incorporated in v2.*

---

## Decision Register

All decisions made during design, consolidated here as the authoritative reference.

| # | Decision | Outcome |
|---|----------|---------|
| 1 | Should the yielding/unyielding interpreter arms be reduced by extracting shared helpers? | Yes — extract the six helpers listed in H1 |
| 2 | Which shared helpers should be extracted? | `eval_literal`, `eval_unary_op`, `apply_property_read`, `apply_index_read`, `apply_property_write`, `apply_index_write` |
| 3 | How broadly to fix `ast_builder.rs` unwraps? | Non-grammar-structural sites only (numeric parse overflow, `lines.last()`, `value.unwrap()` in `build_var_decl`) |
| 4 | Should `register_ast_builtins` be reduced with a helper? | Yes — introduce `simple_ast_builtin` |
| 5 | Is the `ast_<kind>` naming-convention dependency acceptable? | Yes |
| 6 | Should `arity` and `new_builtin` wrappers be added to builtins.rs? | Yes |
| 7 | Should `scan_to_close_brace` be extracted from `interpolate_shell_quoted`? | Yes |
| 8 | Apply exit-code sentinel constant (L1)? | Yes |
| 9 | Address Display split (L2)? | Deferred |

---

## Scope Note

The accepted [stubbed-analyzer proposal](stubbed-analyzer.md) (Decision 3) established that
the yielding/unyielding split should use "parallel structure for match arms" while extracting
"pure computation into helpers." All helper extractions in H1 are consistent with that
decision — they extract bodies that are already *identical* between the two variants. The
parallel match structure is preserved.

---

## Feature H1: Extract Shared Helpers from Yielding/Unyielding Interpreter Pair

### Background

`exec_statement_yielding` (667 lines) and `exec_statement_unyielding` (498 lines) share a
large number of match arms whose bodies are structurally identical — the only difference
between the two variants is which evaluator is called for sub-expressions. The same pattern
holds for `eval_expression_yielding` (437 lines) and `eval_expression_unyielding` (277 lines).

Research into both variants identified six arms whose post-evaluation bodies are character-
for-character identical and can be extracted as pure sync helpers without changing any
observable behaviour.

### Six Helpers to Extract

**From `eval_expression_*` (Expression evaluation):**

1. `fn eval_literal(lit: &Literal) -> Value`
   Extracts the six-arm `Literal` match at the top of both `eval_expression_*` functions.
   Location in yielding: line ~946. Location in unyielding: line ~2622.

2. `fn eval_unary_op(op: &UnaryOperator, val: Value) -> Result<Value, RuntimeError>`
   Extracts the `UnaryOperator` match body (Not / Negate / Try) inside `Expression::UnaryOp`.
   The arms are word-for-word identical. Location in yielding: line ~982. In unyielding: ~2657.

3. `fn apply_property_read(obj: Value, property: &str) -> Result<Value, RuntimeError>`
   Extracts the post-evaluation match body of `Expression::PropertyAccess`. Both variants
   receive `obj: Value` from their respective evaluator then execute identical match logic.
   Location in yielding: ~1073. In unyielding: ~2703.

4. `fn apply_index_read(obj: Value, idx: Value) -> Result<Value, RuntimeError>`
   Extracts the post-evaluation match body of `Expression::IndexAccess` (List+Int bounds
   check, Object+String lookup, error for other combinations). Location in yielding: ~1088.
   In unyielding: ~2720.

**From `exec_statement_*` (Statement execution):**

5. `fn apply_property_write(obj: &Value, property: &str, val: Value) -> Result<(), RuntimeError>`
   Extracts the `AssignTarget::Property` post-evaluation logic inside `Statement::Assignment`.
   Both variants evaluate the object then run identical write logic.
   Location in yielding: ~2253. In unyielding: ~2123.

6. `fn apply_index_write(obj: &Value, idx: &Value, val: Value) -> Result<(), RuntimeError>`
   Extracts the `AssignTarget::Index` post-evaluation logic (list bounds check, object string
   key write, error for other targets). Location in yielding: ~2265. In unyielding: ~2135.

### Issues to Watch Out For

- All six helpers must be purely synchronous — they cannot call back into either evaluator.
  Each receives already-evaluated `Value`s and returns a `Result`. This is satisfied by all
  six candidates.
- The `Statement::Throw` arm also looks identical between variants but requires
  `&Rc<RefCell<IshVm>>` to mutate the ledger. This is a viable follow-up extraction but is
  more complex and is excluded from this pass.
- Borrow lifetimes: helpers that accept `Value` by value are safe. Helpers that accept
  `&Value` must not re-enter `vm.borrow_mut()` during their execution.
- Concurrency: these helpers are sync and called only from within the existing async/sync
  interpreter context. No concurrency impact.

### Testability

No new tests are needed. The existing unit tests in `interpreter.rs` and the acceptance test
suite (`proto/ish-tests/`) cover all code paths through these arms. Correctness is verified
by `cargo test --workspace` before and after each extraction.

### Proposed Implementation

All changes in `proto/ish-vm/src/interpreter.rs`. No other files change.

1. Add the six `fn` helpers as private free functions below the `impl IshVm` block (before the
   `#[cfg(test)]` section):
   ```rust
   fn eval_literal(lit: &Literal) -> Value { ... }
   fn eval_unary_op(op: &UnaryOperator, val: Value) -> Result<Value, RuntimeError> { ... }
   fn apply_property_read(obj: Value, property: &str) -> Result<Value, RuntimeError> { ... }
   fn apply_index_read(obj: Value, idx: Value) -> Result<Value, RuntimeError> { ... }
   fn apply_property_write(obj: &Value, property: &str, val: Value) -> Result<(), RuntimeError> { ... }
   fn apply_index_write(obj: &Value, idx: &Value, val: Value) -> Result<(), RuntimeError> { ... }
   ```
2. In `eval_expression_yielding`, replace `Expression::Literal`, `Expression::UnaryOp` (inner
   match), `Expression::PropertyAccess` (post-eval), and `Expression::IndexAccess` (post-eval)
   bodies with calls to the corresponding helpers.
3. In `eval_expression_unyielding`, same replacements.
4. In `exec_statement_yielding`, replace `AssignTarget::Property` and `AssignTarget::Index`
   bodies with calls to `apply_property_write` and `apply_index_write`.
5. In `exec_statement_unyielding`, same replacements.
6. Run `cargo test --workspace` to confirm no regressions.

---

## Feature H2: Fix Non-Grammar-Structural Panics in ast_builder.rs

### Background

`ast_builder.rs` contains 102 `.unwrap()` calls in production code. The majority (~95) are
`inner.next().unwrap()` on pest `Pairs` iterators — grammar-structural panics where the rule
match guarantees the child token exists. These are acceptable for prototype stage.

Four sites represent genuine runtime risks not guaranteed by the grammar:

| Line | Code | Risk |
|------|------|------|
| ~1564 | `inner.as_str().parse::<i64>().unwrap()` | Overflow panic if user writes a too-large integer literal |
| ~1568 | `inner.as_str().parse::<f64>().unwrap()` | Overflow/NaN panic for malformed float literal |
| ~1445 | `lines.last().unwrap()` | Panic if `lines` is unexpectedly empty |
| ~120 | `value: value.unwrap()` in `build_var_decl` | Panic if grammar changes to make value optional |

### Issues to Watch Out For

- The numeric parse sites (lines 1564, 1568) are inside a closure passed to a `match` arm.
  The surrounding function signature (`build_match_pattern`) already returns `Result<_, ParseError>`,
  so `?` propagation works directly.
- `lines.last()` is called on a `Vec` built in the same function from non-empty input
  (`strip_triple_quote_indentation`). The `.unwrap()` is safe in the current implementation
  but `.unwrap_or(&"".to_string())` expresses the intent more clearly and survives future
  refactoring.
- `value.unwrap()` (line ~120, `build_var_decl`): `value` is populated by a `for` loop over
  inner pairs. The grammar guarantees a value in a `VariableDecl`, but adding `?` propagation
  converts a potential panic into a structured `ParseError`.
- Concurrency: parser is single-threaded and stateless. No concurrency impact.

### Testability

The integer overflow case can be tested with a new acceptance test:
```
assert_fail 'let x = 99999999999999999999' "out of range"
```
The existing parser tests cover the normal paths and confirm no regressions.

### Proposed Implementation

All changes in `proto/ish-parser/src/ast_builder.rs`. No other files change.

1. **Line ~1564 (integer parse):**
   ```rust
   // Before:
   let n: i64 = inner.as_str().parse().unwrap();
   // After:
   let span = inner.as_span();
   let n: i64 = inner.as_str().parse().map_err(|_| ParseError {
       message: format!("integer literal '{}' overflows i64", inner.as_str()),
       span: Some(span),
   })?;
   ```

2. **Line ~1568 (float parse):**
   Same pattern for `f64`.

3. **Line ~1445 (`lines.last()`):**
   ```rust
   // Before:
   let last_line = lines.last().unwrap();
   // After:
   let empty = String::new();
   let last_line = lines.last().unwrap_or(&empty);
   ```

4. **Line ~120 (`value.unwrap()` in `build_var_decl`):**
   Change the loop to propagate a `ParseError` if no value is found:
   ```rust
   value.ok_or_else(|| ParseError {
       message: "variable declaration missing value".to_string(),
       span: None,
   })?
   ```
   The function signature already returns `Result<Statement, ParseError>`, so `?` works.

5. Add one acceptance test for integer overflow (see Testability above).

6. Run `cargo test --workspace`.

---

## Feature M1: Reduce `register_ast_builtins` Boilerplate in reflection.rs

### Background

`register_ast_builtins` (380 lines) registers ~20 AST constructor builtins by repeating an
identical pattern: arity check → build `HashMap` with `"kind"` plus named fields → return
`Value::Object`. The only variation per builtin is the name, expected arity, and field names.

### `simple_ast_builtin` Helper

```rust
fn simple_ast_builtin(
    name: &'static str,
    arity: usize,
    fields: &'static [&'static str],
) -> Value {
    use crate::value::new_compiled_function;
    new_compiled_function(name, vec![], vec![], None, move |args| {
        if args.len() != arity {
            return Err(RuntimeError::system_error(
                format!("{} expects {} argument(s)", name, arity),
                ErrorCode::TypeMismatch,
            ));
        }
        let mut map = HashMap::new();
        // Derive "kind" from the builtin name by stripping the "ast_" prefix.
        map.insert("kind".to_string(), str_val(
            name.strip_prefix("ast_").unwrap_or(name)
        ));
        for (field, arg) in fields.iter().zip(args.iter()) {
            map.insert(field.to_string(), arg.clone());
        }
        Ok(Value::Object(Gc::new(GcCell::new(map))))
    }, Some(false))
}
```

The naming-convention dependency (`ast_<kind>`) is accepted. All current builtins follow this
convention.

### Issues to Watch Out For

- `ast_literal` has custom body logic (inspects `args[0]` to set `literal_type`). Keep it as
  an inline closure; do not attempt to pass it through `simple_ast_builtin`.
- The `.unwrap_or(name)` fallback in `strip_prefix` is harmless — no current builtin lacks
  the `ast_` prefix.
- Concurrency: `register_ast_builtins` is called once during VM setup on the main thread.
  No concurrency impact.

### Testability

The existing self-hosting analyzer tests exercise the AST builtins at runtime. Running
`cargo test --workspace` after the change is sufficient.

### Proposed Implementation

All changes in `proto/ish-vm/src/reflection.rs`. No other files change.

1. Add `fn simple_ast_builtin(...)` before `register_ast_builtins`.
2. Identify the structurally simple builtins (all except `ast_literal` and any others with
   non-trivial bodies). Replace each `env.define(...)` block with:
   ```rust
   env.define("ast_foo".into(), simple_ast_builtin("ast_foo", N, &["field1", "field2"]));
   ```
3. Leave `ast_literal` as its current inline closure.
4. Run `cargo test --workspace`.

Estimated reduction: ~380 lines → ~120 lines.

---

## Feature M2: Reduce Arity-Check Boilerplate in builtins.rs

### Background

Every builtin function body repeats the same three-line arity check. `new_compiled_function`
is always called with `vec![], vec![], None` for params/param_types/return_type. Two small
helpers eliminate both repetitions.

### Helpers

```rust
/// Check that `args` has exactly `n` elements; return a structured error otherwise.
fn arity(name: &str, args: &[Value], n: usize) -> Result<(), RuntimeError> {
    if args.len() != n {
        return Err(RuntimeError::system_error(
            format!("{} expects {} argument(s), got {}", name, n, args.len()),
            ErrorCode::ArgumentCountMismatch,
        ));
    }
    Ok(())
}

/// Create a builtin function value with no typed parameters (the common case).
fn new_builtin(
    name: &'static str,
    f: impl Fn(&[Value]) -> Result<Value, RuntimeError> + 'static,
) -> Value {
    new_compiled_function(name, vec![], vec![], None, f, Some(false))
}
```

### Issues to Watch Out For

- `new_builtin` hard-codes `has_yielding_entry: Some(false)`. No current builtin requires
  `Some(true)` or `None`. If one is added later, use `new_compiled_function` directly.
- The existing error message format varies slightly between builtins (some say "expects 2
  arguments", some say "expects 2 arguments (string, int)"). The new `arity` helper uses a
  standard format. Any type-hint information currently in the arity error messages will be
  lost. This is acceptable for prototype stage.
- Concurrency: builtin registration is single-threaded setup. No concurrency impact.

### Testability

The existing builtin tests and acceptance tests cover all builtins. `cargo test --workspace`
is sufficient verification.

### Proposed Implementation

All changes in `proto/ish-vm/src/builtins.rs`. No other files change.

1. Add `fn arity(...)` and `fn new_builtin(...)` at the top of the file, after the imports.
2. Convert all builtins in `register_strings`, `register_lists`, `register_objects`,
   `register_conversion`, and `register_io` to use:
   - `arity("fn_name", args, N)?` at the top of each closure body.
   - `new_builtin("fn_name", |args| { ... })` for the `env.define(...)` call.
3. Run `cargo test --workspace`.

---

## Feature M3: Extract Brace-Scanning Helper in `interpolate_shell_quoted`

### Background

`interpolate_shell_quoted` (63 lines) handles `$var`, `${var}`, and `{var}` substitution by
manually scanning a `Vec<char>`. The `${var}` and `{var}` branches each contain an inline
loop to find `}` and extract the variable name — the logic is nearly identical and can be
unified. The `{var}` branch also contains `.unwrap()` on `name.chars().next()` that is
guarded by `!name.is_empty()` but is still a panic site.

### Helper

```rust
/// Scan forward from `start` in `chars` to find `}`. Returns the variable name
/// and the index of the character after `}`, or `None` if `}` is not found.
fn scan_to_close_brace(chars: &[char], start: usize) -> Option<(String, usize)> {
    let mut j = start;
    while j < chars.len() && chars[j] != '}' {
        j += 1;
    }
    if j < chars.len() {
        Some((chars[start..j].iter().collect(), j + 1))
    } else {
        None
    }
}
```

The `{var}` branch name-validation check also uses `.unwrap()`:
```rust
// Before:
name.chars().next().unwrap().is_ascii_alphabetic() || name.starts_with('_')
// After:
name.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
```

### Issues to Watch Out For

- The `${var}` branch starts scanning at `i + 2` (after `$` and `{`); the `{var}` branch
  at `i + 1` (after `{`). `scan_to_close_brace` is called with the correct start offset
  at each call site.
- Concurrency: `interpolate_shell_quoted` is a pure function, called synchronously from the
  interpreter. No concurrency impact.

### Testability

The existing `interpolate_shell_quoted` tests in `interpreter.rs` and acceptance tests
for shell variable substitution cover the affected code paths. `cargo test --workspace`
confirms no regressions.

### Proposed Implementation

All changes in `proto/ish-vm/src/interpreter.rs`. No other files change.

1. Add `fn scan_to_close_brace(chars: &[char], start: usize) -> Option<(String, usize)>`
   as a free function near `interpolate_shell_quoted`.
2. Replace the inline `${var}` brace-scanning loop with `scan_to_close_brace(chars, i + 2)`.
3. Replace the inline `{var}` brace-scanning loop with `scan_to_close_brace(chars, i + 1)`.
4. Replace `name.chars().next().unwrap().is_ascii_alphabetic() || name.starts_with('_')`
   with `name.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')`.
5. Run `cargo test --workspace`.

---

## Feature L1: Exit-Code Sentinel Constant

### Proposed Implementation

In `proto/ish-vm/src/interpreter.rs`, near the top of the module (after the imports):

```rust
/// Sentinel exit code used when a process terminates without a numeric code
/// (e.g., killed by signal).
const MISSING_EXIT_CODE: i64 = -1;
```

Replace both `unwrap_or(-1)` calls (lines ~1794 and ~1803 in `run_command_pipeline`) with
`unwrap_or(MISSING_EXIT_CODE)`.

No new tests. No other file changes.

---

## Feature L2: Large Display Impls in display.rs — Deferred

The two `fmt` implementations in `proto/ish-ast/src/display.rs` (233 and 107 lines) are
structurally flat. No action in this pass. Revisit if the file grows beyond ~500 lines or
if adding a new AST node type becomes difficult to locate.

---

## Sequencing

Implement in this order. Each step is independently verifiable with `cargo test --workspace`.

| Step | Feature | Files changed |
|------|---------|--------------|
| 1 | L1 — exit-code constant | `interpreter.rs` |
| 2 | M3 — `scan_to_close_brace` | `interpreter.rs` |
| 3 | M2 — `arity` + `new_builtin` | `builtins.rs` |
| 4 | M1 — `simple_ast_builtin` | `reflection.rs` |
| 5 | H2 — non-structural unwrap fixes | `ast_builder.rs` |
| 6 | H1 — six interpreter helpers | `interpreter.rs` |

Steps 1–2 and steps 3–4 can be done in the same commit if desired, but H1 and H2 should
each be separate commits to allow isolated bisection if a test fails.

---

## Documentation Updates

These changes are pure internal refactoring with no language-visible behaviour changes.

- **`docs/architecture/vm.md`** — if updated, add a brief note that `eval_literal`,
  `eval_unary_op`, `apply_property_read`, `apply_index_read`, `apply_property_write`, and
  `apply_index_write` are shared sync helpers called by both execution paths.
- **`docs/project/history/2026-04-06-prototype-refactoring/`** — already created.
- All spec, user-guide, and AI-guide docs are unaffected.

---

## History Updates

- [x] Create `docs/project/history/2026-04-06-prototype-refactoring/` directory
- [x] Add `summary.md` with narrative prose
- [ ] Update `docs/project/history/INDEX.md`

---

## Referenced by

- [docs/project/rfp/refactoring-prototype.md](../rfp/refactoring-prototype.md)
- [docs/project/proposals/INDEX.md](INDEX.md)
