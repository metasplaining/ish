# Phase 5: ish-vm — Remove ModDecl Arms

*Part of: [module-system-core-a1/overview.md](overview.md)*

## Context Files

- [context/vm-moddecl-arms.md](context/vm-moddecl-arms.md) — all ModDecl arms in ish-vm files

## Requirements

- No `ModDecl` match arms exist in any ish-vm source file.
- No `Visibility::Public`, `Visibility::Private`, or `Visibility::PubScope` references exist.
- Every exhaustive match on `Statement` in ish-vm compiles without non-exhaustive warnings.
- `DeclareBlock` and `Bootstrap` have no-op / placeholder arms in the interpreter (A-2 will fill them in).

## Tasks

### proto/ish-vm/src/analyzer.rs

- [x] 1. Remove `Statement::ModDecl` from the terminal arm — `proto/ish-vm/src/analyzer.rs`

  Find the arm containing `| Statement::ModDecl { .. }` and remove that pattern alternative. Add `| Statement::DeclareBlock { .. }` and `| Statement::Bootstrap { .. }` to the same terminal arm.

  Context: The analyzer uses a terminal arm that catches all non-recursive statement variants and returns `false` (not yielding). `DeclareBlock` bodies are not recursed into by the analyzer in A-1 (A-2 adds the mutual-recursion yielding logic). Adding it to the terminal arm is correct for now.

### proto/ish-vm/src/interpreter.rs

- [x] 2. Remove `Statement::ModDecl` stub arm (line ~716) — `proto/ish-vm/src/interpreter.rs`

  Delete the arm:
  ```rust
              Statement::ModDecl { .. } => {
                  // ... no-op or log
              }
  ```

  Add placeholder arms for the new variants in the same match:
  ```rust
              Statement::DeclareBlock { .. } => {
                  // Execution deferred to A-2
                  Ok(ControlFlow::None)
              }
              Statement::Bootstrap { .. } => {
                  // Execution deferred to A-2
                  Ok(ControlFlow::None)
              }
  ```

- [x] 3. Remove `Statement::ModDecl` no-op arm (line ~2208) — `proto/ish-vm/src/interpreter.rs`

  Delete the arm:
  ```rust
              Statement::ModDecl { .. } => Ok(ControlFlow::None),
  ```

  Add new no-op arms for the new variants:
  ```rust
              | Statement::DeclareBlock { .. }
              | Statement::Bootstrap { .. } => Ok(ControlFlow::None),
  ```

  Or as separate arms — match what the surrounding code uses.

### proto/ish-vm/src/reflection.rs

- [x] 4. Remove `Statement::ModDecl` arm — `proto/ish-vm/src/reflection.rs`

  Find the arm (line ~143):
  ```rust
          Statement::ModDecl { name, body, .. } => {
  ```

  Delete the entire arm. Add no-op arms for the new variants — return an appropriate placeholder `Value` (e.g., `Value::Null` or a string representation):
  ```rust
          Statement::DeclareBlock { body } => {
              // Reflection for DeclareBlock deferred to A-2
              Value::Null
          }
          Statement::Bootstrap { .. } => {
              Value::Null
          }
  ```

  Adjust the return type and surrounding code as needed to match reflection.rs conventions.

## Verification

Run: `cd proto && cargo build --workspace 2>&1`

Check: Full workspace build succeeds with zero errors and zero non-exhaustive-match warnings. No `ModDecl`, `Visibility::Public`, `Visibility::Private`, or `PubScope` references remain anywhere in the workspace.

Run: `cd proto && cargo test --workspace 2>&1`

Check: All 317 tests pass (or the current count if it differs after phase5.rs replacement in Phase 4).

Invoke: `/verify module-system-core-a1/phase-5.md`
