#!/usr/bin/env bash
# ---
# feature: Code analyzer yielding classification
# docs: docs/architecture/vm.md
# section: Code Analyzer
# ---
# Tests that the code analyzer correctly classifies functions as yielding
# or unyielding at declaration time.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "--- Code Analyzer Classification ---"

# async fn is yielding → await succeeds
output=$(run_ish 'async fn work() { return 42 }; println(await work())')
assert_output "async fn classified yielding" "42" "$output"

# function containing await is yielding
output=$(run_ish 'async fn inner() { return 10 }; fn outer() { return await inner() }; println(await outer())')
assert_output "fn with await classified yielding" "10" "$output"

# function containing spawn is yielding — await wrapper() proves no E012
# The spawn inside creates a nested Future, so the result is a future type.
output=$(run_ish 'async fn inner() { return 5 }; fn wrapper() { return spawn inner() }; println(type_of(await wrapper()))')
assert_output "fn with spawn classified yielding" "future" "$output"

# function with no yielding nodes is unyielding → await throws E012
output=$(run_ish 'fn pure() { return 5 }; try { await pure() } catch (e) { println(error_code(e)) }')
assert_output "unyielding fn await is E012" "E012" "$output"

# function calling a yielding function is classified yielding (implied await propagation)
output=$(run_ish 'async fn yielding() { return 99 }; fn caller() { return yielding() }; println(await caller())')
assert_output "fn calling yielding fn classified yielding" "99" "$output"

# lambda classified correctly — unyielding
output=$(run_ish 'let f = () => 42; try { await f() } catch (e) { println(error_code(e)) }')
assert_output "unyielding lambda await is E012" "E012" "$output"

# lambda classified correctly — yielding (calls yielding fn)
output=$(run_ish 'async fn work() { return 7 }; let f = () => work(); println(await f())')
assert_output "yielding lambda via call" "7" "$output"

finish
