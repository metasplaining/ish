#!/usr/bin/env bash
# Tests for future cancellation behavior.
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "concurrency / cancellation"

# --- Drop a future without awaiting (no standard — no error at low assurance) ---
output=$(run_ish 'fn work() { return 1 }; spawn work(); println("ok")')
assert_output "drop future no error at low assurance" "ok" "$output"

# --- Spawn returns a future value ---
output=$(run_ish 'fn work() { return 1 }; let f = spawn work(); println(type_of(f))')
assert_output "spawn returns future type" "future" "$output"

# --- Defer runs in spawned+awaited task ---
output=$(run_ish '
fn work() {
  defer println("deferred")
  return 1
}
println(await work())
')
assert_output "defer runs in awaited task" $'deferred\n1' "$output"

# --- Error in awaited function propagates ---
output=$(run_ish '
fn failing() { throw { message: "boom" } }
try {
  await failing()
} catch (e) {
  println(e.message)
}
')
assert_output "error in await caught" "boom" "$output"

# --- Multiple sequential awaits ---
output=$(run_ish '
fn a() { return 1 }
fn c() { return 3 }
let ra = await a()
let rc = await c()
println(ra + rc)
')
assert_output "sequential awaits work" "4" "$output"

finish
