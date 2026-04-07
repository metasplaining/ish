---
title: "Plan Phase 8: Acceptance Tests"
category: plan
audience: [ai-dev]
status: completed
last-verified: 2026-04-06
depends-on: [docs/project/plans/module-system-core-a2/overview.md, docs/project/plans/module-system-core-a2/phase-5.md, docs/project/plans/module-system-core-a2/phase-6.md]
---

# Phase 8: Acceptance Tests

*Part of: [module-system-core-a2/overview.md](overview.md)*

Create the `modules/` acceptance test group. All tests exercise the binary via the shell (`ish -c` for inline tests; temp project directories for file-based tests). Depends on Phases 5 and 6 being complete.

## Context Files

None — acceptance tests are written against the acceptance test requirements in the proposal (see [module-system-core-a2.md](../../../proposals/module-system-core-a2.md) §Acceptance Tests for the full list of 50+ required scenarios).

## Requirements

- `proto/ish-tests/modules/run_group.sh` exists (same pattern as other group runners).
- Test files exist covering the six categories below.
- `cd proto && bash ish-tests/run_all.sh` passes all tests in the modules group.
- No existing tests regress.

## Tasks

- [x] 1. Create `proto/ish-tests/modules/run_group.sh` (copy pattern from `proto/ish-tests/basics/run_group.sh`):

  ```bash
  #!/usr/bin/env bash
  SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
  FAILURES=0
  for test_file in "$SCRIPT_DIR"/*.sh; do
      [[ "$(basename "$test_file")" == "run_group.sh" ]] && continue
      echo ""
      if bash "$test_file"; then :; else FAILURES=$((FAILURES + 1)); fi
  done
  [[ $FAILURES -gt 0 ]] && exit 1 || exit 0
  ```

- [x] 2. Create `proto/ish-tests/modules/declare_blocks.sh` — tests for `declare { }` blocks and mutual recursion:

  Use `source "$SCRIPT_DIR/../lib/test_lib.sh"` at top.

  Tests (all runnable via `run_ish` — no filesystem needed):
  - `mutual_recursion_in_declare_block`: `declare { fn even(n) { if n == 0 { true } else { odd(n-1) } }; fn odd(n) { if n == 0 { false } else { even(n-1) } } }; println(even(4))` → `true`
  - `declare_block_command_error`: `declare { let x = 1 }` → output contains `E020`
  - `declare_block_functions_visible_after`: `declare { fn greet() { "hello" } }; println(greet())` → `hello`
  - `two_fns_no_explicit_declare_mutual_recursion`: define two mutually recursive functions at top level (no `declare`) — expect them to work because `declare { }` is just a grouping hint; top-level forward refs may need `declare`.

  Note: Top-level mutual recursion without `declare { }` is handled by the implicit wrapping rule only for modules loaded via `use`. In the REPL/inline context, `declare { }` is needed for mutual recursion. Keep tests realistic.

- [x] 3. Create `proto/ish-tests/modules/module_identity.sh` — tests requiring temp files on disk:

  Helper function pattern for file-based tests:

  ```bash
  make_project() {
      local dir
      dir=$(mktemp -d)
      echo '{}' > "$dir/project.json"
      mkdir -p "$dir/src"
      echo "$dir"
  }
  ```

  Tests:
  - `file_with_top_level_command_runs_directly`: write a `.ish` file with `println("ok")`, run `$ISH $file`, expect exit 0 and output `ok`.
  - `file_with_top_level_command_not_importable`: create a project, write `src/cmd.ish` with `println("oops")`, import via `use cmd` from another file in project, expect E018.
  - `file_with_only_fns_importable`: write `src/pure.ish` with `pub fn f() { 42 }`, import via `use pure { f }`, call `f()`, expect `42`.
  - `cross_module_cycle`: create `src/a.ish` that does `use b` and `src/b.ish` that does `use a`, run a file that does `use a`, expect E017 with both file names in error.
  - `use_path_no_ish_extension`: run `use my-tool` where `my-tool` has no `.ish` file, expect E016.

- [x] 4. Create `proto/ish-tests/modules/module_mapping.sh` — tests for file-to-module mapping:

  Tests (all require temp project directories):
  - `use_net_http_resolves_to_file`: create `src/net/http.ish` with `pub fn get() { "ok" }`, do `use net/http { get }; println(get())`, expect `ok`.
  - `use_net_resolves_to_index`: create `src/net/index.ish` with `pub fn connect() { "connected" }`, do `use net { connect }; println(connect())`, expect `connected`.
  - `index_ish_defines_parent_not_self`: same setup as above; verify module is named `net` not `net/index` (test by checking qualified access works as `net.connect`).
  - `path_conflict_error`: create both `src/foo.ish` and `src/foo/index.ish`, do `use foo`, expect E019 naming both paths.
  - `not_found_error`: do `use nonexistent`, expect E016.

- [x] 5. Create `proto/ish-tests/modules/import_syntax.sh` — tests for all four import forms:

  Tests (require a project with a module `src/utils.ish` containing `pub fn greet(name) { str_concat("hello ", name) }` and `pub fn farewell(name) { str_concat("bye ", name) }`):
  - `plain_use`: `use utils; println(utils.greet("world"))` → `hello world`
  - `alias_use`: `use utils as u; println(u.greet("world"))` → `hello world`
  - `selective_use`: `use utils { greet }; println(greet("world"))` → `hello world`
  - `selective_alias_use`: `use utils { greet as hi }; println(hi("world"))` → `hello world`
  - `qualified_access_without_use`: access `utils.greet("world")` without `use` statement — expect resolution to work if `src/utils.ish` exists (qualified access does not require explicit `use`).

- [x] 6. Create `proto/ish-tests/modules/visibility.sh` — tests for visibility enforcement:

  Tests:
  - `pkg_item_accessible_same_project`: `src/a.ish` defines `fn internal() { 99 }` (no visibility keyword = pkg), another file does `use a { internal }; println(internal())` → `99`.
  - `priv_item_inaccessible_other_module`: `src/a.ish` defines `priv fn secret() { 1 }`, importing via `use a { secret }` → access error.
  - `pub_item_accessible_any_context`: `src/a.ish` defines `pub fn exported() { 42 }`, importable from outside project → `42`.
  - `inline_cannot_access_pkg`: running `ish -c 'use mymod { internal }'` from within a project dir (inline mode) → access error for `pkg` items (inline = no file path = not a project member).

- [x] 7. Create `proto/ish-tests/modules/bootstrap.sh` — tests for the `bootstrap` directive:

  Tests:
  - `bootstrap_outside_project_ok`: write a standalone script (not under any `project.json`) containing `bootstrap 'file:///tmp/dummy.json'`, run it, expect no E021 error (config parsing deferred, so it should be a no-op).
  - `bootstrap_inside_project_error`: write a file under a directory with `project.json` that contains `bootstrap 'file:///tmp/dummy.json'`, run it, expect E021.

- [x] 8. Create `proto/ish-tests/modules/interface_freeze.sh` — tests for `ish interface freeze`:

  Tests:
  - `freeze_generates_ishi`: set up project with `src/utils.ish` containing `pub fn greet() -> String { "hi" }`. Run `$ISH interface freeze`. Assert `src/utils.ishi` is created. Assert it contains `pub fn greet`.
  - `freeze_with_target`: run `$ISH interface freeze utils`. Assert `src/utils.ishi` is created.
  - `freeze_overwrites`: create a stale `src/utils.ishi` with wrong content. Run `$ISH interface freeze`. Assert it is overwritten with current content.

## Verification

Run: `cd proto && cargo build -p ish-shell --quiet && bash ish-tests/run_all.sh 2>&1 | tail -20`
Check: "All groups passed." in output and "modules" group appears with no failures.

Run: `cd proto && bash ish-tests/run_all.sh 2>&1 | grep -E "FAIL|pass"`
Check: No "FAIL" lines.

Invoke: `/verify module-system-core-a2/phase-8.md`
