#!/usr/bin/env bash
# ---
# feature: Assurance Ledger — Standards
# docs: docs/spec/assurance-ledger.md
# section: Standards
# ---
# Tests for standard definition, built-in standards, and standard application.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "--- Standards ---"

# Built-in standards exist
output=$(run_ish 'println(has_standard("streamlined"))')
assert_output "built-in streamlined exists" "true" "$output"

output=$(run_ish 'println(has_standard("cautious"))')
assert_output "built-in cautious exists" "true" "$output"

output=$(run_ish 'println(has_standard("rigorous"))')
assert_output "built-in rigorous exists" "true" "$output"

# Non-existent standard
output=$(run_ish 'println(has_standard("nonexistent"))')
assert_output "nonexistent standard returns false" "false" "$output"

# Custom standard definition
output=$(run_ish '
standard my_std [
    types(required, build),
    null_safety(optional, runtime)
]
println(has_standard("my_std"))
')
assert_output "custom standard definition" "true" "$output"

# Standard inheritance (extends)
output=$(run_ish '
standard strict_math extends cautious [
    overflow(required, build)
]
println(has_standard("strict_math"))
')
assert_output "standard with extends registered" "true" "$output"

# Inherited features visible under child standard
output=$(run_ish '
standard strict_math extends cautious [
    overflow(required, build)
]
@standard[strict_math]
let t: String = feature_state("types")
@standard[strict_math]
let o: String = feature_state("overflow")
println(t)
println(o)
')
assert_output "inherited features resolve" $'required/runtime\nrequired/build' "$output"

finish
