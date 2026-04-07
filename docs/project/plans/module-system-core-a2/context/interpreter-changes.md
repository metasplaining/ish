*Extracted verbatim from [module-system-core-a2.md](../../../proposals/module-system-core-a2.md) §`interpreter.rs` changes.*

---

**`interpreter.rs` changes**

Replace the `Statement::Use { .. } => Ok(ControlFlow::None)` stub with a real implementation that delegates to the new modules:

1. Determine if the module path is external (contains a `.` in the first segment) or internal.
2. For internal paths: call `module_loader::resolve_module_path` against the caller's `src_root`.
3. Check for cycles against the current loading stack. If a cycle is found, return `RuntimeError` with code E017 (`module/cycle`), listing the full cycle path.
4. Load and parse the file.
5. Wrap its contents in an implicit `DeclareBlock`. If any statement in the file is not a declaration, return `RuntimeError` with code E018 (`module/script-not-importable`), naming the file.
6. Call `interface_checker::check_interface` on the file. Surface any interface errors before proceeding.
7. Evaluate the `DeclareBlock` in a fresh child environment.
8. Bind the module namespace into the caller's environment according to the import form (qualified, aliased, or selective).
9. On selective imports, call `access_control::check_access` for each imported symbol.

Replace the `Statement::ModDecl { .. }` stub with an internal error: `ModDecl` is no longer a valid statement; the parser should never produce it. If encountered, return an internal error.

Add `Statement::DeclareBlock` evaluation:

1. Collect all declarations in the block into a temporary scope.
2. Evaluate them with mutual forward-reference resolution (all function and type names are pre-registered before any body is evaluated).
3. Merge the resulting bindings into the parent environment.
4. If any statement in the block is not a declaration, return a compile error with code E020 (`module/declare-block-command`).

Add `Statement::Bootstrap` evaluation (D20 — partially deferred):

1. Check that the caller file is not under any `project.json` in its hierarchy (using `module_loader::find_project_root`). If it is, return E021 (`module/bootstrap-in-project`).
2. Config parsing, application, and URL fetching are deferred to a future revision. `ISH_PROXY` specification is deferred.

---

**Stub locations in current code** (interpreter.rs, both exec paths):

Yielding path (exec_statement_yielding):
- Line 711: `Statement::Use { .. } => Ok(ControlFlow::None)` — comment: "Module imports are resolved at load time, not runtime"
- Line 716: `Statement::DeclareBlock { .. } => Ok(ControlFlow::None)` — comment: "Execution deferred to A-2"
- Line 721: `Statement::Bootstrap { .. } => Ok(ControlFlow::None)` — comment: "Execution deferred to A-2"

Unyielding path (exec_statement_unyielding):
- Line 2212: `Statement::Use { .. } => Ok(ControlFlow::None)`
- Line 2213: `Statement::DeclareBlock { .. } => Ok(ControlFlow::None)`
- Line 2214: `Statement::Bootstrap { .. } => Ok(ControlFlow::None)`

Note: There is no `Statement::ModDecl` arm in either path — A-1 already removed it from the AST.

**What constitutes a "declaration" for DeclareBlock/implicit-declare purposes:**

Declarations are: `Statement::FunctionDecl`, `Statement::TypeAlias`. All other statement kinds are non-declarations. The implicit-declare-wrap check (step 5 in Use evaluation) must verify every top-level statement in the loaded file is one of these two kinds. DeclareBlock evaluation (step 4) must verify every statement in its body is one of these two kinds.
