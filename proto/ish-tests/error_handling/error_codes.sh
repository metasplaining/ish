#!/usr/bin/env bash
# ---
# feature: Error Codes E001-E010
# docs: docs/errors/INDEX.md
# section: Error Handling
# ---
# Tests that each error code in the catalog is produced correctly,
# can be caught (where applicable), and carries the correct code.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "--- Error Codes ---"

# --- E001: Unhandled throw ---
# E001 can't be caught (by definition), so test exit code
assert_exit_code "E001: unhandled throw exits 1" 1 'throw { message: "boom" }'

# --- E002: Division by zero ---
output=$(run_ish 'fn test() { try { let x = 1 / 0 } catch (e) { return error_code(e) } }; println(test())')
assert_output "E002: division by zero" "E002" "$output"

output=$(run_ish 'fn test() { try { let x = 10 % 0 } catch (e) { return error_code(e) } }; println(test())')
assert_output "E002: modulo by zero" "E002" "$output"

# --- E003: Argument count mismatch ---
output=$(run_ish 'fn f(a) { return a }; fn test() { try { f(1, 2) } catch (e) { return error_code(e) } }; println(test())')
assert_output "E003: too many args" "E003" "$output"

output=$(run_ish 'fn f(a, b) { return a }; fn test() { try { f(1) } catch (e) { return error_code(e) } }; println(test())')
assert_output "E003: too few args" "E003" "$output"

# --- E004: Type mismatch ---
output=$(run_ish 'fn test() { try { let a = [1, 2]; let x = a[true] } catch (e) { return error_code(e) } }; println(test())')
assert_output "E004: list index wrong type" "E004" "$output"

output=$(run_ish 'fn test() { try { let x = -"hello" } catch (e) { return error_code(e) } }; println(test())')
assert_output "E004: negate string" "E004" "$output"

# --- E005: Undefined variable ---
output=$(run_ish 'fn test() { try { let x = undefined_var } catch (e) { return error_code(e) } }; println(test())')
assert_output "E005: undefined variable" "E005" "$output"

# --- E006: Not callable ---
output=$(run_ish 'fn test() { try { let x = 42; x() } catch (e) { return error_code(e) } }; println(test())')
assert_output "E006: call integer" "E006" "$output"

output=$(run_ish 'fn test() { try { let s = "hello"; s() } catch (e) { return error_code(e) } }; println(test())')
assert_output "E006: call string" "E006" "$output"

# --- E007: Index out of bounds ---
output=$(run_ish 'fn test() { try { let a = [1, 2]; let x = a[5] } catch (e) { return error_code(e) } }; println(test())')
assert_output "E007: index too high" "E007" "$output"

output=$(run_ish 'fn test() { try { let a = [1, 2]; let x = a[-1] } catch (e) { return error_code(e) } }; println(test())')
assert_output "E007: negative index" "E007" "$output"

# --- E008: File I/O error ---
output=$(run_ish 'fn test() { try { read_file("/nonexistent_path_ish_test/file.txt") } catch (e) { return error_code(e) } }; println(test())')
assert_output "E008: read nonexistent file" "E008" "$output"

# --- E009: Null unwrap ---
output=$(run_ish 'fn test() { try { let x = null; x? } catch (e) { return error_code(e) } }; println(test())')
assert_output "E009: null unwrap" "E009" "$output"

output=$(run_ish 'fn test() { try { let x = null; x? } catch (e) { return is_error(e) } }; println(test())')
assert_output "E009: null unwrap is error" "true" "$output"

# --- E010: Shell command error ---
# Shell command that doesn't exist should produce E010
output=$(run_ish 'fn test() { try { !this_command_absolutely_does_not_exist_ish_test } catch (e) { return error_code(e) } }; println(test())')
assert_output "E010: nonexistent shell command" "E010" "$output"

# --- All system errors are proper error objects ---
output=$(run_ish 'fn test() { try { let x = 1 / 0 } catch (e) { return is_error(e) } }; println(test())')
assert_output "system error is_error" "true" "$output"

output=$(run_ish 'fn test() { try { let x = 1 / 0 } catch (e) { return error_message(e) } }; println(test())')
assert_output_contains "system error has message" "zero" "$output"

# --- Throw audit: entry-based behavior ---

# Throw with message under standard → accepted (auto Error entry)
# Standard is active during let RHS evaluation, including called functions
output=$(run_ish '
standard audit_std [
    types(optional, runtime)
]
fn do_throw() {
    throw { message: "custom error" }
}
try {
    @standard[audit_std]
    let x = do_throw()
} catch (e) {
    println(error_message(e))
}
')
assert_output "throw audit: message object accepted" "custom error" "$output"

# Throw with message + code under standard → accepted (auto CodedError entry)
output=$(run_ish '
standard audit_std [
    types(optional, runtime)
]
fn do_throw() {
    throw { message: "coded error", code: "CUSTOM01" }
}
try {
    @standard[audit_std]
    let x = do_throw()
} catch (e) {
    println(error_message(e))
    println(error_code(e))
    println(is_error(e))
}
')
assert_output "throw audit: coded error accepted" $'coded error\nCUSTOM01\ntrue' "$output"

# Throw without message → object wraps in system error (unconditional)
output=$(run_ish '
fn do_throw() {
    throw { name: "not an error" }
}
try {
    let x = do_throw()
} catch (e) {
    println(error_code(e))
}
')
assert_output "throw audit: no message wraps" "E001" "$output"

# Throw non-object → wraps in system error (unconditional)
output=$(run_ish 'try { throw "plain string" } catch (e) { println(e.original) }')
assert_output "throw non-object: string wrapped" "plain string" "$output"

output=$(run_ish 'try { throw 42 } catch (e) { println(e.original) }')
assert_output "throw non-object: integer wrapped" "42" "$output"

finish
