#!/usr/bin/env bash
# ---
# feature: Comparison Operators
# docs: docs/spec/syntax.md
# section: Operators
# ---
# Tests comparison operators: ==, !=, <, >, <=, >=

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "--- Comparison ---"

# Equality
output=$(run_ish 'println(1 == 1)')
assert_output "equal integers" "true" "$output"

# Inequality
output=$(run_ish 'println(1 != 2)')
assert_output "not equal integers" "true" "$output"

# Less than
output=$(run_ish 'println(1 < 2)')
assert_output "less than true" "true" "$output"

output=$(run_ish 'println(2 < 1)')
assert_output "less than false" "false" "$output"

# Greater than
output=$(run_ish 'println(2 > 1)')
assert_output "greater than true" "true" "$output"

# Less than or equal
output=$(run_ish 'println(1 <= 1)')
assert_output "less than or equal" "true" "$output"

# Greater than or equal
output=$(run_ish 'println(2 >= 1)')
assert_output "greater than or equal" "true" "$output"

# String equality
output=$(run_ish 'println("abc" == "abc")')
assert_output "string equality" "true" "$output"

# String inequality
output=$(run_ish 'println("abc" != "def")')
assert_output "string inequality" "true" "$output"

# Boolean equality
output=$(run_ish 'println(true == true)')
assert_output "boolean equality" "true" "$output"

# Null equality
output=$(run_ish 'println(null == null)')
assert_output "null equality" "true" "$output"

finish
