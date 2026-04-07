# Phase 1: L1 — Exit-Code Sentinel Constant

*Part of: [refactoring-prototype/overview.md](overview.md)*

## Context Files
- [context/helper-signatures.md](context/helper-signatures.md) — `MISSING_EXIT_CODE` definition

## Requirements
- `MISSING_EXIT_CODE: i64 = -1` constant exists near the top of `interpreter.rs`.
- Both `unwrap_or(-1)` calls in `run_command_pipeline` use `unwrap_or(MISSING_EXIT_CODE)`.
- No other changes.

## Tasks

- [x] 1. In `proto/ish-vm/src/interpreter.rs`, after the `use` imports and before the first `enum` or `struct`, add:
  ```rust
  /// Sentinel exit code used when a process terminates without a numeric code
  /// (e.g., killed by signal).
  const MISSING_EXIT_CODE: i64 = -1;
  ```

- [x] 2. In `run_command_pipeline` (~line 1794), replace `unwrap_or(-1)` with `unwrap_or(MISSING_EXIT_CODE)` (first occurrence: `output.status.code().unwrap_or(-1) as i64`).

- [x] 3. In `run_command_pipeline` (~line 1803), replace the second `unwrap_or(-1)` with `unwrap_or(MISSING_EXIT_CODE)` (`status.code().unwrap_or(-1) as i64`).

## Verification

Run: `cd proto && cargo test --workspace`
Check: all tests pass; no new warnings about unused constants.
Invoke: `/verify refactoring-prototype/phase-1.md`
