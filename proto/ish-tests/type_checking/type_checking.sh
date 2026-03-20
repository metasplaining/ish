#!/usr/bin/env bash
# ---
# feature: Type Checking
# docs: docs/spec/types.md, docs/spec/assurance-ledger.md
# section: Type Checking
# ---
# Tests for type checking behavior under cautious standard.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "--- Type Checking ---"

# Correct type annotation passes
output=$(run_ish '
standard typed_std [
    types(optional, runtime)
]
@standard[typed_std]
let x: i32 = 42
println(x)
')
assert_output "correct i32 annotation" "42" "$output"

# Wrong type annotation fails
output=$(run_ish '
standard typed_std [
    types(optional, runtime)
]
@standard[typed_std]
let x: String = 42
println(x)
')
assert_output_contains "wrong annotation produces error" "type" "$output"

# Required annotation missing
output=$(run_ish '
standard strict_std [
    types(required, runtime)
]
@standard[strict_std]
let x = 42
println(x)
')
assert_output_contains "required missing annotation" "type" "$output"

# Optional annotation not required
output=$(run_ish '
standard opt_std [
    types(optional, runtime)
]
@standard[opt_std]
let x = 42
println(x)
')
assert_output "optional annotation not required" "42" "$output"

# String annotation correct
output=$(run_ish '
standard typed_std [
    types(optional, runtime)
]
@standard[typed_std]
let s: String = "hello"
println(s)
')
assert_output "correct String annotation" "hello" "$output"

# Bool annotation correct
output=$(run_ish '
standard typed_std [
    types(optional, runtime)
]
@standard[typed_std]
let b: bool = true
println(b)
')
assert_output "correct bool annotation" "true" "$output"

# Float annotation correct
output=$(run_ish '
standard typed_std [
    types(optional, runtime)
]
@standard[typed_std]
let f: f64 = 3.14
println(f)
')
assert_output "correct f64 annotation" "3.14" "$output"

# Function parameter type checking — correct
output=$(run_ish '
standard typed_std [
    types(optional, runtime)
]
fn greet(name: String) {
    return "Hello, " + name
}
@standard[typed_std]
let msg: String = greet("world")
println(msg)
')
assert_output "correct param type" "Hello, world" "$output"

# Function parameter type checking — wrong
output=$(run_ish '
standard typed_std [
    types(optional, runtime)
]
fn greet(name: String) {
    return "Hello, " + name
}
@standard[typed_std]
let msg = greet(42)
println(msg)
')
assert_output_contains "wrong param type" "type" "$output"

# Function return type checking — correct
output=$(run_ish '
standard typed_std [
    types(optional, runtime)
]
@standard[typed_std]
fn get_num() -> i32 {
    return 42
}
@standard[typed_std]
let n: i32 = get_num()
println(n)
')
assert_output "correct return type" "42" "$output"

# Function return type checking — wrong
output=$(run_ish '
standard typed_std [
    types(optional, runtime)
]
@standard[typed_std]
fn get_name() -> String {
    return 42
}
@standard[typed_std]
let n = get_name()
')
assert_output_contains "wrong return type" "type" "$output"

# Union type annotation — first alternative
output=$(run_ish '
standard typed_std [
    types(optional, runtime)
]
@standard[typed_std]
let x: i32 | String = 42
println(x)
')
assert_output "union type first alt" "42" "$output"

# Union type annotation — second alternative
output=$(run_ish '
standard typed_std [
    types(optional, runtime)
]
@standard[typed_std]
let x: i32 | String = "hello"
println(x)
')
assert_output "union type second alt" "hello" "$output"

# Nullable union accepts null
output=$(run_ish '
standard typed_std [
    types(optional, runtime)
]
@standard[typed_std]
let x: i32 | null = null
println(x)
')
assert_output "nullable union accepts null" "null" "$output"

# Non-nullable type rejects null
output=$(run_ish '
standard typed_std [
    types(optional, runtime)
]
@standard[typed_std]
let x: i32 = null
')
assert_output_contains "non-nullable rejects null" "type" "$output"

# No type checking without standard
output=$(run_ish '
let x: String = 42
println(x)
')
assert_output "no checking without standard" "42" "$output"

# Cautious built-in standard
output=$(run_ish '
@standard[cautious]
let x: i32 = 42
println(x)
')
assert_output "cautious correct annotation" "42" "$output"

# Cautious requires annotations
output=$(run_ish '
@standard[cautious]
let x = 42
')
assert_output_contains "cautious requires annotation" "type" "$output"

finish
