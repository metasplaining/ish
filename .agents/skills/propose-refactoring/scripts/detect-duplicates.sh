#!/usr/bin/env bash
set -euo pipefail

# Detect duplicate or near-duplicate code blocks in Rust source files.
# Looks for non-trivial lines (>45 chars, not comments) that appear in
# more than one file.
# Usage: detect-duplicates.sh [directory]

TARGET_DIR="${1:-proto}"
MIN_LINE_LEN=45

echo "Scanning for duplicate lines in: $TARGET_DIR"
echo "(Min line length: ${MIN_LINE_LEN} chars, comments excluded)"
echo ""

TMPFILE=$(mktemp)
trap 'rm -f "$TMPFILE"' EXIT

# Collect all non-comment, non-blank, non-trivial lines with their file
find "$TARGET_DIR" -type f -name "*.rs" | grep -v "/target/" | sort | while read -r file; do
    grep -nE ".{${MIN_LINE_LEN},}" "$file" \
        | grep -vE "^\s*(//)|(^\s*#\[)|(^\s*//!)" \
        | while IFS=: read -r lineno content; do
            # Trim leading whitespace for comparison
            trimmed=$(echo "$content" | sed 's/^[[:space:]]*//')
            echo "$trimmed|$file:$lineno"
        done
done > "$TMPFILE"

# Find lines that appear in more than one distinct file
sort -t'|' -k1,1 "$TMPFILE" \
    | awk -F'|' '
    {
        line = $1
        loc  = $2
        if (line == prev_line) {
            # Extract file from loc (strip line number)
            split(loc, a, ":")
            split(prev_loc, b, ":")
            if (a[1] != b[1]) {
                if (!reported[line]++) {
                    print "Duplicate line:"
                    print "  Content: " substr(line, 1, 80)
                    print "  Found in:"
                }
                print "    " loc
            }
        }
        prev_line = line
        prev_loc  = loc
    }
    '

echo ""
echo "Done."
