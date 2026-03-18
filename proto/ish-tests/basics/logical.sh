#!/usr/bin/env bash
# ---
# feature: Logical Operators
# docs: docs/spec/syntax.md
# section: Operators
# ---
# Tests logical operators: and, or, not

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "--- Logical Operators ---"

# and - both true
output=$(run_ish 'println(true and true)')
assert_output "true and true" "true" "$output"

# and - one false
output=$(run_ish 'println(true and false)')
assert_output "true and false" "false" "$output"

# or - both false
output=$(run_ish 'println(false or false)')
assert_output "false or false" "false" "$output"

# or - one true
output=$(run_ish 'println(false or true)')
assert_output "false or true" "true" "$output"

# not true
output=$(run_ish 'println(not true)')
assert_output "not true" "false" "$output"

# not false
output=$(run_ish 'println(not false)')
assert_output "not false" "true" "$output"

# compound logical
output=$(run_ish 'println(true and not false)')
assert_output "true and not false" "true" "$output"

# logical with comparison
output=$(run_ish 'println(1 < 2 and 3 > 2)')
assert_output "comparison with and" "true" "$output"

# logical with comparison (or)
output=$(run_ish 'println(1 > 2 or 3 > 2)')
assert_output "comparison with or" "true" "$output"

# truthiness: 0 is falsy
output=$(run_ish 'println(0 or "fallback")')
assert_output "0 is falsy in or" "fallback" "$output"

finish
