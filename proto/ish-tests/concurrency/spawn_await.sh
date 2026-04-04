#!/usr/bin/env bash
# Integration tests for spawn/await concurrency primitives.
# After the grammar restriction, await and spawn must take function calls.
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "concurrency / spawn_await"

# --- Basic await of function call ---
output=$(run_ish 'async fn work() { return 42 }; let r = await work(); println(r)')
assert_output "await function call returns value" "42" "$output"

# --- Spawn returns a future value ---
output=$(run_ish 'async fn work() { return 42 }; let f = spawn work(); println(type_of(f))')
assert_output "spawn returns future" "future" "$output"

# --- Multiple sequential awaits ---
output=$(run_ish 'async fn a() { return 10 }; async fn b() { return 20 }; let ra = await a(); let rb = await b(); println(ra + rb)')
assert_output "multiple sequential awaits" "30" "$output"

# --- Yield statement ---
output=$(run_ish 'yield; println("after yield")')
assert_output "yield then continue" "after yield" "$output"

# --- Await captures closure ---
output=$(run_ish 'let x = 100; async fn work() { return x }; let r = await work(); println(r)')
assert_output "await captures variable" "100" "$output"

# --- Nested await ---
output=$(run_ish 'async fn inner() { return 5 }; async fn outer() { return await inner() }; println(await outer())')
assert_output "nested await" "5" "$output"

# --- Await function returning string ---
output=$(run_ish 'async fn greet() { return "hello" }; println(await greet())')
assert_output "await returning string" "hello" "$output"

# --- await non-call → parse error (Incomplete node) ---
output=$(run_ish 'let v = await 42; println(v)')
assert_output_contains "await non-call int is parse error" "unexpected end of input" "$output"

output=$(run_ish 'let v = await "hello"; println(v)')
assert_output_contains "await non-call string is parse error" "unexpected end of input" "$output"

# --- spawn non-call → parse error (Incomplete node) ---
output=$(run_ish 'let x = spawn 42')
assert_output_contains "spawn non-call int is parse error" "unexpected end of input" "$output"

output=$(run_ish 'let x = spawn "hello"')
assert_output_contains "spawn non-call string is parse error" "unexpected end of input" "$output"

# --- await unyielding function → E012 ---
output=$(run_ish '@[unyielding] fn pure() { return 5 }; fn test() { try { await pure() } catch (e) { return error_code(e) } }; println(test())')
assert_output "await unyielding is E012" "E012" "$output"

# --- spawn unyielding function → E013 ---
output=$(run_ish '@[unyielding] fn pure() { return 5 }; fn test() { try { spawn pure() } catch (e) { return error_code(e) } }; println(test())')
assert_output "spawn unyielding is E013" "E013" "$output"

# --- await unyielding function (analyzer-classified, no annotation) → E012 ---
output=$(run_ish 'fn unyielding() { return 5 }; fn test() { try { await unyielding() } catch (e) { return error_code(e) } }; println(test())')
assert_output "await fn without async is E012" "E012" "$output"

finish
