#!/usr/bin/env bash
# Tests for error propagation through async/await.
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "concurrency / error_propagation"

# --- Error in awaited function propagates to caller ---
output=$(run_ish '
fn failing() { throw { message: "oops" } }
try {
  await failing()
} catch (e) {
  println("caught: " + e.message)
}
')
assert_output "error propagates through await" "caught: oops" "$output"

# --- Error in non-spawned function propagates normally ---
output=$(run_ish '
fn failing() { throw { message: "sync_err" } }
try {
  failing()
} catch (e) {
  println("caught: " + e.message)
}
')
assert_output "sync error propagates normally" "caught: sync_err" "$output"

# --- try/catch around await catches error ---
output=$(run_ish '
fn work() { throw { message: "async_err" } }
try {
  let r = await work()
  println("should not reach")
} catch (e) {
  println(e.message)
}
')
assert_output "try catch around await" "async_err" "$output"

# --- defer runs even when awaited function errors ---
output=$(run_ish '
fn work() {
  defer println("cleanup")
  throw { message: "err" }
}
try {
  await work()
} catch (e) {
  println(e.message)
}
')
assert_output "defer runs on awaited task error" $'cleanup\nerr' "$output"

# --- Nested error propagation through await chain ---
output=$(run_ish '
fn inner() { throw { message: "deep" } }
fn outer() {
  return await inner()
}
try {
  await outer()
} catch (e) {
  println("got: " + e.message)
}
')
assert_output "nested error propagation" "got: deep" "$output"

# --- Successful result after catching previous error ---
output=$(run_ish '
fn failing() { throw { message: "fail" } }
fn success() { return 42 }
try { await failing() } catch (e) { println("err: " + e.message) }
let r = await success()
println(r)
')
assert_output "success after catching error" $'err: fail\n42' "$output"

finish
