#!/usr/bin/env bash
# ---
# feature: Strings
# docs: docs/spec/syntax.md
# section: Strings
# ---
# Tests string literals (single-quoted, double-quoted, interpolation),
# triple-quoted strings, and string builtins.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "--- Strings ---"

# Double-quoted string
output=$(run_ish 'println("hello world")')
assert_output "double-quoted string" "hello world" "$output"

# Single-quoted string
output=$(run_ish "println('hello world')")
assert_output "single-quoted string" "hello world" "$output"

# String interpolation
output=$(run_ish 'let name = "ish"; println("hello {name}")')
assert_output "string interpolation" "hello ish" "$output"

# Expression interpolation
output=$(run_ish 'println("result: {2 + 2}")')
assert_output "expression interpolation" "result: 4" "$output"

# String length
output=$(run_ish 'println(str_length("abc"))')
assert_output "str_length" "3" "$output"

# String contains
output=$(run_ish 'println(str_contains("hello world", "world"))')
assert_output "str_contains true" "true" "$output"

output=$(run_ish 'println(str_contains("hello world", "xyz"))')
assert_output "str_contains false" "false" "$output"

# String starts_with
output=$(run_ish 'println(str_starts_with("hello", "hel"))')
assert_output "str_starts_with" "true" "$output"

# String replace
output=$(run_ish 'println(str_replace("hello world", "world", "ish"))')
assert_output "str_replace" "hello ish" "$output"

# String split
output=$(run_ish 'println(str_split("a,b,c", ","))')
assert_output "str_split" "[a, b, c]" "$output"

# String to_upper / to_lower
output=$(run_ish 'println(str_to_upper("abc"))')
assert_output "str_to_upper" "ABC" "$output"

output=$(run_ish 'println(str_to_lower("ABC"))')
assert_output "str_to_lower" "abc" "$output"

# String trim
output=$(run_ish 'println(str_trim("  hello  "))')
assert_output "str_trim" "hello" "$output"

# String concatenation with +
output=$(run_ish 'println("a" + "b" + "c")')
assert_output "string concat with +" "abc" "$output"

finish
