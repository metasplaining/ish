*Extracted verbatim from [module-system-core-a2.md](../../../proposals/module-system-core-a2.md) §Analyzer update for declare blocks (D22).*

---

**Analyzer update for declare blocks (D22)**

Update the existing code analyzer (in `ish-stdlib` or wherever yielding analysis runs) to handle mutually recursive functions declared together in a `declare { }` block:

1. When analyzing a `DeclareBlock`, first register all function names in the block as a mutual-recursion group before analyzing any bodies.
2. For each function in the group, determine yielding based on its own operations first.
3. If any function in the group calls another function in the group, propagate yielding transitively.
4. If a cycle is detected within the group and at least one function is yielding (by any criterion), mark all functions in the cycle as yielding.
5. If a cycle is detected and no function has any yielding criterion other than the cyclic call, mark all functions in the cycle as unyielding.

---

**Current state in `proto/ish-vm/src/analyzer.rs`:**

- `Statement::DeclareBlock { .. }` is in the `contains_yielding_node` non-yielding arm (line 234): treated as non-yielding without body traversal.
- `Statement::FunctionDecl` (line 226): treated as non-yielding (does not recurse into nested function bodies — they are classified independently when declared).
- The `classify_function` function (line 36) takes `body`, `is_async`, `env`, `param_names`, `fn_name` and walks the body looking for yielding operations.

**Where to add the DeclareBlock handler:**

The DeclareBlock analysis should run in `interpreter.rs` during `Statement::DeclareBlock` evaluation (before registering any functions), not in the standalone `classify_function`. The procedure:

1. Pre-scan the block: collect all `FunctionDecl` names and bodies.
2. Seed the analysis environment with all names as `Value::Null` (so cross-calls within the group don't error as "undefined").
3. Call `classify_function` for each body using this seeded environment.
4. Check if any function calls another function in the group — if so, propagate yielding per D22 rules.
5. Assign final `YieldingClassification` to each function and register them in the environment with the correct `has_yielding_entry`.

The `contains_yielding_node` arm for `Statement::DeclareBlock` in `analyzer.rs` can remain `Ok(false)` — the DeclareBlock itself does not make an enclosing function yielding; its content is classified separately.
