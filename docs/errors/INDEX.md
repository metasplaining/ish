---
title: Error Catalog
category: project
audience: [all]
status: draft
last-verified: 2026-04-06
depends-on: [docs/INDEX.md, docs/spec/errors.md]
---

# Error Catalog

Every error the ish language processor can produce, with explanations and remediation.

Errors use the entry-based error model defined in [errors.md](../spec/errors.md). Each error is an object carrying an `@Error` entry (the only predefined error entry type). Error classifications like `CodedError`, `TypeError`, and `SystemError` are ordinary ish types recognized structurally by the `code` property value.

The runtime crate (`ish-runtime`) defines an `ErrorCode` enum with a type-safe variant for each code below (e.g., `ErrorCode::DivisionByZero` for E002). `RuntimeError::system_error()` accepts `ErrorCode` instead of string literal codes.

> **Maintenance note for agents:** When adding a new error condition to the interpreter or builtins, assign the next available error code, add the variant to `ErrorCode` in `ish-runtime/src/error.rs`, and add a row to this catalog. Update both this file and the error codes table in `docs/spec/errors.md`.

---

## Error Codes

Domain types are structural ish types (not entry types) defined in [errors.md](../spec/errors.md). Classification is determined by the `code` property value. The `ErrorCode` enum is defined in `ish-runtime/src/error.rs`.

| Code | ErrorCode Variant | Structural Type | Summary | Production Sites |
|------|----------------|---------|-----------------|
| E001 | `UnhandledThrow` | `Error` | Unhandled throw — a thrown value escaped all try/catch blocks | `interpreter.rs` (top-level run) |
| E002 | `DivisionByZero` | `CodedError` | Division by zero (includes modulo by zero) | `interpreter.rs` (eval_binary_op) |
| E003 | `ArgumentCountMismatch` | `ArgumentError` | Argument count mismatch — function called with wrong number of arguments | `interpreter.rs` (call_function), `builtins.rs` (all builtins) |
| E004 | `TypeMismatch` | `TypeError` | Type mismatch — operation applied to incompatible types; includes cannot-add, cannot-compare, cannot-index, cannot-iterate, missing annotation | `interpreter.rs` (binary ops, property access, indexing, type audit), `builtins.rs` (type checks) |
| E005 | `UndefinedVariable` | `CodedError` | Undefined variable — referenced name not found in scope | `environment.rs` (get, set) |
| E006 | `NotCallable` | `TypeError` | Not callable — attempted to call a non-function value | `interpreter.rs` (call_function) |
| E007 | `IndexOutOfBounds` | `CodedError` | Index out of bounds — list or string index outside valid range | `interpreter.rs` (list/string indexing) |
| E008 | `IoError` | `FileError` | File I/O error — file read or write failed | `builtins.rs` (read_file, write_file) |
| E009 | `NullUnwrap` | `TypeError` | Null unwrap — attempted to unwrap null with `?` operator | `interpreter.rs` (unwrap expression) |
| E010 | `ShellError` | `CodedError` | Shell command error — external command execution failed | `interpreter.rs` (shell command execution) |
| E011 | `AsyncError` | `ConcurrencyError` | Concurrency error — cancelled task, panicked task, assurance discrepancy, already-awaited future | `interpreter.rs` (await, spawn, audits) |
| E012 | `AwaitUnyielding` | `TypeError` | Await type mismatch — `await` applied to a call to an explicitly unyielding function | `interpreter.rs` (Expression::Await) |
| E013 | `SpawnUnyielding` | `ConcurrencyError` | Spawn type mismatch — `spawn` applied to a call to an explicitly unyielding function | `interpreter.rs` (Expression::Spawn) |
| E014 | `AwaitNonFuture` | `TypeError` | Await type mismatch — `await` applied to a non-future value | `interpreter.rs` (Expression::Await, non-Future value) |
| E015 | `UnyieldingViolation` | `ConcurrencyError` | Unyielding annotation violation — function declared `@[unyielding]` contains yielding operations | `interpreter.rs` (Statement::FunctionDecl, @[unyielding] annotation check) |
| E016 | `ModuleNotFound` | `CodedError` | `use` path has no matching `.ish` file | `module_loader::resolve_module_path` |
| E017 | `ModuleCycle` | `CodedError` | Circular `use` dependency detected | `interpreter.rs` — Use evaluation |
| E018 | `ModuleScriptNotImportable` | `CodedError` | File imported via `use` contains top-level commands | `interpreter.rs` — Use evaluation |
| E019 | `ModulePathConflict` | `CodedError` | Both `foo.ish` and `foo/index.ish` exist | `module_loader::resolve_module_path` |
| E020 | `ModuleDeclareBlockCommand` | `CodedError` | `declare { }` block contains a non-declaration statement | `interpreter.rs` — DeclareBlock evaluation |
| E021 | `ModuleBootstrapInProject` | `CodedError` | `bootstrap` used inside a project hierarchy | `interpreter.rs` — Bootstrap evaluation |
| E022 | `InterfaceSymbolNotInImplementation` | `CodedError` | `.ishi` declares a symbol absent from the `.ish` file | `interface_checker.rs` |
| E023 | `InterfaceSymbolNotInInterface` | `CodedError` | `.ish` has a `pub` symbol not declared in `.ishi` | `interface_checker.rs` |
| E024 | `InterfaceSymbolMismatch` | `CodedError` | Symbol present in both `.ishi` and `.ish` with mismatched signatures | `interface_checker.rs` |

---

## Referenced by

- [docs/INDEX.md](../INDEX.md)
- [docs/spec/errors.md](../spec/errors.md)
