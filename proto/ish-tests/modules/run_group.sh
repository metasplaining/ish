#!/usr/bin/env bash
# Runs all test files in the modules/ group.
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
FAILURES=0

for test_file in "$SCRIPT_DIR"/*.sh; do
    [[ "$(basename "$test_file")" == "run_group.sh" ]] && continue
    echo ""
    if bash "$test_file"; then
        :
    else
        FAILURES=$((FAILURES + 1))
    fi
done

if [[ $FAILURES -gt 0 ]]; then
    exit 1
fi
