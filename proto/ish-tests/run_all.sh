#!/usr/bin/env bash
# -----------------------------------------------------------------------
# Top-level acceptance test runner for ish.
#
# Builds ish-shell, then runs every group runner found under this
# directory. Exits 0 if all groups pass, 1 if any fail.
#
# Usage:
#   cd proto && bash ish-tests/run_all.sh
#   ISH=/path/to/ish-shell bash ish-tests/run_all.sh   # skip build
# -----------------------------------------------------------------------
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROTO_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

# Build unless the caller supplied ISH.
if [[ -z "${ISH:-}" ]]; then
    echo "Building ish-shell..."
    (cd "$PROTO_DIR" && cargo build -p ish-shell --quiet)
    export ISH="$PROTO_DIR/target/debug/ish-shell"
fi

echo "Using ish binary: $ISH"
echo ""

GROUPS_RUN=0
GROUPS_PASSED=0
GROUPS_FAILED=0

for group_runner in "$SCRIPT_DIR"/*/run_group.sh; do
    [[ -f "$group_runner" ]] || continue
    group_name="$(basename "$(dirname "$group_runner")")"
    echo "=== $group_name ==="

    GROUPS_RUN=$((GROUPS_RUN + 1))
    if bash "$group_runner"; then
        GROUPS_PASSED=$((GROUPS_PASSED + 1))
    else
        GROUPS_FAILED=$((GROUPS_FAILED + 1))
    fi
    echo ""
done

echo "==============================="
echo "Groups: $GROUPS_PASSED/$GROUPS_RUN passed"
if [[ $GROUPS_FAILED -gt 0 ]]; then
    echo "$GROUPS_FAILED group(s) FAILED"
    exit 1
else
    echo "All groups passed."
    exit 0
fi
