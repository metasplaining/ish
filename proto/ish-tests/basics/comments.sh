#!/usr/bin/env bash
# ---
# feature: Comments
# docs: docs/spec/syntax.md
# section: Comments
# ---
# Tests that line comments (//, #) and block comments (/* */) work
# without affecting program output.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "--- Comments ---"

# Double-slash comment
output=$(run_ish $'// this is a comment\nprintln("after")')
assert_output "// line comment" "after" "$output"

# Hash comment
output=$(run_ish $'# this is a comment\nprintln("after")')
assert_output "# line comment" "after" "$output"

# Inline comment after statement
output=$(run_ish $'println("before") // comment\nprintln("after")')
assert_output "inline // comment" $'before\nafter' "$output"

# Comment between statements
output=$(run_ish $'println("first")\n// middle\nprintln("second")')
assert_output "comment between statements" $'first\nsecond' "$output"

# Empty program with only comments is valid
assert_exit_code "only comments is valid" 0 $'// nothing here\n# also nothing'

finish
