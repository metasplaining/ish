#!/usr/bin/env bash
# ---
# feature: Closures
# docs: docs/spec/syntax.md
# section: Closures
# ---
# Tests that functions capture variables from their enclosing scope
# and that captured variables maintain their state.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "--- Closures ---"

# Basic closure captures local variable
output=$(run_ish 'fn make() { let x = 10; return () => x }; println(make()())')
assert_output "closure captures variable" "10" "$output"

# Closure captures parameter
output=$(run_ish 'fn make(x) { return () => x * 2 }; let f = make(5); println(f())')
assert_output "closure captures parameter" "10" "$output"

# Multiple closures from same scope
output=$(run_ish 'fn make(x) { let getter = () => x; return getter }; println(make(42)())')
assert_output "closure getter" "42" "$output"

# Closure with argument
output=$(run_ish 'fn make(base) { return (x) => base + x }; let adder = make(10); println(adder(5)); println(adder(20))')
assert_output "closure with argument" $'15\n30' "$output"

# Nested closures
output=$(run_ish 'fn a(x) { return (y) => (z) => x + y + z }; println(a(1)(2)(3))')
assert_output "nested closures" "6" "$output"

# Closure over loop-created values
output=$(run_ish 'let fns = []; for i in [1, 2, 3] { let v = i; list_push(fns, () => v) }; for f in fns { println(f()) }')
assert_output "closures in loop" $'1\n2\n3' "$output"

finish
