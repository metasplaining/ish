#!/usr/bin/env bash
set -euo pipefail

# Find .unwrap() and .expect() calls in Rust source files.
# These are Rust-specific code smells: panicking on error instead of
# propagating it properly. In prototype code some are acceptable, but
# clusters in library code (ish-vm, ish-runtime) warrant attention.
# Usage: unwrap-check.sh [directory]

TARGET_DIR="${1:-proto}"

echo "Scanning for .unwrap() and .expect() calls in: $TARGET_DIR"
echo "(Excludes proto/target/ and test modules)"
echo ""

echo "=== .unwrap() calls ==="
grep -rn --include="*.rs" '\.unwrap()' "$TARGET_DIR" \
    | grep -v "/target/" \
    | grep -v "#\[cfg(test)\]" \
    | sort

echo ""
echo "=== .expect() calls ==="
grep -rn --include="*.rs" '\.expect(' "$TARGET_DIR" \
    | grep -v "/target/" \
    | grep -v "#\[cfg(test)\]" \
    | sort

echo ""
# Summarize by file
echo "=== Summary by file ==="
grep -rn --include="*.rs" -E '\.unwrap\(\)|\.expect\(' "$TARGET_DIR" \
    | grep -v "/target/" \
    | grep -v "#\[cfg(test)\]" \
    | cut -d: -f1 \
    | sort | uniq -c | sort -rn \
    | awk '{ printf "%4d  %s\n", $1, $2 }'

echo ""
echo "Done."
