---
title: "Plan Phase 7: Unit Tests"
category: plan
audience: [ai-dev]
status: completed
last-verified: 2026-04-06
depends-on: [docs/project/plans/module-system-core-a2/overview.md, docs/project/plans/module-system-core-a2/phase-3.md, docs/project/plans/module-system-core-a2/phase-4.md]
---

# Phase 7: Unit Tests

*Part of: [module-system-core-a2/overview.md](overview.md)*

Add unit tests to `ish-vm` for the three new modules. Depends on Phases 3 and 4 being complete.

## Context Files

- [context/vm-new-modules.md](context/vm-new-modules.md) — function specifications to test against
- [context/interface-checker.md](context/interface-checker.md) — check_interface behavior to test

## Requirements

- `module_loader::derive_module_path`: tests for standard path, `index.ish` path, and path-conflict detection.
- `module_loader::resolve_module_path`: tests for found, not-found, and conflict cases.
- `access_control::check_access`: tests for all nine combinations of `(Priv/Pkg/Pub) × (same-module/same-project/external)`.
- `module_loader::find_project_root`: tests for found at current dir, found via walk, not found.
- `module_loader::check_cycle`: tests for cycle detected and not detected.
- `interface_checker::check_interface`: tests for symbol-not-in-implementation (E022), symbol-not-in-interface (E023), symbol-mismatch (E024), and no `.ishi` file present (empty result).
- All tests pass: `cd proto && cargo test -p ish-vm`.

## Tasks

- [x] 1. Add a `#[cfg(test)]` module to `proto/ish-vm/src/module_loader.rs` with the following tests (use `tempfile` crate or `std::env::temp_dir()` for filesystem tests):

  - `find_project_root_at_current_dir`: create a temp dir with `project.json` in it, call `find_project_root` from that dir, assert returns `Some(that dir)`.
  - `find_project_root_via_walk`: create `tmp/parent/project.json` and call `find_project_root` from `tmp/parent/child/`, assert returns `Some(tmp/parent/)`.
  - `find_project_root_not_found`: call `find_project_root` from `/tmp` (or a fresh temp dir without any `project.json` ancestor), assert returns `None`. Use a tmpdir deep in a known path without `project.json`.
  - `derive_module_path_standard`: `file_path = src/net/http.ish`, `src_root = src/`, expect `["net", "http"]`.
  - `derive_module_path_index`: `file_path = src/net/index.ish`, `src_root = src/`, expect `["net"]`.
  - `resolve_module_path_found_file`: create `src/net/http.ish` in a temp dir, resolve `["net", "http"]`, expect `Ok(that path)`.
  - `resolve_module_path_found_index`: create `src/net/index.ish` in a temp dir, resolve `["net"]`, expect `Ok(that path)`.
  - `resolve_module_path_not_found`: resolve `["nonexistent"]` against an empty temp src dir, expect `Err` with E016.
  - `resolve_module_path_conflict`: create both `src/foo.ish` and `src/foo/index.ish` in a temp dir, resolve `["foo"]`, expect `Err` with E019.
  - `check_cycle_detected`: call `check_cycle(&[vec!["net".into(), "http".into()]], &["net".into(), "http".into()])`, expect `true`.
  - `check_cycle_not_detected`: call `check_cycle(&[vec!["net".into()]], &["net".into(), "http".into()])`, expect `false`.

- [x] 2. Add a `#[cfg(test)]` module to `proto/ish-vm/src/access_control.rs` with tests for all nine combinations:

  Combinations: visibility (`Priv`, `Pkg`, `Pub`) × caller location (`same_file`, `same_project`, `external`):

  - `priv_same_file`: item at `/proj/src/a.ish`, caller at `/proj/src/a.ish` → `Ok(())`
  - `priv_same_project`: item at `/proj/src/a.ish`, caller at `/proj/src/b.ish` → `Err(AccessError::Private)`
  - `priv_external`: caller at `/other/src/b.ish` → `Err(AccessError::Private)`
  - `pkg_same_file`: item project_root `/proj`, caller at `/proj/src/b.ish` → `Ok(())`
  - `pkg_same_project`: caller at `/proj/scripts/run.sh` → `Ok(())`
  - `pkg_external`: caller at `/other/src/b.ish`, item project_root `/proj` → `Err(AccessError::PackageOnly)`
  - `pub_same_file` / `pub_same_project` / `pub_external`: all → `Ok(())`
  - `pkg_inline_caller`: `caller_file_path = None` → `Err(AccessError::PackageOnly)`

- [x] 3. Add a `#[cfg(test)]` module to `proto/ish-vm/src/interface_checker.rs` with tests:

  Use `tempfile`-style temp dirs or write small fixture files:

  - `no_ishi_file`: call `check_interface` with a path whose `.ishi` sibling does not exist, expect empty vec.
  - `symbol_not_in_implementation`: create a `.ishi` declaring `pub fn foo()` but pass empty `pub_declarations`, expect `InterfaceError` with code E022 and symbol `"foo"`.
  - `symbol_not_in_interface`: create a `.ishi` that is empty but pass a `pub_declarations` list containing a `FunctionDecl` named `bar`, expect `InterfaceError` with code E023.
  - `symbol_mismatch`: `.ishi` declares `pub fn foo(x: Int)`, `pub_declarations` contains `pub fn foo(x: String)`, expect E024.
  - `all_match`: `.ishi` declares `pub fn foo(x: Int)`, `pub_declarations` contains same, expect empty vec.

## Verification

Run: `cd proto && cargo test -p ish-vm 2>&1 | tail -20`
Check: All tests pass. Count includes new module_loader, access_control, and interface_checker tests.

Run: `cd proto && cargo test -p ish-vm module_loader 2>&1`
Check: All module_loader tests listed and passed.

Invoke: `/verify module-system-core-a2/phase-7.md`
