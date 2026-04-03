#!/usr/bin/env bash
# Tests for yield control: explicit yield, yield every, unyielding, yield_budget.
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "concurrency / yield_control"

# --- Explicit yield statement ---
output=$(run_ish 'yield; println("after yield")')
assert_output "explicit yield" "after yield" "$output"

# --- yield every N in for loop ---
output=$(run_ish '
let c = { v: 0 }
for i in [1, 2, 3, 4, 5] yield every 2 {
  c.v = c.v + i
}
println(c.v)
')
assert_output "yield every in for loop" "15" "$output"

# --- yield every N in while loop ---
output=$(run_ish '
let c = { v: 0, i: 0 }
while c.i < 5 yield every 1 {
  c.i = c.i + 1
  c.v = c.v + c.i
}
println(c.v)
')
assert_output "yield every in while loop" "15" "$output"

# --- @[unyielding] suppresses yielding ---
output=$(run_ish '
@[unyielding] {
  let c = { v: 0, i: 0 }
  while c.i < 3 {
    c.i = c.i + 1
    c.v = c.v + c.i
  }
  println(c.v)
}
')
assert_output "unyielding suppresses yield" "6" "$output"

# --- @[yield_budget] changes threshold ---
output=$(run_ish '
@[yield_budget(100us)] {
  let c = { v: 0 }
  for i in [1, 2, 3] {
    c.v = c.v + i
  }
  println(c.v)
}
')
assert_output "yield_budget annotation works" "6" "$output"

# --- Two sequential await tasks with yield ---
output=$(run_ish '
fn task_a() { yield; return "a" }
fn task_b() { yield; return "b" }
let ra = await task_a()
let rb = await task_b()
println(ra + rb)
')
assert_output "sequential awaits with yield" "ab" "$output"

finish
