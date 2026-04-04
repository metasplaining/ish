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
output=$(run_ish 'async fn some_fn() { return 1 }
                  @[unyielding] fn bad() { await some_fn() }')
assert_output_contains "unyielding fn with await errors" "E015" "$output"

# @[unyielding] function body containing yield → analyzer error at declaration
output=$(run_ish '@[unyielding] fn bad() { yield }')
assert_output_contains "unyielding fn with yield errors" "E015" "$output"

# @[unyielding] function body containing spawn → no error (spawn is allowed)
output=$(run_ish 'async fn some_fn() { return 1 }
                  @[unyielding] fn ok() { spawn some_fn() }
                  println("ok")')
assert_output "unyielding fn with spawn is valid" "ok" "$output"

finish
