#!/usr/bin/env bash
# ---
# feature: If / Else
# docs: docs/spec/syntax.md
# section: Control Flow
# ---
# Tests if statements, else branches, else-if chains, and nested
# conditionals. Parentheses around conditions are prohibited in ish.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "--- If / Else ---"

# Simple if true
output=$(run_ish 'if true { println("yes") }')
assert_output "if true" "yes" "$output"

# Simple if false (no output)
output=$(run_ish 'if false { println("no") }; println("done")')
assert_output "if false skipped" "done" "$output"

# If-else
output=$(run_ish 'if false { println("a") } else { println("b") }')
assert_output "if-else takes else" "b" "$output"

# If with comparison
output=$(run_ish 'if 5 > 3 { println("greater") }')
assert_output "if with comparison" "greater" "$output"

# Else-if chain
output=$(run_ish 'let x = 2; if x == 1 { println("one") } else if x == 2 { println("two") } else { println("other") }')
assert_output "else-if chain" "two" "$output"

# Nested if
output=$(run_ish 'if true { if false { println("a") } else { println("b") } }')
assert_output "nested if-else" "b" "$output"

# If with logical operators
output=$(run_ish 'if true and true { println("both") }')
assert_output "if with and" "both" "$output"

output=$(run_ish 'if false or true { println("either") }')
assert_output "if with or" "either" "$output"

# If with not
output=$(run_ish 'if not false { println("negated") }')
assert_output "if with not" "negated" "$output"

# If as last statement (no explicit return needed at top level)
output=$(run_ish 'let x = 10; if x > 5 { println("big") } else { println("small") }')
assert_output "if value comparison" "big" "$output"

finish
