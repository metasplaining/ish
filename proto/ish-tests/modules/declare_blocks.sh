#!/usr/bin/env bash
# ---
# feature: Declare blocks
# docs: docs/spec/modules.md
# section: Declare Blocks
# ---
# Tests for `declare { }` blocks and mutual recursion.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "--- Declare Blocks ---"

# Mutual recursion in declare block
output=$(run_ish 'declare { fn even(n) { if n == 0 { return true } else { return odd(n - 1) } }; fn odd(n) { if n == 0 { return false } else { return even(n - 1) } } }; println(even(4))')
assert_output "mutual recursion in declare block" "true" "$output"

# Declare block rejects non-declarations
output=$(run_ish 'declare { let x = 1 }')
assert_output_contains "declare block command error" "E020" "$output"

# Functions in declare block are visible after the block
output=$(run_ish 'declare { fn greet() { return "hello" } }; println(greet())')
assert_output "functions visible after declare block" "hello" "$output"

# Multiple functions in declare block
output=$(run_ish 'declare { fn add(a, b) { return a + b }; fn double(x) { return add(x, x) } }; println(double(5))')
assert_output "multiple functions in declare block" "10" "$output"

finish
