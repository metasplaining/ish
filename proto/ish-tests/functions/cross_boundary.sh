#!/usr/bin/env bash
# ---
# feature: Cross-boundary function calls
# docs: docs/architecture/runtime.md
# section: Shim
# ---
# Tests that functions can be called across the compiled/interpreted boundary
# via the apply() builtin. apply() is a compiled shim, so calling it with an
# interpreted function creates the interpreted→compiled→interpreted path.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "--- Cross-Boundary Function Calls ---"

# Basic apply: interpreted function via compiled builtin
output=$(run_ish 'println(apply((x) => x + 1, [10]))')
assert_output "apply basic" "11" "$output"

# Double nesting: 4 boundary crossings
output=$(run_ish 'let f = (x) => x + 1; let g = (x) => apply(f, [x]); println(apply(g, [20]))')
assert_output "apply double nesting" "21" "$output"

# Triple nesting: 6+ boundary crossings
output=$(run_ish 'let f = (x) => x + 1; let g = (x) => apply(f, [x]); let h = (x) => apply(g, [x]); println(apply(h, [30]))')
assert_output "apply triple nesting" "31" "$output"

# Closure capture across boundaries
output=$(run_ish 'let base = 100; println(apply((x) => x + base, [5]))')
assert_output "apply closure capture" "105" "$output"

# Closure capture with nested apply
output=$(run_ish 'let base = 100; let add_base = (x) => x + base; println(apply((x) => apply(add_base, [x]), [42]))')
assert_output "apply nested closure" "142" "$output"

finish
