#!/usr/bin/env bash
# ---
# feature: Variables
# docs: docs/spec/syntax.md
# section: Variables and Expressions
# ---
# Tests variable declaration with let, immutable bindings, and
# multiple declarations on a single line.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "--- Variables ---"

# let binding with integer
output=$(run_ish 'let x = 5; println(x)')
assert_output "let integer binding" "5" "$output"

# let binding with string
output=$(run_ish 'let s = "hello"; println(s)')
assert_output "let string binding" "hello" "$output"

# let binding with boolean
output=$(run_ish 'let b = true; println(b)')
assert_output "let boolean binding" "true" "$output"

# let binding with null
output=$(run_ish 'let n = null; println(n)')
assert_output "let null binding" "null" "$output"

# let binding with float
output=$(run_ish 'let f = 3.14; println(f)')
assert_output "let float binding" "3.14" "$output"

# multiple let bindings
output=$(run_ish 'let a = 1; let b = 2; println(a); println(b)')
assert_output "multiple let bindings" $'1\n2' "$output"

# let with expression value
output=$(run_ish 'let x = 2 + 3; println(x)')
assert_output "let with expression" "5" "$output"

# let mut declaration
output=$(run_ish 'let mut y = 10; println(y)')
assert_output "let mut declaration" "10" "$output"

# variable used in expression
output=$(run_ish 'let x = 10; let y = x + 5; println(y)')
assert_output "variable in expression" "15" "$output"

# braced env variable in interpolated string
expected_pwd="$(pwd)"
output=$(run_ish 'println("${PWD}")')
assert_output "braced env variable in string interpolation" "$expected_pwd" "$output"

# shell quoted string interpolates ish vars and env vars
output=$(run_ish 'let myVar = 45; println("println {myVar} ${PWD}"); echo "echo {myVar} ${PWD}"')
expected=$'println 45 '"$expected_pwd"$'\necho 45 '"$expected_pwd"
assert_output "shell quoted interpolation with ish and env vars" "$expected" "$output"

# undefined variable produces error
assert_exit_code "undefined variable is error" 1 'println(undefined_var)'

finish
