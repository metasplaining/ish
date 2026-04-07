#!/usr/bin/env bash
set -euo pipefail

# Find Rust functions that exceed a line-length threshold.
# Usage: long-functions.sh [directory] [min-lines]
# Defaults: directory=proto, min-lines=50

TARGET_DIR="${1:-proto}"
MIN_LINES="${2:-50}"

echo "Scanning for functions longer than ${MIN_LINES} lines in: $TARGET_DIR"
echo ""

find "$TARGET_DIR" -type f -name "*.rs" | grep -v "/target/" | sort | while read -r file; do
    awk -v file="$file" -v min="$MIN_LINES" '
    /^[[:space:]]*(pub[[:space:]]+)?(async[[:space:]]+)?fn[[:space:]]/ {
        fn_name = $0
        fn_start = NR
        brace_depth = 0
        in_fn = 1
    }
    in_fn {
        for (i = 1; i <= length($0); i++) {
            c = substr($0, i, 1)
            if (c == "{") brace_depth++
            if (c == "}") {
                brace_depth--
                if (brace_depth == 0) {
                    fn_len = NR - fn_start + 1
                    if (fn_len >= min) {
                        # Extract function name for display
                        name = fn_name
                        gsub(/^[[:space:]]+/, "", name)
                        gsub(/[[:space:]]*\{.*$/, "", name)
                        print file ":" fn_start ": " fn_len " lines — " name
                    }
                    in_fn = 0
                    fn_start = 0
                    brace_depth = 0
                }
            }
        }
    }
    ' "$file"
done | sort -t: -k3 -rn

echo ""
echo "Done. Functions listed in descending order of length."
