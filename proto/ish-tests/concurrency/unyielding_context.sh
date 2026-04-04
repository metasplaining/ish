#!/usr/bin/env bash
# ---
# feature: Unyielding context error detection
# docs: docs/spec/concurrency.md
# section: Yielding Classification and Validation
# ---
# Tests that the code analyzer rejects yielding nodes inside functions
# that are otherwise unyielding.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "--- Unyielding Context Errors ---"

# @[unyielding] function body containing await → analyzer error at declaration
output=$(run_ish '@[unyielding] fn bad() { await some_fn() }')
assert_output_contains "unyielding fn with await errors" "yielding" "$output"

# @[unyielding] function body containing spawn → analyzer error at declaration
output=$(run_ish '@[unyielding] fn bad() { spawn some_fn() }')
assert_output_contains "unyielding fn with spawn errors" "yielding" "$output"

# @[unyielding] function body containing yield → analyzer error at declaration
output=$(run_ish '@[unyielding] fn bad() { yield }')
assert_output_contains "unyielding fn with yield errors" "yielding" "$output"

finish
