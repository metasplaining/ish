---
title: "Plan Phase 4: Interface Checker"
category: plan
audience: [ai-dev]
status: completed
last-verified: 2026-04-06
depends-on: [docs/project/plans/module-system-core-a2/overview.md, docs/project/plans/module-system-core-a2/phase-2.md]
---

# Phase 4: Interface Checker

*Part of: [module-system-core-a2/overview.md](overview.md)*

Create `interface_checker.rs` in `ish-vm`. This module is independent of `module_loader.rs` and `access_control.rs` and can be done in parallel with Phase 3.

## Context Files

- [context/interface-checker.md](context/interface-checker.md) — full function spec and error behavior
- [context/error-codes.md](context/error-codes.md) — error codes used here: E022, E023, E024

## Requirements

- `proto/ish-vm/src/interface_checker.rs` exists with a `check_interface` function.
- `check_interface` returns an empty vec when no `.ishi` file exists alongside the `.ish` file.
- `check_interface` emits `InterfaceSymbolNotInImplementation` (E022) for each symbol in `.ishi` absent from `pub_declarations`.
- `check_interface` emits `InterfaceSymbolNotInInterface` (E023) for each `pub` symbol in `pub_declarations` absent from the `.ishi` file.
- `check_interface` emits `InterfaceSymbolMismatch` (E024) for each symbol present in both with differing signatures.
- The module is exported from `proto/ish-vm/src/lib.rs`.
- `cd proto && cargo check -p ish-vm` passes.

## Tasks

- [x] 1. Create `proto/ish-vm/src/interface_checker.rs`:

  The function signature and semantics:

  ```rust
  use std::path::Path;
  use ish_ast::Statement;
  use crate::error::ErrorCode;

  #[derive(Debug)]
  pub struct InterfaceError {
      pub code: ErrorCode,
      pub symbol: String,
      pub message: String,
  }

  /// Check interface file consistency for a module.
  ///
  /// `module_file` is the path to the `.ish` file.
  /// `pub_declarations` is the list of top-level statements with Pub visibility
  ///   (FunctionDecl and TypeAlias are the relevant kinds).
  ///
  /// Returns an empty Vec if no sibling `.ishi` file exists.
  pub fn check_interface(
      module_file: &Path,
      pub_declarations: &[Statement],
  ) -> Vec<InterfaceError> {
      // 1. Compute sibling path: replace .ish extension with .ishi.
      // 2. If .ishi does not exist: return vec![].
      // 3. Read and parse the .ishi file using ish_parser.
      // 4. Extract pub symbols from the parsed .ishi (function names and type names).
      // 5. Extract pub symbol names from pub_declarations.
      // 6. For each symbol in .ishi not in pub_declarations: emit E022.
      // 7. For each pub symbol in pub_declarations not in .ishi: emit E023.
      // 8. For each symbol in both: compare signatures; if different, emit E024.
  }
  ```

  **Signature comparison for E024:** Two `FunctionDecl` nodes match if their `params` lists have equal length and each parameter's `type_annotation` matches. Two `TypeAlias` nodes match if their `definition` fields match. Use `PartialEq` on the AST node fields (already derived via `#[derive(PartialEq)]` on `ish_ast` types).

  **Parsing `.ishi` files:** Use `ish_parser::parse` directly. `.ishi` files are valid ish source containing only `pub fn` and `pub type` statements. Parse errors in the `.ishi` file produce a single `InterfaceError` with a descriptive message and code E022 (conservative: treat parse failure as "all symbols missing").

  **`ish_parser` dependency:** `ish-vm` already depends on `ish_ast`. Add `ish-parser` as a dev-dependency if not already present, or as a regular dependency if `interface_checker.rs` is in the non-test module. Check `proto/ish-vm/Cargo.toml` — if `ish-parser` is not listed, add it under `[dependencies]`.

- [x] 2. Edit `proto/ish-vm/src/lib.rs` — add:

  ```rust
  pub mod interface_checker;
  ```

- [x] 3. Check `proto/ish-vm/Cargo.toml` — if `ish-parser` is not in `[dependencies]`, add:

  ```toml
  ish-parser = { path = "../ish-parser" }
  ```

## Verification

Run: `cd proto && cargo check -p ish-vm`
Check: Compiles with no errors.

Run: `grep "pub fn check_interface" proto/ish-vm/src/interface_checker.rs`
Check: Line found.

Invoke: `/verify module-system-core-a2/phase-4.md`
