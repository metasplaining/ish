#!/usr/bin/env bash
# ---
# feature: Cross-boundary yielding function calls
# docs: docs/architecture/vm.md
# section: Shim-Only Function Dispatch
# ---
# Tests that apply() works correctly with yielding and unyielding functions.
# apply() is unyielding — calling it with an async function returns a Future
# (no implied await). The caller must explicitly await the result.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "--- Cross-Boundary Yielding Tests ---"

# apply(async_fn, args) returns a Future — no implied await (apply is unyielding)
output=$(run_ish 'async fn work(x) { return x + 1 }
                  println(type_of(apply(work, [10])))')
assert_output "apply async fn returns future" "future" "$output"

# The returned future can be awaited
output=$(run_ish 'async fn work(x) { return x + 1 }
                  let f = apply(work, [10])
                  println(await f)')
assert_output "apply async fn future resolves correctly" "11" "$output"

# apply(unyielding_fn, args) returns the result directly
output=$(run_ish 'fn add(a, b) { return a + b }
                  println(apply(add, [3, 4]))')
assert_output "apply unyielding fn direct result" "7" "$output"

# apply is itself unyielding
output=$(run_ish 'println(is_yielding(apply))')
assert_output "apply is unyielding" "false" "$output"

finish
