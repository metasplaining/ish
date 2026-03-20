#!/usr/bin/env bash
# ---
# feature: Type Narrowing
# docs: docs/spec/types.md, docs/spec/assurance-ledger.md
# section: Type Narrowing
# ---
# Tests for type narrowing behavior in if/else branches.
# Narrowing is unconditional — it works without any standard active.
# Tests assert both behavioral output AND ledger state.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "--- Type Narrowing ---"

# Null exclusion: x != null narrows in true branch — verify ledger state
output=$(run_ish '
let x: i32 | null = 42
if x != null {
    println(ledger_state("x"))
}
')
assert_output_contains "null exclusion narrows entries" "ExcludeNull" "$output"

# Null exclusion: else branch when null
output=$(run_ish '
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
let x: i32 | null = null
if x == null {
    println("is null")
} else {
    println("not null")
}
')
assert_output "eq null true branch" "is null" "$output"

# x == null: else branch narrows to exclude null
output=$(run_ish '
let x: i32 | null = 42
if x == null {
    println("is null")
} else {
    println(ledger_state("x"))
}
')
assert_output_contains "eq null else branch narrows" "ExcludeNull" "$output"

# Narrowing does not affect code after if/else (entries restored)
output=$(run_ish '
let x: i32 | null = 42
if x != null {
    println("branch")
}
println("after")
')
assert_output "entry restoration after branch" $'branch\nafter' "$output"

# Narrowing with if-only (no else) — still works without standard
output=$(run_ish '
let x: i32 | null = null
if x != null {
    println("not null")
}
println("done")
')
assert_output "narrowing if-only null" "done" "$output"

# Narrowing works without any standard active — verify ledger state
output=$(run_ish '
let x: i32 | null = 42
if x != null {
    println(has_entry("x", "ExcludeNull"))
}
')
assert_output "no standard has_entry check" "true" "$output"

# Nested narrowing: inner if after outer null check
output=$(run_ish '
let x: i32 | null = 42
if x != null {
    if x != null {
        println(has_entry("x", "ExcludeNull"))
    }
}
')
assert_output "nested narrowing" "true" "$output"

finish
