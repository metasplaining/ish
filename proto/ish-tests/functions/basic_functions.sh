#!/usr/bin/env bash
# ---
# feature: Functions
# docs: docs/spec/syntax.md
# section: Functions and Closures
# ---
# Tests function declarations, return values, multiple parameters,
# recursion, and nested functions.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "--- Functions ---"

# Simple function
output=$(run_ish 'fn greet() { return "hello" }; println(greet())')
assert_output "simple function" "hello" "$output"

# Function with parameters
output=$(run_ish 'fn add(a, b) { return a + b }; println(add(2, 3))')
assert_output "function with params" "5" "$output"

# Function with multiple statements
output=$(run_ish 'fn process(x) { let y = x * 2; let z = y + 1; return z }; println(process(5))')
assert_output "multi-statement function" "11" "$output"

# Recursion (factorial)
output=$(run_ish 'fn fact(n) { if n <= 1 { return 1 }; return n * fact(n - 1) }; println(fact(5))')
assert_output "recursive factorial" "120" "$output"

# Nested function declarations
output=$(run_ish 'fn outer() { fn inner() { return 42 }; return inner() }; println(outer())')
assert_output "nested functions" "42" "$output"

# Function with no return (returns null)
output=$(run_ish 'fn noop() { let x = 1 }; println(noop())')
assert_output "no explicit return" "null" "$output"

# Function as value (type_of)
output=$(run_ish 'fn f() { return 1 }; println(type_of(f))')
assert_output "function type" "function" "$output"

# Function passed as argument
output=$(run_ish 'fn apply(f, x) { return f(x) }; fn double(x) { return x * 2 }; println(apply(double, 5))')
assert_output "higher-order function" "10" "$output"

# Multiple return paths
output=$(run_ish 'fn abs(x) { if x < 0 { return -x }; return x }; println(abs(-5)); println(abs(3))')
assert_output "conditional return" $'5\n3' "$output"

# Function calling function
output=$(run_ish 'fn square(x) { return x * x }; fn sum_squares(a, b) { return square(a) + square(b) }; println(sum_squares(3, 4))')
assert_output "function calls function" "25" "$output"

finish
