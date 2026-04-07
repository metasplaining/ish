#!/usr/bin/env bash
# ---
# feature: Bootstrap
# docs: docs/spec/modules.md
# section: Bootstrap
# ---
# Tests for the `bootstrap` directive.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "--- Bootstrap ---"

make_project() {
    local dir
    dir=$(mktemp -d)
    echo '{}' > "$dir/project.json"
    mkdir -p "$dir/src"
    echo "$dir"
}

# bootstrap inside project → E021
proj=$(make_project)
cat > "$proj/src/main.ish" << 'EOF'
bootstrap 'file:///tmp/dummy.json'
EOF
output=$("$ISH" "$proj/src/main.ish" 2>&1)
assert_output_contains "bootstrap inside project error" "E021" "$output"
rm -rf "$proj"

# bootstrap outside project → no error (standalone script, no project.json ancestor)
standalone_dir=$(mktemp -d)
cat > "$standalone_dir/script.ish" << 'EOF'
bootstrap 'file:///tmp/dummy.json'
println("ok")
EOF
output=$("$ISH" "$standalone_dir/script.ish" 2>&1)
# Should not contain E021
TESTS_RUN=$((TESTS_RUN + 1))
if [[ "$output" == *"E021"* ]]; then
    echo "  FAIL: bootstrap outside project ok"
    echo "    expected no E021 but got: $output"
    TESTS_FAILED=$((TESTS_FAILED + 1))
else
    echo "  pass: bootstrap outside project ok"
    TESTS_PASSED=$((TESTS_PASSED + 1))
fi
rm -rf "$standalone_dir"

finish
