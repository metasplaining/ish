#!/usr/bin/env bash
# check-glossary.sh — Find capitalized terms in docs that might need glossary entries.
# Scans for PascalCase and ALL_CAPS terms not present in GLOSSARY.md.
# Usage: bash docs/scripts/check-glossary.sh [docs_root]
set -euo pipefail

DOCS_ROOT="${1:-$(cd "$(dirname "$0")/.." && pwd)}"
PROJECT_ROOT="$(cd "$DOCS_ROOT/.." && pwd)"
GLOSSARY="$PROJECT_ROOT/GLOSSARY.md"

if [[ ! -f "$GLOSSARY" ]]; then
    echo "ERROR: GLOSSARY.md not found at $GLOSSARY"
    exit 1
fi

# Extract defined terms from glossary (lines starting with ##)
glossary_terms="$(grep -oP '^## \K.*' "$GLOSSARY" | tr '[:upper:]' '[:lower:]')"

# Known terms to ignore (common Markdown/code terms)
IGNORE="TODO|NOTE|FIXME|README|INDEX|YAML|JSON|API|URL|HTTP|HTTPS|HTML|CSS|SQL|CLI|IDE|ADR|OK|BSD|MIT|ASCII|UTF|PEG"

echo "Terms found in docs but not in GLOSSARY.md:"
echo "============================================="

found=0
while IFS= read -r -d '' file; do
    # Skip glossary itself and scripts
    if [[ "$file" == "$GLOSSARY" ]]; then continue; fi

    # Extract potential terms: PascalCase words (2+ capitals)
    grep -oP '\b[A-Z][a-z]+(?:[A-Z][a-z]+)+\b' "$file" 2>/dev/null | sort -u | while IFS= read -r term; do
        lower="$(echo "$term" | tr '[:upper:]' '[:lower:]')"
        if ! echo "$glossary_terms" | grep -qx "$lower"; then
            rel_file="${file#"$PROJECT_ROOT"/}"
            echo "  $term ($rel_file)"
            found=$((found + 1))
        fi
    done
done < <(find "$DOCS_ROOT" -name '*.md' -not -path '*/scripts/*' -print0)

echo ""
echo "Review these terms and add relevant ones to GLOSSARY.md."
