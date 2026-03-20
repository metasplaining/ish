#!/usr/bin/env bash
# ---
# feature: With Blocks
# docs: docs/spec/syntax.md
# section: Error Handling
# ---
# Tests with blocks for resource management: basic usage, error handling,
# nesting, and interaction with defer.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "--- With Blocks ---"

# Basic with block — resource acquired and body executes
output=$(run_ish 'let r = {value: 42}; with (r = r) { println(r.value) }')
assert_output "with block basic usage" "42" "$output"

# With block — close method called on exit
output=$(run_ish '
let log = {entries: []}
let r = {
    value: 10,
    close: () => { println("closed") }
}
with (res = r) {
    println(res.value)
}
')
assert_output "with block calls close" $'10\nclosed' "$output"

# With block — close called even when body throws
output=$(run_ish '
let r = {
    value: 1,
    close: () => { println("closed") }
}
try {
    with (res = r) {
        println("before throw")
        throw { message: "boom" }
    }
} catch (e) {
    println(error_message(e))
}
')
assert_output "with block close on throw" $'before throw\nclosed\nboom' "$output"

# Nested with blocks — close in reverse order
output=$(run_ish '
let r1 = { close: () => { println("close r1") } }
let r2 = { close: () => { println("close r2") } }
with (a = r1) {
    with (b = r2) {
        println("body")
    }
}
')
assert_output "nested with blocks" $'body\nclose r2\nclose r1' "$output"

# Multiple resources in single with block — close in reverse order
output=$(run_ish '
let r1 = { close: () => { println("close r1") } }
let r2 = { close: () => { println("close r2") } }
with (a = r1, b = r2) {
    println("body")
}
')
assert_output "with block multiple resources" $'body\nclose r2\nclose r1' "$output"

# With block combined with defer
output=$(run_ish '
fn f() {
    defer println("defer runs")
    let r = { close: () => { println("close runs") } }
    with (res = r) {
        println("body")
    }
    return 0
}
f()
')
assert_output "with block combined with defer" $'body\nclose runs\ndefer runs' "$output"

# With block — resource without close method (no error)
output=$(run_ish 'let r = {value: 99}; with (x = r) { println(x.value) }')
assert_output "with block no close method" "99" "$output"

finish
