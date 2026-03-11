#!/usr/bin/env bash
# check-stale.sh — Report documents whose last-verified date is older than a threshold.
# Usage: bash docs/scripts/check-stale.sh [days] [docs_root]
# Default threshold: 90 days
set -euo pipefail

THRESHOLD_DAYS="${1:-90}"
DOCS_ROOT="${2:-$(cd "$(dirname "$0")/.." && pwd)}"
today_epoch="$(date +%s)"
threshold_epoch=$((today_epoch - THRESHOLD_DAYS * 86400))
stale=0

while IFS= read -r -d '' file; do
    # Extract last-verified from frontmatter
    verified="$(awk 'NR==1{next} /^---$/{exit} /^last-verified:/{print $2}' "$file")"
    if [[ -z "$verified" ]]; then
        echo "NO DATE: $file (missing last-verified)"
        stale=$((stale + 1))
        continue
    fi
    # Parse date
    verified_epoch="$(date -d "$verified" +%s 2>/dev/null || echo 0)"
    if [[ "$verified_epoch" -lt "$threshold_epoch" ]]; then
        days_old=$(( (today_epoch - verified_epoch) / 86400 ))
        echo "STALE (${days_old}d): $file (last-verified: $verified)"
        stale=$((stale + 1))
    fi
done < <(find "$DOCS_ROOT" -name '*.md' -not -path '*/scripts/*' -print0)

if [[ $stale -gt 0 ]]; then
    echo ""
    echo "Found $stale stale/undated document(s) (threshold: ${THRESHOLD_DAYS} days)."
    exit 1
else
    echo "All documents are current (threshold: ${THRESHOLD_DAYS} days)."
fi
