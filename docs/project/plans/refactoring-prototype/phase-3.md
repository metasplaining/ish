# Phase 3: M2 — Add `arity` and `new_builtin` Helpers to builtins.rs

*Part of: [refactoring-prototype/overview.md](overview.md)*

## Context Files
- [context/helper-signatures.md](context/helper-signatures.md) — `arity` and `new_builtin` definitions

## Requirements
- `fn arity(name: &str, args: &[Value], n: usize) -> Result<(), RuntimeError>` exists in `builtins.rs`.
- `fn new_builtin(name: &'static str, f: impl Fn(&[Value]) -> Result<Value, RuntimeError> + 'static) -> Value` exists in `builtins.rs`.
- All builtins in `register_strings`, `register_lists`, `register_objects`, `register_conversion`,
  and `register_io` use `arity(name, args, N)?` for arity checking.
- All `env.define(name, new_compiled_function(name, vec![], vec![], None, closure, Some(false)))` 
  calls are replaced with `env.define(name.into(), new_builtin(name, closure))`.
- No builtin behaviour changes. Arity error messages now use the standard format
  `"<name> expects <n> argument(s), got <actual>"`.

## Tasks

- [x] 1. In `proto/ish-vm/src/builtins.rs`, after the `use` imports and before `register_all`,
  add the `arity` and `new_builtin` functions exactly as specified in
  [context/helper-signatures.md](context/helper-signatures.md) §M2.

- [x] 2. Convert all builtins in `register_io` to use `arity(...)` and `new_builtin(...)`.
  For each `env.define(...)` block: replace the `new_compiled_function(name, vec![], vec![], None, |args| { ... }, Some(false))` wrapper with `new_builtin(name, |args| { ... })`, and replace the inline `if args.len() != N { return Err(...) }` block with `arity(name, args, N)?` at the top of the closure.

- [x] 3. Same conversion for all builtins in `register_strings`.

- [x] 4. Same conversion for all builtins in `register_lists`.

- [x] 5. Same conversion for all builtins in `register_objects`.

- [x] 6. Same conversion for all builtins in `register_conversion`.

- [x] 7. Same conversion for all builtins in `register_errors` (if it exists in this file).

## Verification

Run: `cd proto && cargo test --workspace`
Check: all tests pass. Spot-check that arity error messages now read "expects N argument(s), got M" rather than the old format.
Invoke: `/verify refactoring-prototype/phase-3.md`
