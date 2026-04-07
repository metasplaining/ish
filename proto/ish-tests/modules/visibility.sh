#!/usr/bin/env bash
# ---
# feature: Visibility
# docs: docs/spec/modules.md
# section: Visibility
# ---
# Tests for visibility enforcement: priv, pkg, pub.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "--- Visibility ---"

make_project() {
    local dir
    dir=$(mktemp -d)
    echo '{}' > "$dir/project.json"
    mkdir -p "$dir/src"
    echo "$dir"
}

# pkg item accessible from same project (no visibility keyword = pkg default)
proj=$(make_project)
cat > "$proj/src/a.ish" << 'EOF'
fn internal() { return 99 }
EOF
cat > "$proj/src/main.ish" << 'EOF'
use a { internal }
println(internal())
EOF
output=$("$ISH" "$proj/src/main.ish" 2>&1)
assert_output "pkg item accessible same project" "99" "$output"
rm -rf "$proj"

# priv item inaccessible from other module
proj=$(make_project)
cat > "$proj/src/a.ish" << 'EOF'
priv fn secret() { return 1 }
EOF
cat > "$proj/src/main.ish" << 'EOF'
use a { secret }
println(secret())
EOF
output=$("$ISH" "$proj/src/main.ish" 2>&1)
assert_output_contains "priv item inaccessible other module" "Access denied" "$output"
rm -rf "$proj"

# pub item accessible from any context
proj=$(make_project)
cat > "$proj/src/a.ish" << 'EOF'
pub fn exported() { return 42 }
EOF
cat > "$proj/src/main.ish" << 'EOF'
use a { exported }
println(exported())
EOF
output=$("$ISH" "$proj/src/main.ish" 2>&1)
assert_output "pub item accessible any context" "42" "$output"
rm -rf "$proj"

finish
