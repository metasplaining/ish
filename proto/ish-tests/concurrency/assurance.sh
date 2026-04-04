#!/usr/bin/env bash
# Integration tests for concurrency assurance ledger features.
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "concurrency / assurance"

# --- async_annotation: no discrepancy at low assurance (custom optional standard) ---
output=$(run_ish '
standard low_async [ async_annotation(optional, runtime) ]
@standard[low_async]
fn work() { yield }
@standard[low_async]
{
  work()
  println("ok")
}
')
assert_output "async_annotation optional" "ok" "$output"

# --- async_annotation: discrepancy at high assurance ---
output=$(run_ish '
standard strict_async [ async_annotation(required, runtime) ]
fn work() { yield }
@standard[strict_async] { work() }
')
assert_output_contains "async_annotation required" "not declared" "$output"

# --- async_annotation: no discrepancy when function is async ---
output=$(run_ish '
standard strict_async [ async_annotation(required, runtime) ]
@standard[strict_async]
async fn work() { yield }
@standard[strict_async]
{
  work()
  println("ok")
}
')
assert_output "async fn passes async_annotation audit" "ok" "$output"

# --- future_drop: no discrepancy at low assurance ---
output=$(run_ish '
async fn work() { return 1 }
spawn work()
println("ok")
')
assert_output "future_drop optional at streamlined" "ok" "$output"

# --- future_drop: discrepancy when required ---
output=$(run_ish '
standard strict_drop [ future_drop(required, runtime) ]
@standard[strict_drop] {
  async fn work() { return 1 }
  spawn work()
}
')
assert_output_contains "future_drop required" "dropped without" "$output"

# --- future_drop: no discrepancy when future is awaited ---
output=$(run_ish '
standard strict_drop [ future_drop(required, runtime) ]
@standard[strict_drop] {
  async fn work() { return 1 }
  await work()
  println("ok")
}
')
assert_output "future awaited passes future_drop audit" "ok" "$output"

# --- guaranteed_yield: discrepancy for unyielding when required ---
output=$(run_ish '
standard strict_yield [ guaranteed_yield(required, runtime) ]
@standard[strict_yield]
@[unyielding] { let x = 1 }
')
assert_output_contains "guaranteed_yield required" "unyielding" "$output"

# --- guaranteed_yield: no discrepancy when optional ---
output=$(run_ish '
standard low_yield [ guaranteed_yield(optional, runtime) ]
@standard[low_yield]
@[unyielding] { let x = 1 }
println("ok")
')
assert_output "unyielding ok when optional" "ok" "$output"

# --- concurrency features registered on cautious ---
output=$(run_ish '
@standard[cautious]
let s1: String = feature_state("async_annotation")
println(s1)
')
assert_output "cautious has async_annotation" "required/runtime" "$output"

output=$(run_ish '
@standard[cautious]
let s2: String = feature_state("future_drop")
println(s2)
')
assert_output "cautious has future_drop" "required/runtime" "$output"

output=$(run_ish '
@standard[cautious]
let s3: String = feature_state("guaranteed_yield")
println(s3)
')
assert_output "cautious has guaranteed_yield" "required/runtime" "$output"

# --- concurrency features on rigorous include parameter ---
output=$(run_ish '
@standard[rigorous]
let s4: String = feature_state("yield_control")
println(s4)
')
assert_output "rigorous has yield_control with time_and_count" "required/build:time_and_count" "$output"

# --- Complexity and Yielding entry types exist ---
output=$(run_ish 'println(has_entry_type("Complexity"))')
assert_output "Complexity entry type exists" "true" "$output"

output=$(run_ish 'println(has_entry_type("Yielding"))')
assert_output "Yielding entry type exists" "true" "$output"

finish
