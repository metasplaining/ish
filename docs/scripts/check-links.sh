#!/usr/bin/env bash
# check-links.sh — Verify all Markdown cross-references resolve to existing files.
# Usage: bash docs/scripts/check-links.sh [docs_root]
set -euo pipefail

DOCS_ROOT="${1:-$(cd "$(dirname "$0")/.." && pwd)}"
PROJECT_ROOT="$(cd "$DOCS_ROOT/.." && pwd)"
errors=0

while IFS= read -r -d '' file; do
    dir="$(dirname "$file")"
    # Extract markdown links: [text](target) — skip external URLs and anchors-only
    grep -oP '\[(?:[^\]]*)\]\(\K[^)]+' "$file" 2>/dev/null | while IFS= read -r target; do
        # Strip anchor fragments
        target_path="${target%%#*}"
        # Skip empty (anchor-only links), external URLs, and mailto
        if [[ -z "$target_path" || "$target_path" =~ ^https?:// || "$target_path" =~ ^mailto: ]]; then
            continue
        fi
        # Resolve relative path
        resolved="$(cd "$dir" && realpath -m "$target_path" 2>/dev/null || echo "")"
        if [[ -z "$resolved" || ! -e "$resolved" ]]; then
            rel_file="${file#"$PROJECT_ROOT"/}"
            echo "BROKEN LINK: $rel_file -> $target"
            errors=$((errors + 1))
        fi
    done
done < <(find "$DOCS_ROOT" "$PROJECT_ROOT/GLOSSARY.md" "$PROJECT_ROOT/CONTRIBUTING.md" "$PROJECT_ROOT/AGENTS.md" "$PROJECT_ROOT/README.md" -name '*.md' -print0 2>/dev/null)

if [[ $errors -gt 0 ]]; then
    echo ""
    echo "Found $errors broken link(s)."
    exit 1
else
    echo "All links OK."
fi
