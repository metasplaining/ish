#!/usr/bin/env bash
# ---
# feature: Cross-boundary yielding function calls
# docs: docs/architecture/vm.md
# section: Shim-Only Function Dispatch
# ---
# Tests that apply() works correctly with yielding and unyielding functions.
# apply() is a compiled shim; calling it with an interpreted yielding function
# should return a Future; calling with an unyielding function returns directly.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "--- Cross-Boundary Yielding Tests ---"

# apply(async_fn, [args]) — implied await resolves the Future to the result
output=$(run_ish 'async fn work(x) { return x + 1 }; println(apply(work, [10]))')
assert_output "apply async fn via implied await" "11" "$output"

# apply(unyielding_fn, [args]) returns result directly
output=$(run_ish 'fn add(a, b) { return a + b }; println(apply(add, [3, 4]))')
assert_output "apply unyielding fn direct" "7" "$output"

# apply(async_fn) — implied await means result is resolved, not a future
output=$(run_ish 'async fn work(x) { return x }; println(type_of(apply(work, [10])))')
assert_output "apply async fn result type after implied await" "int" "$output"

finish
