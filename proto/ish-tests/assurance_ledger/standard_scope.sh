#!/usr/bin/env bash
# ---
# feature: Assurance Ledger — Standard Scope
# docs: docs/spec/assurance-ledger.md
# section: Standard Scope
# ---
# Tests for standard scope push/pop behavior via annotations.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "--- Standard Scope ---"

# No active standard by default
output=$(run_ish 'println(active_standard())')
assert_output "no active standard" "null" "$output"

# Standard active only inside annotated statement
output=$(run_ish '
@standard[cautious]
let inside: String = active_standard()
let outside = active_standard()
println(inside)
println(outside)
')
assert_output "standard scoped to annotation" $'cautious\nnull' "$output"

# Standard active during function declaration, not during call
output=$(run_ish '
@standard[rigorous]
fn f() { return active_standard() }
println(f())
')
assert_output "standard not active during call" "null" "$output"

# Different standards in sequence
output=$(run_ish '
@standard[cautious]
let a: String = active_standard()
@standard[rigorous]
let b: String = active_standard()
let c = active_standard()
println(a)
println(b)
println(c)
')
assert_output "sequential standard annotations" $'cautious\nrigorous\nnull' "$output"

# Custom standard scope
output=$(run_ish '
standard my_std [
    types(optional, runtime)
]
@standard[my_std]
let s = active_standard()
println(s)
')
assert_output "custom standard in scope" "my_std" "$output"

# Feature state only available in scope
output=$(run_ish '
@standard[cautious]
let inside: String = feature_state("types")
let outside = feature_state("types")
println(inside)
println(outside)
')
assert_output "feature state scoped to annotation" $'required/runtime\nnull' "$output"

finish
