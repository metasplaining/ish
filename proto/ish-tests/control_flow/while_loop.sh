#!/usr/bin/env bash
# ---
# feature: While Loop
# docs: docs/spec/syntax.md
# section: Control Flow
# ---
# Tests while loops with various conditions. Note: simple variable
# reassignment (x = x + 1) is not yet supported in the parser, so
# these tests use object property mutation or list-based patterns.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "--- While Loop ---"

# While with object-based counter (since bare x = x + 1 isn't parsed)
output=$(run_ish 'let c = {v: 0}; while c.v < 3 { println(c.v); c.v = c.v + 1 }')
assert_output "while with counter" $'0\n1\n2' "$output"

# While false never executes
output=$(run_ish 'while false { println("never") }; println("done")')
assert_output "while false skipped" "done" "$output"

# While with list mutation
output=$(run_ish 'let l = [1, 2, 3]; let r = {v: 0}; while r.v < list_length(l) { println(list_get(l, r.v)); r.v = r.v + 1 }')
assert_output "while iterating list" $'1\n2\n3' "$output"

# While with complex condition
output=$(run_ish 'let s = {v: 1}; while s.v < 100 and s.v > 0 { s.v = s.v * 2 }; println(s.v)')
assert_output "while with compound condition" "128" "$output"

# Nested while
output=$(run_ish 'let i = {v: 0}; while i.v < 2 { let j = {v: 0}; while j.v < 2 { println(str_concat(to_string(i.v), to_string(j.v))); j.v = j.v + 1 }; i.v = i.v + 1 }')
assert_output "nested while" $'00\n01\n10\n11' "$output"

finish
