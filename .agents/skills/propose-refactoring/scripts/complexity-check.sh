#!/usr/bin/env bash
set -euo pipefail

# Estimate cyclomatic complexity of Rust functions by counting
# control-flow branch points within each function body.
# Usage: complexity-check.sh [file-or-directory]

TARGET="${1:-proto}"
WARN_THRESHOLD=20
HIGH_THRESHOLD=50

echo "Checking complexity in: $TARGET"
echo "(Threshold: warn at ${WARN_THRESHOLD}, high at ${HIGH_THRESHOLD} branch points)"
echo ""

if [ -f "$TARGET" ]; then
    FILES="$TARGET"
else
    FILES=$(find "$TARGET" -type f -name "*.rs" | grep -v "/target/" | sort)
fi

for file in $FILES; do
    count=$(grep -cE \
        '\bif\b|\belse\b|\bfor\b|\bwhile\b|\bloop\b|\bmatch\b|=>[[:space:]]*\{|&&|\|\||\bbreak\b|\bcontinue\b|\breturn\b' \
        "$file" 2>/dev/null || echo 0)

    if [ "$count" -ge "$HIGH_THRESHOLD" ]; then
        echo "HIGH  ($count) $file"
    elif [ "$count" -ge "$WARN_THRESHOLD" ]; then
        echo "WARN  ($count) $file"
    fi
done | sort -rn -k2

echo ""
echo "Files below ${WARN_THRESHOLD} branch points are omitted."
echo "Done."
