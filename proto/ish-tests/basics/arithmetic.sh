#!/usr/bin/env bash
# ---
# feature: Arithmetic Operators
# docs: docs/spec/syntax.md
# section: Operators
# ---
# Tests arithmetic operators: +, -, *, /, % on integers and floats.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "--- Arithmetic ---"

# Addition
output=$(run_ish 'println(2 + 3)')
assert_output "integer addition" "5" "$output"

# Subtraction
output=$(run_ish 'println(10 - 4)')
assert_output "integer subtraction" "6" "$output"

# Multiplication
output=$(run_ish 'println(3 * 7)')
assert_output "integer multiplication" "21" "$output"

# Division
output=$(run_ish 'println(20 / 4)')
assert_output "integer division" "5" "$output"

# Modulo
output=$(run_ish 'println(10 % 3)')
assert_output "integer modulo" "1" "$output"

# Float addition
output=$(run_ish 'println(1.5 + 2.5)')
assert_output "float addition" "4.0" "$output"

# Negative numbers (unary minus)
output=$(run_ish 'println(-5)')
assert_output "unary minus" "-5" "$output"

# Compound expression
output=$(run_ish 'println(2 + 3 * 4)')
assert_output "operator precedence" "14" "$output"

# Parenthesized expression
output=$(run_ish 'println((2 + 3) * 4)')
assert_output "parenthesized expression" "20" "$output"

# String concatenation with +
output=$(run_ish 'println("hello" + " " + "world")')
assert_output "string concatenation" "hello world" "$output"

finish
