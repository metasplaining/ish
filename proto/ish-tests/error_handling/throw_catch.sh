#!/usr/bin/env bash
# ---
# feature: Throw and Catch
# docs: docs/spec/syntax.md
# section: Error Handling
# ---
# Tests throw, try/catch, error creation, and error inspection builtins.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "--- Throw and Catch ---"

# Unhandled throw exits with code 1
assert_exit_code "unhandled throw exits 1" 1 'throw new_error("fail")'

# Caught throw
output=$(run_ish 'try { throw new_error("boom") } catch (e) { println(error_message(e)) }')
assert_output "caught throw" "boom" "$output"

# Catch does not execute when no throw
output=$(run_ish 'try { println("ok") } catch (e) { println("caught") }')
assert_output "no throw, no catch" "ok" "$output"

# Error builtins
output=$(run_ish 'let e = new_error("test"); println(is_error(e)); println(is_error(42))')
assert_output "is_error builtin" $'true\nfalse' "$output"

output=$(run_ish 'let e = new_error("msg"); println(error_message(e))')
assert_output "error_message builtin" "msg" "$output"

# Throw from function, caught at call site
output=$(run_ish 'fn fail() { throw new_error("inner") }; try { fail() } catch (e) { println(error_message(e)) }')
assert_output "throw from function" "inner" "$output"

# Catch re-throw
output=$(run_ish 'try { try { throw new_error("deep") } catch (e) { throw e } } catch (e2) { println(error_message(e2)) }')
assert_output "re-throw" "deep" "$output"

# Throw non-error value
output=$(run_ish 'try { throw "plain string" } catch (e) { println(e) }')
assert_output "throw non-error value" "plain string" "$output"

# Error exit code from runtime error
assert_exit_code "undefined var runtime error" 1 'println(undefined_var)'

# Error exit code from parse error
assert_exit_code "parse error exits 1" 1 'let = ='

finish
