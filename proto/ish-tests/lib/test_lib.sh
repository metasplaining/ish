#!/usr/bin/env bash
# -----------------------------------------------------------------------
# Test library for ish acceptance tests.
# Source this file at the top of each test script.
#
# Usage:
#   SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
#   source "$SCRIPT_DIR/../lib/test_lib.sh"
#
#   output=$(run_ish 'println("hello")')
#   assert_output "println string" "hello" "$output"
#
#   run_ish_check_exit "syntax error" 1 'let = ='
#
#   finish
# -----------------------------------------------------------------------

TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# Resolve the ish binary: honour ISH env var, else find relative to lib/
ISH="${ISH:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)/target/debug/ish-shell}"

if [[ ! -x "$ISH" ]]; then
    echo "ERROR: ish binary not found at $ISH"
    echo "Build with: cd proto && cargo build -p ish-shell"
    exit 1
fi

# --- Helpers ---

# Run ish in inline (-c) mode. Captures stdout+stderr.
run_ish() {
    "$ISH" -c "$1" 2>&1
}

# Run ish and capture exit code only.
run_ish_exit() {
    "$ISH" -c "$1" >/dev/null 2>&1
    echo $?
}

# Run ish and capture stdout (stderr discarded).
run_ish_stdout() {
    "$ISH" -c "$1" 2>/dev/null
}

# Run ish and capture stderr (stdout discarded).
run_ish_stderr() {
    "$ISH" -c "$1" 2>&1 >/dev/null
}

# --- Assertions ---

assert_output() {
    local name="$1" expected="$2" actual="$3"
    TESTS_RUN=$((TESTS_RUN + 1))
    if [[ "$actual" == "$expected" ]]; then
        echo "  pass: $name"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo "  FAIL: $name"
        echo "    expected: $(printf '%q' "$expected")"
        echo "    actual:   $(printf '%q' "$actual")"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
}

assert_exit_code() {
    local name="$1" expected="$2" code
    shift 2
    "$ISH" -c "$*" >/dev/null 2>&1
    code=$?
    TESTS_RUN=$((TESTS_RUN + 1))
    if [[ "$code" -eq "$expected" ]]; then
        echo "  pass: $name"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo "  FAIL: $name (expected exit $expected, got $code)"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
}

assert_output_contains() {
    local name="$1" substring="$2" actual="$3"
    TESTS_RUN=$((TESTS_RUN + 1))
    if [[ "$actual" == *"$substring"* ]]; then
        echo "  pass: $name"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo "  FAIL: $name"
        echo "    expected to contain: $substring"
        echo "    actual:              $(printf '%q' "$actual")"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
}

# --- Summary ---

finish() {
    echo ""
    echo "  $TESTS_PASSED/$TESTS_RUN passed"
    if [[ $TESTS_FAILED -gt 0 ]]; then
        exit 1
    fi
    exit 0
}
