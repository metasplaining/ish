#!/usr/bin/env bash
# ---
# feature: Interface freeze
# docs: docs/spec/modules.md
# section: Interface Files
# ---
# Tests for `ish interface freeze` command.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "--- Interface Freeze ---"

make_project() {
    local dir
    dir=$(mktemp -d)
    echo '{}' > "$dir/project.json"
    mkdir -p "$dir/src"
    echo "$dir"
}

# freeze generates .ishi file
proj=$(make_project)
cat > "$proj/src/utils.ish" << 'EOF'
pub fn greet() -> String { return "hi" }
EOF
output=$(cd "$proj" && "$ISH" interface freeze 2>&1)
TESTS_RUN=$((TESTS_RUN + 1))
if [[ -f "$proj/src/utils.ishi" ]]; then
    echo "  pass: freeze generates ishi"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo "  FAIL: freeze generates ishi (file not found)"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi

# Check .ishi content contains pub fn greet
ishi_content=$(cat "$proj/src/utils.ishi" 2>/dev/null || echo "")
assert_output_contains "ishi contains pub fn greet" "pub fn greet" "$ishi_content"
rm -rf "$proj"

# freeze with target module
proj=$(make_project)
cat > "$proj/src/utils.ish" << 'EOF'
pub fn greet() -> String { return "hi" }
EOF
output=$(cd "$proj" && "$ISH" interface freeze utils 2>&1)
TESTS_RUN=$((TESTS_RUN + 1))
if [[ -f "$proj/src/utils.ishi" ]]; then
    echo "  pass: freeze with target"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo "  FAIL: freeze with target (file not found)"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi
rm -rf "$proj"

# freeze overwrites stale content
proj=$(make_project)
cat > "$proj/src/utils.ish" << 'EOF'
pub fn greet() -> String { return "hi" }
EOF
echo "stale content" > "$proj/src/utils.ishi"
output=$(cd "$proj" && "$ISH" interface freeze 2>&1)
ishi_content=$(cat "$proj/src/utils.ishi" 2>/dev/null || echo "")
assert_output_contains "freeze overwrites stale content" "pub fn greet" "$ishi_content"
rm -rf "$proj"

# freeze with no src/ directory
dir=$(mktemp -d)
output=$(cd "$dir" && "$ISH" interface freeze 2>&1)
assert_output_contains "freeze no src directory error" "no src/ directory" "$output"
rm -rf "$dir"

finish
