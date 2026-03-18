#!/usr/bin/env bash
# ---
# feature: Type Checking
# docs: docs/spec/types.md
# section: Primitive Types
# ---
# Tests type_of, is_type, and type conversion builtins.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "--- Type Checking ---"

# type_of for each type
output=$(run_ish 'println(type_of(42))')
assert_output "type_of int" "int" "$output"

output=$(run_ish 'println(type_of(3.14))')
assert_output "type_of float" "float" "$output"

output=$(run_ish 'println(type_of("hello"))')
assert_output "type_of string" "string" "$output"

output=$(run_ish 'println(type_of(true))')
assert_output "type_of bool" "bool" "$output"

output=$(run_ish 'println(type_of(null))')
assert_output "type_of null" "null" "$output"

output=$(run_ish 'println(type_of([1, 2]))')
assert_output "type_of list" "list" "$output"

output=$(run_ish 'println(type_of({x: 1}))')
assert_output "type_of object" "object" "$output"

output=$(run_ish 'fn f() { return 1 }; println(type_of(f))')
assert_output "type_of function" "function" "$output"

output=$(run_ish 'let c = char("A"); println(type_of(c))')
assert_output "type_of char" "char" "$output"

# is_type
output=$(run_ish 'println(is_type(42, "int")); println(is_type(42, "string"))')
assert_output "is_type" $'true\nfalse' "$output"

# Conversions
output=$(run_ish 'println(to_string(42))')
assert_output "to_string int" "42" "$output"

output=$(run_ish 'println(to_int("123"))')
assert_output "to_int string" "123" "$output"

output=$(run_ish 'println(to_float("3.14"))')
assert_output "to_float string" "3.14" "$output"

# char builtin
output=$(run_ish 'println(char("A"))')
assert_output "char from string" "A" "$output"

output=$(run_ish 'println(char(65))')
assert_output "char from code point" "A" "$output"

finish
