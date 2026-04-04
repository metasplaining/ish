#!/usr/bin/env bash
# Tests for shell command integration in async context.
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "concurrency / shell_integration"

# --- Shell command works in normal context ---
output=$(run_ish 'let r = $(echo hello); println(r)')
assert_output "basic shell command" "hello" "$output"

# --- Shell command in awaited task ---
output=$(run_ish '
async fn work() {
  let r = $(echo from_spawn)
  return r
}
let r = await work()
println(r)
')
assert_output "shell command in awaited task" "from_spawn" "$output"

# --- println from awaited task visible ---
output=$(run_ish '
async fn work() {
  println("from_task")
  return 1
}
await work()
')
assert_output "println from awaited task" "from_task" "$output"

# --- Multiple shell commands sequentially ---
output=$(run_ish '
let a = $(echo first)
let b = $(echo second)
println(a + " " + b)
')
assert_output "sequential shell commands" "first second" "$output"

# --- Shell pipe ---
output=$(run_ish 'echo hello world | tr " " "_"')
assert_output "shell pipe" "hello_world" "$output"

finish
