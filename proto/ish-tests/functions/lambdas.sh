#!/usr/bin/env bash
# ---
# feature: Lambdas
# docs: docs/spec/syntax.md
# section: Lambdas
# ---
# Tests arrow function (=>) syntax with both expression bodies
# and block bodies.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "--- Lambdas ---"

# Expression body lambda
output=$(run_ish 'let double = (x) => x * 2; println(double(5))')
assert_output "expression body lambda" "10" "$output"

# Multi-parameter lambda
output=$(run_ish 'let add = (a, b) => a + b; println(add(3, 4))')
assert_output "multi-param lambda" "7" "$output"

# Lambda with block body
output=$(run_ish 'let f = (x) => { let y = x * 2; return y + 1 }; println(f(5))')
assert_output "block body lambda" "11" "$output"

# Lambda called immediately
output=$(run_ish 'println(((x) => x * x)(7))')
assert_output "immediately invoked lambda" "49" "$output"

# Lambda stored in list
output=$(run_ish 'let ops = [(x) => x + 1, (x) => x * 2]; println(list_get(ops, 0)(5)); println(list_get(ops, 1)(5))')
assert_output "lambdas in list" $'6\n10' "$output"

# Zero-argument lambda
output=$(run_ish 'let f = () => 42; println(f())')
assert_output "zero-arg lambda" "42" "$output"

# Lambda as callback
output=$(run_ish 'fn apply(f, x) { return f(x) }; println(apply((x) => x + 100, 5))')
assert_output "lambda as callback" "105" "$output"

# Lambda that returns a lambda
output=$(run_ish 'let f = (x) => (y) => x + y; println(f(10)(20))')
assert_output "lambda returning lambda" "30" "$output"

finish
