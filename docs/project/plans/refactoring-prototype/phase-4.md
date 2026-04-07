# Phase 4: M1 — Add `simple_ast_builtin` to reflection.rs

*Part of: [refactoring-prototype/overview.md](overview.md)*

## Context Files
- [context/helper-signatures.md](context/helper-signatures.md) — `simple_ast_builtin` definition
- [context/reflection-exceptions.md](context/reflection-exceptions.md) — which builtins must stay as inline closures, and the complete table of simple builtins with their fields

## Requirements
- `fn simple_ast_builtin(name: &'static str, arity: usize, fields: &'static [&'static str]) -> Value`
  exists in `reflection.rs` before `register_ast_builtins`.
- All 22 simple builtins listed in the table in `reflection-exceptions.md` are converted to
  `simple_ast_builtin` calls.
- The three exceptions (`ast_literal`, `ast_param`, `ast_assign_target_var`) remain as inline
  closures, unchanged.
- `register_ast_builtins` reduces from ~380 lines to ~120 lines.
- No behaviour change for any builtin.

## Tasks

- [x] 1. In `proto/ish-vm/src/reflection.rs`, add `fn simple_ast_builtin(...)` immediately
  before the `pub fn register_ast_builtins(...)` function. Use the exact body from
  [context/helper-signatures.md](context/helper-signatures.md) §M1.

- [x] 2. Working through `register_ast_builtins` top to bottom, convert each simple builtin
  using the table in [context/reflection-exceptions.md](context/reflection-exceptions.md).
  Replace each block like:
  ```rust
  env.define(
      "ast_foo".into(),
      new_compiled_function("ast_foo", vec![], vec![], None, |args| {
          if args.len() != N { return Err(...) }
          let mut map = HashMap::new();
          map.insert("kind".to_string(), str_val("foo"));
          map.insert("field1".to_string(), args[0].clone());
          // ...
          Ok(Value::Object(Gc::new(GcCell::new(map))))
      }, Some(false)),
  );
  ```
  With:
  ```rust
  env.define("ast_foo".into(), simple_ast_builtin("ast_foo", N, &["field1", ...]));
  ```

- [x] 3. Leave `ast_literal`, `ast_param`, and `ast_assign_target_var` exactly as they are.
  Do not modify them.

- [x] 4. Remove any `use crate::value::new_compiled_function;` import inside `register_ast_builtins`
  if `new_compiled_function` is no longer called directly there (it is now called only via
  `simple_ast_builtin`). If `new_compiled_function` is still used for the exceptions, leave
  the import.

## Verification

Run: `cd proto && cargo test --workspace`
Check: all tests pass. The `register_ast_builtins` function should now be visibly shorter.
Specifically, the self-hosting analyzer tests (in `proto/ish-stdlib/src/analyzer.rs`) exercise
most AST builtins — they must all pass.
Invoke: `/verify refactoring-prototype/phase-4.md`
