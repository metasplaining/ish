---
title: Error Catalog
category: project
audience: [all]
status: draft
last-verified: 2026-03-19
depends-on: [docs/INDEX.md, docs/spec/errors.md]
---

# Error Catalog

Every error the ish language processor can produce, with explanations and remediation.

Errors use the entry-based error model defined in [errors.md](../spec/errors.md). Each error is an object carrying an `@Error` entry (the only predefined error entry type). Error classifications like `CodedError`, `TypeError`, and `SystemError` are ordinary ish types recognized structurally by the `code` property value.

> **Maintenance note for agents:** When adding a new error condition to the interpreter or builtins, assign the next available error code and add a row to this catalog. Update both this file and the error codes table in `docs/spec/errors.md`.

---

## Error Codes

Domain types are structural ish types (not entry types) defined in [errors.md](../spec/errors.md). Classification is determined by the `code` property value.

| Code | Structural Type | Summary | Production Sites |
|------|----------------|---------|-----------------|
| E001 | `Error` | Unhandled throw — a thrown value escaped all try/catch blocks | `interpreter.rs` (top-level run) |
| E002 | `CodedError` | Division by zero (includes modulo by zero) | `interpreter.rs` (eval_binary_op) |
| E003 | `ArgumentError` | Argument count mismatch — function called with wrong number of arguments | `interpreter.rs` (call_function), `builtins.rs` (all builtins) |
| E004 | `TypeError` | Type mismatch — operation applied to incompatible types; includes cannot-add, cannot-compare, cannot-index, cannot-iterate, missing annotation | `interpreter.rs` (binary ops, property access, indexing, type audit), `builtins.rs` (type checks) |
| E005 | `CodedError` | Undefined variable — referenced name not found in scope | `environment.rs` (get, set) |
| E006 | `TypeError` | Not callable — attempted to call a non-function value | `interpreter.rs` (call_function) |
| E007 | `CodedError` | Index out of bounds — list or string index outside valid range | `interpreter.rs` (list/string indexing) |
| E008 | `FileError` | File I/O error — file read or write failed | `builtins.rs` (read_file, write_file) |
| E009 | `TypeError` | Null unwrap — attempted to unwrap null with `?` operator | `interpreter.rs` (unwrap expression) |
| E010 | `CodedError` | Shell command error — external command execution failed | `interpreter.rs` (shell command execution) |
| E011 | `ConcurrencyError` | Concurrency error — cancelled task, panicked task, assurance discrepancy, already-awaited future | `interpreter.rs` (await, spawn, audits) |
| E012 | `TypeError` | Await type mismatch — `await` applied to a call to an explicitly unyielding function | `interpreter.rs` (Expression::Await) |
| E013 | `ConcurrencyError` | Spawn type mismatch — `spawn` applied to a call to an explicitly unyielding function | `interpreter.rs` (Expression::Spawn) |

---

## Referenced by

- [docs/INDEX.md](../INDEX.md)
- [docs/spec/errors.md](../spec/errors.md)
