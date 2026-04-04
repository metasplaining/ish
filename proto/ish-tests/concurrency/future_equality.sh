#!/usr/bin/env bash
# Tests for FutureRef identity equality (Feature 1).
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "concurrency / future_equality"

# --- Same future is equal to itself (reflexivity) ---
output=$(run_ish 'async fn work() { return 42 }; let f = spawn work(); println(f == f)')
assert_output "future reflexive equality" "true" "$output"

# --- Two independent futures are not equal ---
output=$(run_ish 'async fn work() { return 42 }; let f1 = spawn work(); let f2 = spawn work(); println(f1 == f2)')
assert_output "independent futures not equal" "false" "$output"

# --- Cloned future reference is equal ---
output=$(run_ish 'async fn work() { return 42 }; let f = spawn work(); let g = f; println(f == g)')
assert_output "cloned future reference equal" "true" "$output"

finish
