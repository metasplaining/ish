#!/usr/bin/env bash
# Integration tests for yield budget, yield_every, and annotations.
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "concurrency / yield_budget"

# --- yield_every in while loop ---
output=$(run_ish 'let c = {v: 0}; while c.v < 5 yield every 2 { c.v = c.v + 1 }; println(c.v)')
assert_output "while with yield_every" "5" "$output"

# --- yield_every in for loop ---
output=$(run_ish 'let sum = {v: 0}; for x in [1, 2, 3, 4, 5] yield every 3 { sum.v = sum.v + x }; println(sum.v)')
assert_output "for with yield_every" "15" "$output"

# --- Sequential await tasks with yield ---
output=$(run_ish 'fn counter(n: int) { let c = {v: 0}; while c.v < n { c.v = c.v + 1; yield }; return c.v }; let r1 = await counter(3); let r2 = await counter(5); println(r1 + r2)')
assert_output "sequential awaits with yield" "8" "$output"

finish
