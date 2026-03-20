#!/usr/bin/env bash
# ---
# feature: Throw and Catch
# docs: docs/spec/errors.md
# section: Error Handling
# ---
# Tests throw, try/catch, entry-based error model, and error inspection builtins.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "--- Throw and Catch ---"

# Unhandled throw exits with code 1
assert_exit_code "unhandled throw exits 1" 1 'throw { message: "fail" }'

# Caught throw with error object
output=$(run_ish 'try { throw { message: "boom" } } catch (e) { println(error_message(e)) }')
assert_output "caught throw" "boom" "$output"

# Catch does not execute when no throw
output=$(run_ish 'try { println("ok") } catch (e) { println("caught") }')
assert_output "no throw, no catch" "ok" "$output"

# Error builtins — is_error checks for message: String property
output=$(run_ish 'let e = { message: "test" }; println(is_error(e)); println(is_error(42))')
assert_output "is_error builtin" $'true\nfalse' "$output"

output=$(run_ish 'let e = { message: "msg" }; println(error_message(e))')
assert_output "error_message builtin" "msg" "$output"

# error_code builtin
output=$(run_ish 'let e = { message: "x", code: "E002" }; println(error_code(e))')
assert_output "error_code builtin" "E002" "$output"

# error_code returns null for error without code
output=$(run_ish 'let e = { message: "x" }; println(error_code(e))')
assert_output "error_code null for uncoded" "null" "$output"

# Throw from function, caught at call site
output=$(run_ish 'fn fail() { throw { message: "inner" } }; try { fail() } catch (e) { println(error_message(e)) }')
assert_output "throw from function" "inner" "$output"

# Catch re-throw
output=$(run_ish 'try { try { throw { message: "deep" } } catch (e) { throw e } } catch (e2) { println(error_message(e2)) }')
assert_output "re-throw" "deep" "$output"

# Throw non-error value (gets wrapped by throw audit)
output=$(run_ish 'try { throw "plain string" } catch (e) { println(e.original) }')
assert_output "throw non-error value wrapped" "plain string" "$output"

# Error exit code from runtime error
assert_exit_code "undefined var runtime error" 1 'println(undefined_var)'

# Error exit code from parse error
assert_exit_code "parse error exits 1" 1 'let = ='

# --- Entry-based error identity ---

# is_error false for object without message
output=$(run_ish 'println(is_error({ name: "x" }))')
assert_output "is_error false without message" "false" "$output"

# is_error false for non-string message
output=$(run_ish 'println(is_error({ message: 42 }))')
assert_output "is_error false for non-string message" "false" "$output"

# --- System error codes ---

# Division by zero produces E002
output=$(run_ish 'fn test() { try { let x = 1 / 0 } catch (e) { return error_code(e) } }; println(test())')
assert_output "div by zero code E002" "E002" "$output"

# System errors pass is_error
output=$(run_ish 'fn test() { try { let x = 1 / 0 } catch (e) { return is_error(e) } }; println(test())')
assert_output "system error is_error true" "true" "$output"

# System errors have message
output=$(run_ish 'fn test() { try { let x = 1 / 0 } catch (e) { return error_message(e) } }; println(test())')
assert_output_contains "system error has message" "zero" "$output"

# Undefined variable produces E005
output=$(run_ish 'fn test() { try { let x = undefined_var } catch (e) { return error_code(e) } }; println(test())')
assert_output "undefined var code E005" "E005" "$output"

# Argument count mismatch produces E003
output=$(run_ish 'fn f(a) { return a }; fn test() { try { f(1, 2) } catch (e) { return error_code(e) } }; println(test())')
assert_output "arg count code E003" "E003" "$output"

# --- Throw audit ---
# Note: Throw audit tests are limited because @standard[name] can only
# annotate fn_decl/let_stmt by grammar, and the standard is scoped to
# the annotated statement (not active at function call time).
# Full throw audit coverage is in unit tests (interpreter.rs).

# Throw audit: standard annotates fn with throw inside
# (audit runs during fn_decl only, not at call time, so this is a
# compile-time-like check for the function body under the standard)
output=$(run_ish '
standard audit_std [
    types(optional, runtime)
]
@standard[audit_std]
fn test() {
    try {
        throw { message: "audited" }
    } catch (e) {
        return error_message(e)
    }
}
println(test())
')
assert_output "throw audit accepts message object" "audited" "$output"

finish
