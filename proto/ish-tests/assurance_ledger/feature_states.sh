#!/usr/bin/env bash
# ---
# feature: Assurance Ledger — Feature States
# docs: docs/spec/assurance-ledger.md
# section: Feature States
# ---
# Tests for querying active feature states under different standards.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "--- Feature States ---"

# No active standard — feature_state returns null
output=$(run_ish 'println(feature_state("types"))')
assert_output "no standard — feature null" "null" "$output"

# Cautious standard — types required/runtime
output=$(run_ish '
@standard[cautious]
let s: String = feature_state("types")
println(s)
')
assert_output "cautious types" "required/runtime" "$output"

# Rigorous standard — types required/build (overrides cautious)
output=$(run_ish '
@standard[rigorous]
let s: String = feature_state("types")
println(s)
')
assert_output "rigorous types" "required/build" "$output"

# Custom standard with specific features
output=$(run_ish '
standard my_std [
    null_safety(optional, runtime),
    overflow(required, build)
]
@standard[my_std]
let n = feature_state("null_safety")
@standard[my_std]
let o = feature_state("overflow")
println(n)
println(o)
')
assert_output "custom feature states" $'optional/runtime\nrequired/build' "$output"

# Feature not defined in standard (use optional types to avoid annotation requirement)
output=$(run_ish '
standard minimal_std [
    types(optional, runtime)
]
@standard[minimal_std]
let u = feature_state("nonexistent_feature")
println(u)
')
assert_output "undefined feature returns null" "null" "$output"

finish
