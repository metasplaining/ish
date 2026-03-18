#!/usr/bin/env bash
# ---
# feature: For Loop
# docs: docs/spec/syntax.md
# section: Control Flow
# ---
# Tests for-each loops over lists.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "--- For Loop ---"

# Basic for-each over list
output=$(run_ish 'for i in [1, 2, 3] { println(i) }')
assert_output "for-each list" $'1\n2\n3' "$output"

# For-each with string list
output=$(run_ish 'for s in ["a", "b", "c"] { println(s) }')
assert_output "for-each string list" $'a\nb\nc' "$output"

# For-each over variable
output=$(run_ish 'let items = [10, 20, 30]; for item in items { println(item) }')
assert_output "for-each variable" $'10\n20\n30' "$output"

# For-each with body using expressions
output=$(run_ish 'for i in [1, 2, 3] { println(i * 10) }')
assert_output "for-each with expression" $'10\n20\n30' "$output"

# For-each building a new list
output=$(run_ish 'let result = []; for i in [1, 2, 3] { list_push(result, i * 2) }; println(result)')
assert_output "for-each building list" "[2, 4, 6]" "$output"

# For-each over empty list (no body executes)
output=$(run_ish 'for i in [] { println(i) }; println("done")')
assert_output "for-each empty list" "done" "$output"

# Nested for loops
output=$(run_ish 'for i in [1, 2] { for j in [10, 20] { println(i * j) } }')
assert_output "nested for loops" $'10\n20\n20\n40' "$output"

# For-each with function call in body
output=$(run_ish 'fn double(x) { return x * 2 }; for i in [1, 2, 3] { println(double(i)) }')
assert_output "for-each calling function" $'2\n4\n6' "$output"

finish
