#!/usr/bin/env bash
# ---
# feature: Type Narrowing
# docs: docs/spec/types.md, docs/spec/assurance-ledger.md
# section: Type Narrowing
# ---
# Tests for type narrowing behavior in if/else branches.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "--- Type Narrowing ---"

# Null exclusion: x != null narrows in true branch
output=$(run_ish '
standard typed_std [
    types(optional, runtime)
]
@standard[typed_std]
let x: i32 | null = 42
if x != null {
    println("not null")
} else {
    println("is null")
}
')
assert_output "null exclusion true branch" "not null" "$output"

# Null exclusion: else branch when null
output=$(run_ish '
standard typed_std [
    types(optional, runtime)
]
@standard[typed_std]
let x: i32 | null = null
if x != null {
    println("not null")
} else {
    println("is null")
}
')
assert_output "null exclusion else branch" "is null" "$output"

# x == null: true branch when null
output=$(run_ish '
standard typed_std [
    types(optional, runtime)
]
@standard[typed_std]
let x: i32 | null = null
if x == null {
    println("is null")
} else {
    println("not null")
}
')
assert_output "eq null true branch" "is null" "$output"

# x == null: else branch when not null
output=$(run_ish '
standard typed_std [
    types(optional, runtime)
]
@standard[typed_std]
let x: i32 | null = 42
if x == null {
    println("is null")
} else {
    println("not null")
}
')
assert_output "eq null else branch" "not null" "$output"

# Narrowing does not affect code after if/else
output=$(run_ish '
standard typed_std [
    types(optional, runtime)
]
@standard[typed_std]
let x: i32 | null = 42
if x != null {
    println("branch")
}
println("after")
')
assert_output "entry restoration after branch" $'branch\nafter' "$output"

# Narrowing with if-only (no else)
output=$(run_ish '
standard typed_std [
    types(optional, runtime)
]
@standard[typed_std]
let x: i32 | null = null
if x != null {
    println("not null")
}
println("done")
')
assert_output "narrowing if-only null" "done" "$output"

# Narrowing without types feature active (no crash)
output=$(run_ish '
let x = 42
if x != null {
    println("ok")
}
')
assert_output "no types feature no crash" "ok" "$output"

# Nested narrowing: inner if after outer null check
output=$(run_ish '
standard typed_std [
    types(optional, runtime)
]
@standard[typed_std]
let x: i32 | null = 42
if x != null {
    if x != null {
        println("double checked")
    }
}
')
assert_output "nested narrowing" "double checked" "$output"

finish
