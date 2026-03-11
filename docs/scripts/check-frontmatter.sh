#!/usr/bin/env bash
# check-frontmatter.sh — Verify all Markdown files under docs/ have valid YAML frontmatter.
# Required fields: title, category, audience, status, last-verified, depends-on
# Usage: bash docs/scripts/check-frontmatter.sh [docs_root]
set -euo pipefail

DOCS_ROOT="${1:-$(cd "$(dirname "$0")/.." && pwd)}"
REQUIRED_FIELDS=("title" "category" "audience" "status" "last-verified" "depends-on")
errors=0

while IFS= read -r -d '' file; do
    # Check file starts with ---
    first_line="$(head -1 "$file")"
    if [[ "$first_line" != "---" ]]; then
        echo "MISSING FRONTMATTER: $file"
        errors=$((errors + 1))
        continue
    fi

    # Extract frontmatter (between first and second ---)
    frontmatter="$(sed -n '1,/^---$/{ /^---$/d; p; }' "$file" | tail -n +1)"
    # Actually extract between line 2 and the next ---
    frontmatter="$(awk 'NR==1{next} /^---$/{exit} {print}' "$file")"

    for field in "${REQUIRED_FIELDS[@]}"; do
        if ! echo "$frontmatter" | grep -qP "^${field}:"; then
            echo "MISSING FIELD '$field': $file"
            errors=$((errors + 1))
        fi
    done
done < <(find "$DOCS_ROOT" -name '*.md' -not -path '*/scripts/*' -print0)

if [[ $errors -gt 0 ]]; then
    echo ""
    echo "Found $errors frontmatter issue(s)."
    exit 1
else
    echo "All frontmatter OK."
fi
