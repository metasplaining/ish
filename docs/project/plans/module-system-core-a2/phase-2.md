---
title: "Plan Phase 2: Error Codes"
category: plan
audience: [ai-dev]
status: completed
last-verified: 2026-04-06
depends-on: [docs/project/plans/module-system-core-a2/overview.md]
---

# Phase 2: Error Codes

*Part of: [module-system-core-a2/overview.md](overview.md)*

Add E016–E024 to the `ErrorCode` enum in `ish-runtime`. This must be done before the VM modules that use these codes (Phases 3–6).

## Context Files

- [context/error-codes.md](context/error-codes.md) — exact variant names, string values, and `as_str` arms

## Requirements

- `ErrorCode` enum in `proto/ish-runtime/src/error.rs` has exactly nine new variants: `ModuleNotFound` (E016) through `InterfaceSymbolMismatch` (E024).
- Each variant has a corresponding `as_str` arm returning `"E016"` through `"E024"`.
- The crate builds without errors after this change: `cd proto && cargo check -p ish-runtime`.

## Tasks

- [x] 1. Edit `proto/ish-runtime/src/error.rs` — add nine variants to the `ErrorCode` enum after `UnyieldingViolation`:

  ```rust
  ModuleNotFound,                     // E016
  ModuleCycle,                        // E017
  ModuleScriptNotImportable,          // E018
  ModulePathConflict,                 // E019
  ModuleDeclareBlockCommand,          // E020
  ModuleBootstrapInProject,           // E021
  InterfaceSymbolNotInImplementation, // E022
  InterfaceSymbolNotInInterface,      // E023
  InterfaceSymbolMismatch,            // E024
  ```

- [x] 2. Edit `proto/ish-runtime/src/error.rs` — add nine `as_str` arms to the `match self` block after the `UnyieldingViolation => "E015"` arm:

  ```rust
  ErrorCode::ModuleNotFound => "E016",
  ErrorCode::ModuleCycle => "E017",
  ErrorCode::ModuleScriptNotImportable => "E018",
  ErrorCode::ModulePathConflict => "E019",
  ErrorCode::ModuleDeclareBlockCommand => "E020",
  ErrorCode::ModuleBootstrapInProject => "E021",
  ErrorCode::InterfaceSymbolNotInImplementation => "E022",
  ErrorCode::InterfaceSymbolNotInInterface => "E023",
  ErrorCode::InterfaceSymbolMismatch => "E024",
  ```

## Verification

Run: `cd proto && cargo check -p ish-runtime`
Check: Compiles with no errors (warnings about unused variants are acceptable).

Run: `grep -c "E01[6-9]\|E02[0-4]" proto/ish-runtime/src/error.rs`
Check: Count is 18 or more (each code appears at least twice: variant + as_str).

Invoke: `/verify module-system-core-a2/phase-2.md`
