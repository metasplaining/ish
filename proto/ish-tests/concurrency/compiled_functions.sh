#!/usr/bin/env bash
# Tests for compiled function architecture (Feature 3).
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "concurrency / compiled_functions"

# --- println works without await (implied await at low assurance) ---
output=$(run_ish 'println("hello")')
assert_output "println without await" "hello" "$output"

# --- Builtins report correct arity errors (E003) ---
output=$(run_ish 'fn test() { try { type_of() } catch (e) { return error_code(e) } }; println(test())')
assert_output "type_of arity error" "E003" "$output"

# --- await unyielding builtin → E012 ---
output=$(run_ish 'fn test() { try { await type_of(42) } catch (e) { return error_code(e) } }; println(test())')
assert_output "await unyielding builtin E012" "E012" "$output"

# --- spawn unyielding builtin → E013 ---
output=$(run_ish 'fn test() { try { spawn type_of(42) } catch (e) { return error_code(e) } }; println(test())')
assert_output "spawn unyielding builtin E013" "E013" "$output"

# --- type_of(println) → "function" (not "builtin") ---
output=$(run_ish 'println(type_of(println))')
assert_output "type_of println is function" "function" "$output"

finish
